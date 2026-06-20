//! `mercury-kt-witness` — a deployable KT **witness** (or **auditor**) for a Mercury relay.
//!
//! A witness is the standard CT/Sigsum answer to a relay that could show different histories to
//! different people (a *split view*). Independent operators co-sign the relay's published tree heads;
//! a client that requires a quorum of cosignatures on the head its proofs are bound to will reject a
//! forked head no witness vouched for. This binary is one such co-signer.
//!
//! Its one job is to be **safe**: it co-signs a head ONLY after [`mercury_kt::evaluate_head`] confirms
//! the head is validly log-signed AND an append-only extension of what this witness last vouched for.
//! Every other outcome is a fail-closed refusal — and a same-epoch fork, a rollback, or an
//! inconsistent extension is a *security signal* (a non-zero exit an operator should alert on), not a
//! transient error to retry past.
//!
//! Single-shot by design: one fetch→verify→co-sign round per invocation, persisting the head it last
//! co-signed to a small state file. Schedule it with cron / a systemd timer from a host INDEPENDENT
//! of the relay operator — that independence is the whole protection; the code only makes the witness
//! honest, it cannot make it independent.
//!
//! ```text
//! mercury-kt-witness --relay https://relay.example.com --key-file ./witness.key --index 0 \
//!     [--auditor] [--state ./witness-state.json] [--log-key <64-hex-ed25519-log-pubkey>]
//! ```
//! `--key-file` holds the witness's Ed25519 SIGNING key as 64 hex chars (a 32-byte seed). `--index`
//! is the `witness_index` the relay pinned for this key (omit/ignored with `--auditor`). `--log-key`
//! pins the relay's log key out-of-band (STRONGEST); without it the witness trusts-on-first-use the
//! key the relay serves at `/kt/vrf-key` and warns.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ed25519_dalek::{Signer, SigningKey};
use mercury_kt::{AppendOnlyProof, SignedTreeHead, WitnessDecision, WitnessRefusal, evaluate_head};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

const USAGE: &str = "usage: mercury-kt-witness --relay <url> --key-file <path> [--index <n>] \
[--auditor] [--state <path>] [--log-key <64-hex>]";

// ---------------------------------------------------------------------------------------------------
// Minimal hex (the relay ships its own copy; a witness binary should not pull a dependency for this).
// ---------------------------------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    s
}

/// Decode EXACTLY `N` bytes of hex (case-insensitive). Returns None on any non-hex character or a
/// length that is not `2*N` — a witness never coerces a malformed field into a fixed buffer.
fn hex_decode_n<const N: usize>(s: &str) -> Option<[u8; N]> {
    let s = s.as_bytes();
    if s.len() != 2 * N {
        return None;
    }
    let mut out = [0u8; N];
    for (i, byte) in out.iter_mut().enumerate() {
        let hi = (s[2 * i] as char).to_digit(16)?;
        let lo = (s[2 * i + 1] as char).to_digit(16)?;
        *byte = (hi * 16 + lo) as u8;
    }
    Some(out)
}

// ---------------------------------------------------------------------------------------------------
// Wire types — the SUBSET of the relay's JSON the witness reads (decoupled from the relay's internal
// types: a witness is an independent client of the wire format, not of the server crate).
// ---------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct WitnessedBundle {
    tree_size: u64,
    root_hash: String,     // hex 32
    timestamp_s: i64,
    log_signature: String, // hex 64
}

#[derive(Debug, Deserialize)]
struct VrfKeyBody {
    log_public_key: String, // hex 32
}

#[derive(Debug, Deserialize)]
struct ConsistencyBody {
    proof: AppendOnlyProof,
}

/// The witness's persisted commitment: the `(epoch, root)` it last co-signed. Read fail-closed — a
/// present-but-corrupt file is an ERROR, never silently treated as a first run (which would reset the
/// commitment and let a tampered state mask an equivocation).
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct LastSeen {
    epoch: u64,
    root_hex: String,
}

fn read_last_seen(path: &Path) -> Result<Option<(u64, [u8; 32])>, String> {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let ls: LastSeen =
                serde_json::from_str(&s).map_err(|e| format!("parse state {}: {e}", path.display()))?;
            let root = hex_decode_n::<32>(&ls.root_hex)
                .ok_or_else(|| format!("state {}: root_hex is not 32-byte hex", path.display()))?;
            Ok(Some((ls.epoch, root)))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read state {}: {e}", path.display())),
    }
}

fn write_last_seen(path: &Path, epoch: u64, root: &[u8; 32]) -> Result<(), String> {
    let ls = LastSeen { epoch, root_hex: hex_encode(root) };
    let s = serde_json::to_string(&ls).map_err(|e| format!("serialize state: {e}"))?;
    std::fs::write(path, s).map_err(|e| format!("write state {}: {e}", path.display()))
}

/// Build the cosignature POST body for `/kt/witness/cosign` (or `/kt/auditor/cosign` when `auditor`).
fn cosign_body(head: &SignedTreeHead, index: u32, sig: &[u8; 64], auditor: bool) -> String {
    let mut v = serde_json::json!({
        "tree_size": head.tree_size,
        "root_hash": hex_encode(&head.root_hash),
        "timestamp_s": head.timestamp_s,
        "signature": hex_encode(sig),
    });
    if !auditor {
        v["witness_index"] = serde_json::json!(index);
    }
    v.to_string()
}

// ---------------------------------------------------------------------------------------------------
// HTTP (blocking ureq; no `json` feature — bodies handled with serde_json explicitly). Fail-closed:
// any transport/parse error propagates as Err and the witness exits non-zero WITHOUT co-signing.
// ---------------------------------------------------------------------------------------------------

fn http_get(url: &str) -> Result<String, String> {
    match ureq::get(url).call() {
        Ok(resp) => resp.into_string().map_err(|e| format!("read body of {url}: {e}")),
        Err(e) => Err(format!("GET {url}: {e}")),
    }
}

fn http_post_json(url: &str, body: &str) -> Result<(), String> {
    match ureq::post(url)
        .set("Content-Type", "application/json")
        .send_string(body)
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(code, _)) => Err(format!("POST {url} -> HTTP {code}")),
        Err(e) => Err(format!("POST {url}: {e}")),
    }
}

// ---------------------------------------------------------------------------------------------------
// Config + one witness round.
// ---------------------------------------------------------------------------------------------------

struct Config {
    relay: String,
    signing_key: SigningKey,
    index: u32,
    auditor: bool,
    state_path: PathBuf,
    log_key: Option<[u8; 32]>,
}

enum Outcome {
    Cosigned(u64),
    Refused(WitnessRefusal),
}

async fn run_round(cfg: &Config) -> Result<Outcome, String> {
    // 1. The pinned log key (out-of-band is strongest; else trust-on-first-use the served key).
    let log_key: [u8; 32] = match cfg.log_key {
        Some(k) => k,
        None => {
            let body = http_get(&format!("{}/kt/vrf-key", cfg.relay))?;
            let vk: VrfKeyBody =
                serde_json::from_str(&body).map_err(|e| format!("parse /kt/vrf-key: {e}"))?;
            eprintln!(
                "mercury-kt-witness: WARNING trusting the relay-served log key on first use; pin it \
                 out-of-band with --log-key for real assurance"
            );
            hex_decode_n::<32>(&vk.log_public_key)
                .ok_or("/kt/vrf-key: log_public_key is not 32-byte hex")?
        }
    };

    // 2. The relay's current published head.
    let body = http_get(&format!("{}/kt/sth/witnessed", cfg.relay))?;
    let bundle: WitnessedBundle =
        serde_json::from_str(&body).map_err(|e| format!("parse /kt/sth/witnessed: {e}"))?;
    let head = SignedTreeHead {
        tree_size: bundle.tree_size,
        root_hash: hex_decode_n::<32>(&bundle.root_hash)
            .ok_or("witnessed head: root_hash is not 32-byte hex")?,
        timestamp_s: bundle.timestamp_s,
    };
    let log_sig = hex_decode_n::<64>(&bundle.log_signature)
        .ok_or("witnessed head: log_signature is not 64-byte hex")?;

    // 3. What we last vouched for.
    let last_seen = read_last_seen(&cfg.state_path)?;

    // 4. The append-only proof — fetched ONLY for a single-epoch advance (the only span the two
    //    endpoint roots can verify; evaluate_head fails closed on a wider gap regardless).
    let proof: Option<AppendOnlyProof> = match last_seen {
        Some((last_epoch, _)) if head.tree_size == last_epoch + 1 => {
            let url = format!(
                "{}/kt/consistency?from={}&to={}",
                cfg.relay, last_epoch, head.tree_size
            );
            let cbody = http_get(&url)?;
            let cb: ConsistencyBody =
                serde_json::from_str(&cbody).map_err(|e| format!("parse /kt/consistency: {e}"))?;
            Some(cb.proof)
        }
        _ => None,
    };

    // 5. The verified decision. Nothing below trusts the relay: the proof is checked against the
    //    pinned log key and our OWN remembered root.
    match evaluate_head(&log_key, last_seen, &head, &log_sig, proof).await {
        WitnessDecision::Cosign => {
            let sig = cfg.signing_key.sign(&head.signing_bytes()).to_bytes();
            let path = if cfg.auditor {
                "/kt/auditor/cosign"
            } else {
                "/kt/witness/cosign"
            };
            http_post_json(
                &format!("{}{}", cfg.relay, path),
                &cosign_body(&head, cfg.index, &sig, cfg.auditor),
            )?;
            // Only advance our commitment AFTER the relay accepted the cosignature.
            write_last_seen(&cfg.state_path, head.tree_size, &head.root_hash)?;
            Ok(Outcome::Cosigned(head.tree_size))
        }
        WitnessDecision::Refuse(r) => Ok(Outcome::Refused(r)),
    }
}

/// A human-readable line + a stable exit code per refusal. The equivocation refusals (and a bad log
/// signature) get exit 3 — alert-worthy; the fell-behind gap gets exit 4 — operational.
fn refusal_report(r: WitnessRefusal) -> (&'static str, u8) {
    match r {
        WitnessRefusal::BadLogSignature => (
            "the head is NOT validly signed by the pinned log key (wrong relay, wrong key, or tamper)",
            3,
        ),
        WitnessRefusal::EpochRegressed => (
            "the head's epoch went BACKWARDS vs what we co-signed: a rollback (equivocation signal)",
            3,
        ),
        WitnessRefusal::SameEpochFork => (
            "same epoch as last co-signed but a DIFFERENT root: the relay forked this epoch (equivocation signal)",
            3,
        ),
        WitnessRefusal::InconsistentExtension => (
            "the new head is NOT an append-only extension of what we vouched for: the split-view signal",
            3,
        ),
        WitnessRefusal::UnverifiableGap => (
            "fell more than one epoch behind, so the span cannot be verified; investigate, then delete \
             the state file to re-bootstrap from the current head",
            4,
        ),
    }
}

fn parse_args() -> Result<Config, String> {
    let mut relay: Option<String> = None;
    let mut key_file: Option<String> = None;
    let mut index: u32 = 0;
    let mut auditor = false;
    let mut state_path = PathBuf::from("./mercury-witness-state.json");
    let mut log_key: Option<[u8; 32]> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--relay" => relay = Some(args.next().ok_or("--relay needs a value")?),
            "--key-file" => key_file = Some(args.next().ok_or("--key-file needs a value")?),
            "--index" => {
                index = args
                    .next()
                    .ok_or("--index needs a value")?
                    .parse()
                    .map_err(|_| "--index must be a non-negative integer")?
            }
            "--auditor" => auditor = true,
            "--state" => state_path = PathBuf::from(args.next().ok_or("--state needs a value")?),
            "--log-key" => {
                let v = args.next().ok_or("--log-key needs a value")?;
                log_key =
                    Some(hex_decode_n::<32>(&v).ok_or("--log-key must be 64 hex chars (32 bytes)")?);
            }
            "-h" | "--help" => return Err("help".into()),
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    let relay = relay.ok_or("--relay is required")?;
    let relay = relay.trim_end_matches('/').to_string();
    let key_file = key_file.ok_or("--key-file is required")?;
    let signing_key = load_signing_key(&key_file)?;
    Ok(Config { relay, signing_key, index, auditor, state_path, log_key })
}

/// Read the witness's Ed25519 signing key (64 hex chars = a 32-byte seed) from a file, zeroizing the
/// file contents + the decoded seed once the key is built. `ed25519-dalek`'s `zeroize` feature wipes
/// the `SigningKey` itself on drop.
fn load_signing_key(path: &str) -> Result<SigningKey, String> {
    let mut contents =
        std::fs::read_to_string(path).map_err(|e| format!("read key file {path}: {e}"))?;
    let mut seed = match hex_decode_n::<32>(contents.trim()) {
        Some(s) => s,
        None => {
            contents.zeroize();
            return Err(format!(
                "key file {path} must contain 64 hex chars (a 32-byte Ed25519 seed)"
            ));
        }
    };
    let key = SigningKey::from_bytes(&seed);
    seed.zeroize();
    contents.zeroize();
    Ok(key)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cfg = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            if e == "help" {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            eprintln!("mercury-kt-witness: {e}\n{USAGE}");
            return ExitCode::from(64); // EX_USAGE
        }
    };
    match run_round(&cfg).await {
        Ok(Outcome::Cosigned(epoch)) => {
            let role = if cfg.auditor { "auditor" } else { "witness" };
            println!("mercury-kt-witness: {role} co-signed epoch {epoch}");
            ExitCode::SUCCESS
        }
        Ok(Outcome::Refused(r)) => {
            let (msg, code) = refusal_report(r);
            eprintln!("mercury-kt-witness: REFUSED to co-sign -- {msg}");
            ExitCode::from(code)
        }
        Err(e) => {
            eprintln!("mercury-kt-witness: error: {e}");
            ExitCode::from(70) // EX_SOFTWARE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_rejects_bad_input() {
        let bytes = [0x00, 0x0f, 0xa5, 0xff, 0x10];
        assert_eq!(hex_encode(&bytes), "000fa5ff10");
        assert_eq!(hex_decode_n::<5>("000fa5ff10"), Some(bytes));
        // Wrong length and non-hex are both rejected (never coerced).
        assert_eq!(hex_decode_n::<5>("000fa5ff"), None);
        assert_eq!(hex_decode_n::<2>("zzzz"), None);
        assert_eq!(hex_decode_n::<32>(&"ab".repeat(32)), Some([0xab; 32]));
    }

    #[test]
    fn parses_a_witnessed_bundle_into_a_head() {
        let json = serde_json::json!({
            "tree_size": 7,
            "root_hash": "ab".repeat(32),
            "timestamp_s": 1_900_000_000i64,
            "log_signature": "cd".repeat(64),
            "cosignatures": [],
            "witnesses": [],
            "auditor_signature": null,
            "auditor_public_key": null,
        })
        .to_string();
        // The witness reads only the subset it needs; extra fields are ignored.
        let bundle: WitnessedBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(bundle.tree_size, 7);
        let head = SignedTreeHead {
            tree_size: bundle.tree_size,
            root_hash: hex_decode_n::<32>(&bundle.root_hash).unwrap(),
            timestamp_s: bundle.timestamp_s,
        };
        assert_eq!(head.root_hash, [0xab; 32]);
        assert_eq!(hex_decode_n::<64>(&bundle.log_signature).unwrap(), [0xcd; 64]);
    }

    #[test]
    fn state_file_round_trips_and_a_missing_file_is_first_run() {
        let dir = std::env::temp_dir().join("mercury_witness_state_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        // Missing file = first run (None), NOT an error.
        assert_eq!(read_last_seen(&path).unwrap(), None);

        // Round-trip a commitment.
        let root = [0x42; 32];
        write_last_seen(&path, 5, &root).unwrap();
        assert_eq!(read_last_seen(&path).unwrap(), Some((5, root)));

        // A corrupt file fails CLOSED (Err), never silently re-bootstraps.
        std::fs::write(&path, "{not json").unwrap();
        assert!(read_last_seen(&path).is_err());
    }

    #[test]
    fn cosign_body_includes_index_for_a_witness_but_not_an_auditor() {
        let head = SignedTreeHead { tree_size: 3, root_hash: [0x11; 32], timestamp_s: 42 };
        let sig = [0x22; 64];

        let w: serde_json::Value =
            serde_json::from_str(&cosign_body(&head, 2, &sig, false)).unwrap();
        assert_eq!(w["tree_size"], 3);
        assert_eq!(w["witness_index"], 2);
        assert_eq!(w["root_hash"], "11".repeat(32));
        assert_eq!(w["signature"], "22".repeat(64));

        let a: serde_json::Value =
            serde_json::from_str(&cosign_body(&head, 2, &sig, true)).unwrap();
        assert!(a.get("witness_index").is_none(), "the auditor body carries no witness_index");
        assert_eq!(a["timestamp_s"], 42);
    }
}
