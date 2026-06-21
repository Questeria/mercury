//! Key-transparency directory service hosted on the relay.
//!
//! The relay is UNTRUSTED: it only serves AKD proofs that clients verify
//! cryptographically against the directory's VRF key. Serving a proof never
//! confers trust -- a tampered or substituted proof simply fails the client's
//! `mercury_kt::verify_*` check against the key the client pinned.
//!
//! Two write/bootstrap surfaces are exposed here, both with HONESTLY-BOUNDED guarantees:
//!
//! - **`POST /username/claim`** publishes a `username -> account_id` binding into the log
//!   (first-claimant-wins, authorized by an Ed25519 proof of possession that the claimer owns the
//!   account). This is a deliberate, narrow publish path for the username registry — NOT general
//!   device-key publishing. See [`crate::username`].
//! - **`GET /kt/vrf-key`** serves the directory's VRF + log public keys for TRUST-ON-FIRST-USE
//!   bootstrap. The STRONG model still pins these out-of-band (a relay malicious from first contact
//!   could serve its own keys); serving them lets a client pin on first use and DETECT a later key
//!   change. The residual (out-of-band pin + independent witnesses/gossip) is documented in
//!   `docs/USERNAMES.md`, not hidden. Crucially, even under TOFU the looked-up account id is then
//!   used to fetch a contact card that is INDEPENDENTLY self-authenticating, so a forged binding
//!   resolves to a card the user can inspect, never to a silently-trusted impostor.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mercury_kt::{
    AppendOnlyProof, HistoryProof, KtDirectory, LookupProof, SignedTreeHead, Witness,
    WitnessCosignature, verify_witness_cosignature,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::hex;
use crate::rate::RateLimiter;
use crate::store::now_unix_seconds;
use crate::username::{ClaimResult, UsernameRegistry, kt_username_label};

/// Shared key-transparency state: the hosted directory + the username registry, each behind an
/// async lock. The username registry is the first-claimant-wins guard; the directory is the
/// cryptographic log a claim is committed into and a lookup is proven against.
#[derive(Clone)]
pub struct KtState {
    pub dir: Arc<Mutex<KtDirectory>>,
    pub usernames: Arc<Mutex<UsernameRegistry>>,
    /// Per-IP flood limiter for the cheaper KT READS (each still generates an on-demand AKD proof).
    read_limiter: Arc<RateLimiter>,
    /// Strict per-IP flood limiter for `POST /username/claim` — each fresh claim verifies an
    /// Ed25519 proof-of-possession AND advances an AKD epoch + signs a new tree head (and holds
    /// the registry mutex across that publish), so it is far more expensive than a read.
    claim_limiter: Arc<RateLimiter>,
    /// Pinned witness co-signing allow-list (operator-tagged). The INDEX into this slice is the
    /// canonical `witness_index` clients also pin out-of-band. EMPTY = witnessing disabled: the
    /// witnessed-head endpoint still serves the head + log signature, just with NO cosignatures, and
    /// every cosignature submission is refused. Real split-view resistance requires these witnesses
    /// to actually be DEPLOYED and submitting — the relay only provides the rails.
    witnesses: Arc<Vec<Witness>>,
    /// Pinned AUDITOR co-signing key (a designated append-only auditor, distinct from the witnesses).
    /// `mercury_kt::kt_witness_status` requires an auditor signature for ANY non-Invalid verdict, so
    /// without this the client quorum gate can never pass — this is the structural keystone. None = no
    /// auditor configured (the witnessed-head endpoint serves no auditor signature; `/kt/auditor/cosign`
    /// is refused). Like witnesses, a REAL auditor must be DEPLOYED and actually checking append-only
    /// consistency before it signs — the relay only provides the rail.
    auditor: Arc<Option<Witness>>,
    /// The current PUBLISHED signed tree head: held STABLE per epoch (a fixed timestamp, refreshed at
    /// most every `KT_STH_REFRESH_S`) so independent witnesses all co-sign the SAME canonical bytes.
    /// Carries its log signature + the valid cosignatures collected for exactly those bytes.
    /// (Re)published lazily by the witnessed-head read path; submissions attach to it. In-memory only
    /// — cosignatures are ephemeral by nature (witnesses re-cosign after any restart or refresh).
    published: Arc<Mutex<Option<PublishedHead>>>,
    /// Tight per-IP limiter for witness cosignature submissions.
    witness_limiter: Arc<RateLimiter>,
}

/// The relay's current published head + its log signature + the witness cosignatures gathered for
/// EXACTLY these bytes. Replaced when the epoch advances or the head is refreshed.
struct PublishedHead {
    sth: SignedTreeHead,
    log_signature: [u8; 64],
    cosignatures: Vec<WitnessCosignature>,
    /// The pinned auditor's Ed25519 signature over EXACTLY these head bytes, once submitted. Reset to
    /// None when the head is replaced/refreshed, like the witness cosignatures.
    auditor_signature: Option<[u8; 64]>,
}

/// Per-IP flood caps for the KT/username router. The router was previously merged into the app
/// WITHOUT any rate-limit layer (the open-endpoint limiter only covered /relay/submit etc.), so
/// every proof-generating read and every epoch-advancing claim was uncapped per source. Reads are
/// generous; claims are tight because each one advances the AKD log. Honest scope is the same as
/// `crate::rate`: this bounds a SINGLE-SOURCE flood, not a distributed one (upstream CDN/proxy).
const KT_READ_MAX_PER_WINDOW: u32 = 120;
const KT_READ_WINDOW_S: i64 = 60;
const KT_CLAIM_MAX_PER_WINDOW: u32 = 8;
const KT_CLAIM_WINDOW_S: i64 = 60;
/// Witness cosignature submissions: a witness re-cosigns only when the head changes / refreshes, so
/// a modest cap (with headroom for retries + a few witnesses behind one proxy IP) is plenty.
const KT_WITNESS_MAX_PER_WINDOW: u32 = 30;
const KT_WITNESS_WINDOW_S: i64 = 60;
/// The published head is re-signed (fresh timestamp, cosignatures reset) at most this often even if
/// the epoch has not advanced, so freshness does not decay indefinitely on a low-churn directory
/// while still giving witnesses a stable window to co-sign a common head.
const KT_STH_REFRESH_S: i64 = 3600;

/// Maximum epoch span a single `/kt/consistency` request may cover. The only legitimate caller — the
/// witness — requests a SINGLE-epoch advance (`to == from + 1`); clients verify bundled proofs and
/// never call this endpoint with a wide range. Without a cap, an attacker-chosen `from=0&to=current`
/// forces an append-only proof over the ENTIRE log history — a cheap-request / expensive-response
/// amplifier that grows with the directory's age. 64 leaves generous headroom over the single-epoch
/// use while bounding the per-request proof cost regardless of how large the log grows.
const MAX_CONSISTENCY_SPAN: u64 = 64;

/// Whether a `/kt/consistency` request's epoch span is servable: `to >= from` (via `checked_sub`, so
/// an inverted `from > to` is refused without underflow) AND the width is at most
/// [`MAX_CONSISTENCY_SPAN`]. Pure, so the bound is unit-testable without a live directory.
fn consistency_span_ok(from: u64, to: u64) -> bool {
    matches!(to.checked_sub(from), Some(span) if span <= MAX_CONSISTENCY_SPAN)
}

/// Body-size cap for the KT POST endpoints (`/username/claim`, `/kt/witness/cosign`,
/// `/kt/auditor/cosign`). Every legitimate body here is a few hundred bytes (a 32-byte key, a 64-byte
/// signature, a short username — all hex). Without this they fell back to axum's 2 MiB default,
/// letting a caller force the relay to buffer up to 2 MiB before the per-field length/hex guards run.
/// 16 KiB is generous for the real bodies and mirrors the explicit limits the open `/relay/submit` +
/// `/directory/publish` routes already carry.
const KT_BODY_LIMIT: usize = 16 * 1024;

impl KtState {
    /// Host an existing directory with a fresh, empty username registry.
    pub fn new(dir: KtDirectory) -> Self {
        Self::with_registry(dir, UsernameRegistry::new())
    }

    /// Host an existing directory with a PRE-BUILT username registry — the boot path for a
    /// durable deployment: the server binary loads persisted bindings into the registry (see
    /// [`UsernameRegistry::with_durable`]) and re-registers them into `dir` in one epoch
    /// before serving, so restored handles are immediately provable.
    pub fn with_registry(dir: KtDirectory, usernames: UsernameRegistry) -> Self {
        Self {
            dir: Arc::new(Mutex::new(dir)),
            usernames: Arc::new(Mutex::new(usernames)),
            read_limiter: Arc::new(RateLimiter::new(KT_READ_MAX_PER_WINDOW, KT_READ_WINDOW_S)),
            claim_limiter: Arc::new(RateLimiter::new(KT_CLAIM_MAX_PER_WINDOW, KT_CLAIM_WINDOW_S)),
            witnesses: Arc::new(Vec::new()),
            auditor: Arc::new(None),
            published: Arc::new(Mutex::new(None)),
            witness_limiter: Arc::new(RateLimiter::new(
                KT_WITNESS_MAX_PER_WINDOW,
                KT_WITNESS_WINDOW_S,
            )),
        }
    }

    /// Configure the pinned witness co-signing allow-list (the boot path parses MERCURY_KT_WITNESSES
    /// into this). The slice ORDER is the canonical `witness_index` clients must also pin out-of-band.
    /// Replaces any existing set.
    pub fn with_witnesses(mut self, witnesses: Vec<Witness>) -> Self {
        self.witnesses = Arc::new(witnesses);
        self
    }

    /// Configure the pinned AUDITOR co-signing key (the boot path parses MERCURY_KT_AUDITOR into this).
    /// Clients pin the same key out-of-band. Without an auditor the quorum gate stays Invalid.
    pub fn with_auditor(mut self, auditor: Witness) -> Self {
        self.auditor = Arc::new(Some(auditor));
        self
    }
}

/// Flood limiter for the KT read endpoints — keyed by source IP via the shared
/// [`crate::http::client_key`] (so the documented behind-Caddy `X-Forwarded-For` handling applies
/// equally). Returns 429 over the cap, before any AKD proof is generated.
async fn kt_read_rate_limit(State(state): State<KtState>, request: Request, next: Next) -> Response {
    if !state.read_limiter.check(&crate::http::client_key(&request)) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(request).await
}

/// Strict flood limiter for `POST /username/claim` — rejects (429) before the Ed25519 verify and
/// the epoch-advancing AKD publish, so a source cannot drive unbounded epoch growth or head-of-line
/// block legitimate claimants behind the registry mutex.
async fn kt_claim_rate_limit(
    State(state): State<KtState>,
    request: Request,
    next: Next,
) -> Response {
    if !state.claim_limiter.check(&crate::http::client_key(&request)) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(request).await
}

/// Tight flood limiter for `POST /kt/witness/cosign` — each submission verifies an Ed25519
/// cosignature against the published head, so cap it per source like the claim path.
async fn kt_witness_rate_limit(
    State(state): State<KtState>,
    request: Request,
    next: Next,
) -> Response {
    if !state.witness_limiter.check(&crate::http::client_key(&request)) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(request).await
}

/// A served inclusion proof + the checkpoint it is relative to. The client
/// reconstructs the checkpoint as `mercury_kt::EpochHash(epoch, root_hash)` and
/// verifies `proof` against its PINNED VRF key via `mercury_kt::verify_inclusion`
/// -- the relay's serving of this is not itself trusted.
#[derive(Debug, Serialize, Deserialize)]
pub struct InclusionResponse {
    /// Directory epoch the proof is anchored to.
    pub epoch: u64,
    /// Hex-encoded 32-byte AKD root hash at that epoch. A client MUST verify this
    /// decodes to exactly 32 bytes before reconstructing the checkpoint (it comes
    /// from the untrusted relay; a wrong length must be rejected, never truncated
    /// or copied into a fixed-size buffer unchecked).
    pub root_hash: String,
    /// The AKD inclusion (lookup) proof.
    pub proof: LookupProof,
}

/// A served append-only (consistency) proof linking two epochs. The client
/// verifies it against the per-epoch root checkpoints it has PINNED via
/// `mercury_kt::verify_consistency` (the relay does not supply those roots).
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsistencyResponse {
    /// First epoch of the audited transition.
    pub from_epoch: u64,
    /// Last epoch of the audited transition.
    pub to_epoch: u64,
    /// The AKD append-only proof.
    pub proof: AppendOnlyProof,
}

/// A served key-history (rotation chain) proof + the checkpoint it is anchored
/// to. Verified client-side via `mercury_kt::verify_key_history` against the
/// pinned VRF key. (See [`InclusionResponse::root_hash`] re: length-checking.)
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyHistoryResponse {
    /// Directory epoch the proof is anchored to.
    pub epoch: u64,
    /// Hex-encoded 32-byte AKD root hash at that epoch.
    pub root_hash: String,
    /// The AKD key-history proof.
    pub proof: HistoryProof,
}

/// The directory's current LOG-signed tree head. The client reconstructs the
/// `SignedTreeHead` and verifies `log_signature` against the log key it PINNED
/// out-of-band (`mercury_kt::verify_signed_tree_head`); the relay never serves
/// that key. See [`InclusionResponse::root_hash`] re: length-checking the hex.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTreeHeadResponse {
    /// Directory size / epoch this head commits to.
    pub tree_size: u64,
    /// Hex-encoded 32-byte AKD root hash at that epoch.
    pub root_hash: String,
    /// Unix seconds the log signed this head (drives freshness).
    pub timestamp_s: i64,
    /// Hex-encoded 64-byte Ed25519 log signature over the canonical head bytes.
    pub log_signature: String,
}

/// `GET /kt/sth/witnessed` body: the relay's current PUBLISHED head + its log signature + the
/// witness cosignatures collected for EXACTLY these bytes, plus the relay's pinned witness set (for
/// transparency — a client still verifies cosignatures against the witnesses IT pinned out-of-band,
/// by `witness_index`). The client reconstructs the `SignedTreeHead`, then calls
/// `mercury_kt::verify_witnessed_tree_head` with its pinned log/witness keys and the head its own
/// proofs are bound to. Serving this is NOT itself trusted: the relay cannot forge a cosignature for
/// a witness key it does not hold, and a head that does not match the client's proof head is
/// rejected by the verifier.
#[derive(Debug, Serialize, Deserialize)]
pub struct WitnessedSthResponse {
    pub tree_size: u64,
    /// Hex-encoded 32-byte AKD root hash (length-check before use).
    pub root_hash: String,
    pub timestamp_s: i64,
    /// Hex-encoded 64-byte Ed25519 LOG signature over the canonical head bytes.
    pub log_signature: String,
    /// The witness cosignatures gathered for this exact head.
    pub cosignatures: Vec<WitnessCosignatureJson>,
    /// The relay's pinned witness allow-list (operator id + hex public key) in `witness_index`
    /// order. Informational: clients verify against their OWN pinned set.
    pub witnesses: Vec<WitnessJson>,
    /// The pinned AUDITOR's hex Ed25519 signature over this head, if submitted. None until the auditor
    /// cosigns — `mercury_kt::kt_witness_status` requires it for any non-Invalid verdict.
    pub auditor_signature: Option<String>,
    /// The relay's pinned auditor public key (hex), if configured. Informational; clients pin their own.
    pub auditor_public_key: Option<String>,
}

/// One witness cosignature in a [`WitnessedSthResponse`].
#[derive(Debug, Serialize, Deserialize)]
pub struct WitnessCosignatureJson {
    /// Index into the pinned witness allow-list (whose key signed).
    pub witness_index: usize,
    /// Hex-encoded 64-byte Ed25519 cosignature over the head's canonical bytes.
    pub signature: String,
}

/// One pinned witness in a [`WitnessedSthResponse`] (transparency only).
#[derive(Debug, Serialize, Deserialize)]
pub struct WitnessJson {
    pub operator_id: u32,
    /// Hex-encoded 32-byte Ed25519 witness public key.
    pub public_key: String,
}

/// `POST /kt/witness/cosign` request: a witness submits its cosignature over the relay's CURRENT
/// published head. The relay verifies the `(tree_size, root_hash, timestamp_s)` match the head it
/// published AND that `signature` is the pinned witness at `witness_index`'s genuine Ed25519
/// signature over the canonical bytes, before storing it. A mismatched / stale head is rejected
/// (409) so the witness re-fetches `GET /kt/sth/witnessed` and re-cosigns the current bytes.
#[derive(Debug, Deserialize)]
pub struct WitnessCosignRequest {
    pub tree_size: u64,
    /// Hex-encoded 32-byte root hash of the head being cosigned.
    pub root_hash: String,
    pub timestamp_s: i64,
    /// Index into the relay's pinned witness allow-list.
    pub witness_index: usize,
    /// Hex-encoded 64-byte Ed25519 cosignature over the head's canonical bytes.
    pub signature: String,
}

/// `POST /kt/auditor/cosign` request: the pinned AUDITOR submits its signature over the relay's
/// CURRENT published head (the same canonical bytes witnesses sign). Verified fail-closed against the
/// pinned auditor key + the exact published head; a stale/mismatched head is rejected (409).
#[derive(Debug, Deserialize)]
pub struct AuditorCosignRequest {
    pub tree_size: u64,
    /// Hex-encoded 32-byte root hash of the head being cosigned.
    pub root_hash: String,
    pub timestamp_s: i64,
    /// Hex-encoded 64-byte Ed25519 auditor signature over the head's canonical bytes.
    pub signature: String,
}

#[derive(Debug, Deserialize)]
struct ConsistencyParams {
    from: u64,
    to: u64,
}

/// `POST /username/claim` request. The proof of possession (`pop_sig` by `identity_pub` over the
/// NORMALIZED username) proves the claimer owns the account id the handle is bound to. The relay
/// re-normalizes and re-verifies; it never trusts the client's normalization.
#[derive(Debug, Deserialize)]
pub struct ClaimUsernameRequest {
    /// The claimer's Ed25519 identity public key (hex, 32 bytes). The bound account id is derived
    /// from it, so a claimer can only ever bind a handle to its OWN account.
    pub identity_pub: String,
    /// The desired handle (any case / surrounding space; the relay normalizes it).
    pub username: String,
    /// Username-claim proof of possession (hex, 64 bytes): `mercury_keys::sign_username_claim`.
    pub pop_sig: String,
}

/// `POST /username/claim` success body: the canonical handle that was registered.
#[derive(Debug, Serialize)]
pub struct ClaimUsernameResponse {
    pub username: String,
    /// True iff this call created a brand-new binding; false if it idempotently re-confirmed an
    /// existing one the same account already owned.
    pub created: bool,
}

/// `GET /username/{name}` success body: the resolved account id plus the KT inclusion proof the
/// client verifies against its pinned VRF key. The client reconstructs the checkpoint as
/// `mercury_kt::EpochHash(epoch, root_hash)`, calls `mercury_kt::verify_inclusion` with the label
/// `username:<name>` and `expected_key = account_id`, and only then trusts the binding.
#[derive(Debug, Serialize, Deserialize)]
pub struct UsernameLookupResponse {
    /// Hex-encoded 32-byte account id the handle resolves to. A client MUST verify this against the
    /// inclusion proof (a relay that lies here produces a proof that fails under the pinned key).
    pub account_id: String,
    /// Directory epoch the proof is anchored to.
    pub epoch: u64,
    /// Hex-encoded 32-byte AKD root hash at that epoch (length-check before use).
    pub root_hash: String,
    /// The AKD inclusion (lookup) proof for label `username:<name>`.
    pub proof: LookupProof,
}

/// `GET /kt/vrf-key` body: the directory's pinnable public keys, for trust-on-first-use bootstrap.
/// See the module docs for the honest bound (out-of-band pinning + witnesses close the residual).
#[derive(Debug, Serialize, Deserialize)]
pub struct VrfKeyResponse {
    /// Hex-encoded VRF public key — what `mercury_kt::verify_inclusion` checks proofs against.
    pub vrf_public_key: String,
    /// Hex-encoded 32-byte Ed25519 log public key — what signed tree heads are verified against.
    pub log_public_key: String,
}

/// `GET /healthz` body: an operational health signal distinct from the KT protocol endpoints. Reports
/// the current KT epoch + witness/auditor FRESHNESS so a monitor can alert when cosigning silently
/// decays — the failure mode that quietly defeats split-view detection (a quorum that stopped
/// cosigning is otherwise invisible until a client rejects a head). Observability, not enforcement.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    /// The directory's current epoch / tree size (None only on a genuine directory error).
    pub kt_epoch: Option<u64>,
    /// How many witnesses are pinned in the allow-list.
    pub witnesses_configured: usize,
    /// Whether an auditor key is configured.
    pub auditor_configured: bool,
    /// Witness cosignatures gathered for the current published head (0 if none / no head published).
    pub witness_cosig_count: usize,
    /// Whether the auditor has cosigned the current published head.
    pub auditor_present: bool,
    /// Age in seconds of the current published head (None if none published yet). A large/growing
    /// value while witnesses are configured = freshness decay worth alerting on.
    pub witness_head_age_s: Option<i64>,
}

/// `GET /healthz` — see [`HealthResponse`]. Unauthenticated + unrate-limited (a liveness/health probe
/// is polled frequently); cheap (one tree-head sign + two lock reads).
async fn healthz(State(state): State<KtState>) -> Response {
    let now = now_unix_seconds();
    let kt_epoch = {
        let dir = state.dir.lock().await;
        dir.signed_tree_head(now)
            .await
            .ok()
            .map(|(sth, _)| sth.tree_size)
    };
    let (witness_cosig_count, auditor_present, witness_head_age_s) =
        match state.published.lock().await.as_ref() {
            Some(ph) => (
                ph.cosignatures.len(),
                ph.auditor_signature.is_some(),
                Some(now.saturating_sub(ph.sth.timestamp_s)),
            ),
            None => (0, false, None),
        };
    Json(HealthResponse {
        status: "ok".to_string(),
        kt_epoch,
        witnesses_configured: state.witnesses.len(),
        auditor_configured: state.auditor.as_ref().is_some(),
        witness_cosig_count,
        auditor_present,
        witness_head_age_s,
    })
    .into_response()
}

/// Build the KT directory-service router.
///
/// - `GET /kt/inclusion/{label}` -> [`InclusionResponse`] (404 if absent).
/// - `GET /kt/consistency?from=&to=` -> [`ConsistencyResponse`] (400 on a bad
///   epoch range).
/// - `GET /kt/history/{label}` -> [`KeyHistoryResponse`] (404 if absent).
/// - `GET /kt/sth` -> [`SignedTreeHeadResponse`] (a fresh directory serves a
///   valid epoch-0 head; 404 only on a genuine directory error).
pub fn kt_router(state: KtState) -> Router {
    // Per-IP flood limiters (previously the KT router carried NONE). Reads share one cap; the
    // epoch-advancing claim + the cosignature-verifying witness submission each get a tighter one.
    // Applied OUTSIDE the handler so a flood is rejected (429) before any AKD proof generation /
    // Ed25519 verify / epoch advance.
    let read_limited = middleware::from_fn_with_state(state.clone(), kt_read_rate_limit);
    let claim_limited = middleware::from_fn_with_state(state.clone(), kt_claim_rate_limit);
    let witness_limited = middleware::from_fn_with_state(state.clone(), kt_witness_rate_limit);
    Router::new()
        .route("/kt/inclusion/{label}", get(inclusion).layer(read_limited.clone()))
        .route("/kt/consistency", get(consistency).layer(read_limited.clone()))
        .route("/kt/history/{label}", get(key_history).layer(read_limited.clone()))
        .route("/kt/sth", get(signed_tree_head).layer(read_limited.clone()))
        .route("/kt/sth/witnessed", get(witnessed_sth).layer(read_limited.clone()))
        .route("/kt/vrf-key", get(vrf_key).layer(read_limited.clone()))
        .route(
            "/kt/witness/cosign",
            post(witness_cosign)
                .layer(DefaultBodyLimit::max(KT_BODY_LIMIT))
                .layer(witness_limited.clone()),
        )
        .route(
            "/kt/auditor/cosign",
            post(auditor_cosign)
                .layer(DefaultBodyLimit::max(KT_BODY_LIMIT))
                .layer(witness_limited),
        )
        .route(
            "/username/claim",
            post(username_claim)
                .layer(DefaultBodyLimit::max(KT_BODY_LIMIT))
                .layer(claim_limited),
        )
        .route("/username/{name}", get(username_lookup).layer(read_limited))
        // Operational health probe (no rate limit — monitors poll it frequently; it is cheap).
        .route("/healthz", get(healthz))
        .with_state(state)
}

async fn inclusion(State(state): State<KtState>, Path(label): Path<String>) -> Response {
    let dir = state.dir.lock().await;
    match dir.prove_inclusion(&label).await {
        Ok((proof, checkpoint)) => Json(InclusionResponse {
            epoch: checkpoint.epoch(),
            root_hash: hex::encode(&checkpoint.hash()),
            proof,
        })
        .into_response(),
        // A label the directory has never published has no proof. The relay
        // returns an opaque 404 and nothing else (no plaintext, no metadata).
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn consistency(
    State(state): State<KtState>,
    Query(params): Query<ConsistencyParams>,
) -> Response {
    // Cap the served epoch span BEFORE any proof work (see [`MAX_CONSISTENCY_SPAN`]): refuse an overly
    // wide or inverted range with an opaque 400, so a malicious client cannot force a full-history
    // append-only proof. The legitimate witness uses a single-epoch span, well within the cap.
    if !consistency_span_ok(params.from, params.to) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let dir = state.dir.lock().await;
    match dir.prove_consistency(params.from, params.to).await {
        Ok(proof) => Json(ConsistencyResponse {
            from_epoch: params.from,
            to_epoch: params.to,
            proof,
        })
        .into_response(),
        // An invalid epoch range (e.g. from >= to, or beyond the current epoch)
        // is a bad request -> opaque 400.
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

async fn key_history(State(state): State<KtState>, Path(label): Path<String>) -> Response {
    let dir = state.dir.lock().await;
    match dir.prove_key_history(&label).await {
        Ok((proof, checkpoint)) => Json(KeyHistoryResponse {
            epoch: checkpoint.epoch(),
            root_hash: hex::encode(&checkpoint.hash()),
            proof,
        })
        .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn signed_tree_head(State(state): State<KtState>) -> Response {
    let dir = state.dir.lock().await;
    match dir.signed_tree_head(now_unix_seconds()).await {
        Ok((sth, signature)) => Json(SignedTreeHeadResponse {
            tree_size: sth.tree_size,
            root_hash: hex::encode(&sth.root_hash),
            timestamp_s: sth.timestamp_s,
            log_signature: hex::encode(&signature),
        })
        .into_response(),
        // A fresh directory has a valid epoch-0 head, so this only fires on a
        // genuine directory/storage error -> opaque 404.
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `GET /kt/sth/witnessed` — serve the relay's current PUBLISHED head (held stable per epoch so
/// independent witnesses co-sign a COMMON head), its log signature, and the cosignatures gathered
/// for exactly those bytes. (Re)publishes a fresh head — resetting cosignatures — when the epoch has
/// advanced or the head has aged past `KT_STH_REFRESH_S`. Serving this confers no trust: a client
/// verifies the log signature + each cosignature against the keys IT pinned and binds the head to
/// its own proofs (`mercury_kt::verify_witnessed_tree_head`).
async fn witnessed_sth(State(state): State<KtState>) -> Response {
    let now = now_unix_seconds();
    let dir = state.dir.lock().await;
    let (cur_sth, cur_sig) = match dir.signed_tree_head(now).await {
        Ok(v) => v,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    // `dir` is held across the `published` lock: this is the ONLY path that takes both, always in
    // this order (the submission path takes only `published`), so no lock cycle is possible.
    let mut published = state.published.lock().await;
    let stale = match published.as_ref() {
        None => true,
        Some(ph) => {
            ph.sth.tree_size != cur_sth.tree_size
                || ph.sth.root_hash != cur_sth.root_hash
                || now.saturating_sub(ph.sth.timestamp_s) > KT_STH_REFRESH_S
        }
    };
    if stale {
        // Publish the current head with a fresh timestamp + reset cosignatures (the prior head's
        // cosignatures are over different bytes; witnesses re-cosign the new head).
        *published = Some(PublishedHead {
            sth: cur_sth,
            log_signature: cur_sig,
            cosignatures: Vec::new(),
            auditor_signature: None,
        });
    }
    let ph = published.as_ref().expect("published head just ensured present");
    Json(WitnessedSthResponse {
        tree_size: ph.sth.tree_size,
        root_hash: hex::encode(&ph.sth.root_hash),
        timestamp_s: ph.sth.timestamp_s,
        log_signature: hex::encode(&ph.log_signature),
        cosignatures: ph
            .cosignatures
            .iter()
            .map(|c| WitnessCosignatureJson {
                witness_index: c.witness_index,
                signature: hex::encode(&c.signature),
            })
            .collect(),
        witnesses: state
            .witnesses
            .iter()
            .map(|w| WitnessJson {
                operator_id: w.operator_id,
                public_key: hex::encode(&w.key_bytes()),
            })
            .collect(),
        auditor_signature: ph.auditor_signature.as_ref().map(|s| hex::encode(s)),
        auditor_public_key: state
            .auditor
            .as_ref()
            .as_ref()
            .map(|a| hex::encode(&a.key_bytes())),
    })
    .into_response()
}

/// `POST /kt/witness/cosign` — a pinned witness submits its cosignature over the relay's CURRENT
/// published head. Fail-closed: the submission must name a configured witness index, match the
/// published head EXACTLY (tree_size + root_hash + timestamp), and carry that witness's genuine
/// Ed25519 signature over the canonical bytes. Anything else is rejected and nothing is stored.
async fn witness_cosign(
    State(state): State<KtState>,
    Json(req): Json<WitnessCosignRequest>,
) -> Response {
    // Decode the head root + signature; a bad length/hex is a 400 (never trusted into a fixed buffer).
    let Some(root_hash) = hex::decode(&req.root_hash)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(signature) = hex::decode(&req.signature)
        .ok()
        .and_then(|b| <[u8; 64]>::try_from(b.as_slice()).ok())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    // The witness must be in the pinned allow-list at the claimed index.
    let Some(witness) = state.witnesses.get(req.witness_index) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut published = state.published.lock().await;
    let Some(ph) = published.as_mut() else {
        // No head published yet — the witness must GET /kt/sth/witnessed first.
        return StatusCode::CONFLICT.into_response();
    };
    // The cosignature must be over the EXACT currently-published head, else it is over different
    // bytes than clients will fetch -> useless. Tell the witness to re-fetch + re-cosign.
    if ph.sth.tree_size != req.tree_size
        || ph.sth.root_hash != root_hash
        || ph.sth.timestamp_s != req.timestamp_s
    {
        return StatusCode::CONFLICT.into_response();
    }
    // Verify it really is this witness's signature over the head's canonical bytes (verify_strict).
    if !verify_witness_cosignature(witness, &ph.sth, &signature) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    // Store, de-duplicating by witness_index (a re-submission by the same witness replaces).
    ph.cosignatures.retain(|c| c.witness_index != req.witness_index);
    ph.cosignatures.push(WitnessCosignature {
        witness_index: req.witness_index,
        signature,
    });
    StatusCode::OK.into_response()
}

/// `POST /kt/auditor/cosign` — the pinned AUDITOR submits its signature over the relay's CURRENT
/// published head (the same canonical bytes witnesses sign). Fail-closed: requires a configured
/// auditor, an EXACT match of the published head, and the auditor's genuine Ed25519 signature; a
/// stale/mismatched head is rejected (409) so the auditor re-fetches and re-signs.
async fn auditor_cosign(
    State(state): State<KtState>,
    Json(req): Json<AuditorCosignRequest>,
) -> Response {
    let Some(root_hash) = hex::decode(&req.root_hash)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(signature) = hex::decode(&req.signature)
        .ok()
        .and_then(|b| <[u8; 64]>::try_from(b.as_slice()).ok())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    // An auditor must be configured for the relay to accept its signature.
    let Some(auditor) = state.auditor.as_ref().as_ref() else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut published = state.published.lock().await;
    let Some(ph) = published.as_mut() else {
        // No head published yet — the auditor must GET /kt/sth/witnessed first.
        return StatusCode::CONFLICT.into_response();
    };
    if ph.sth.tree_size != req.tree_size
        || ph.sth.root_hash != root_hash
        || ph.sth.timestamp_s != req.timestamp_s
    {
        return StatusCode::CONFLICT.into_response();
    }
    // The auditor signs the SAME canonical head bytes as witnesses; reuse the strict verifier.
    if !verify_witness_cosignature(auditor, &ph.sth, &signature) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    ph.auditor_signature = Some(signature);
    StatusCode::OK.into_response()
}

/// `GET /kt/vrf-key` — serve the directory's VRF + log public keys for trust-on-first-use bootstrap.
/// A client pins these on first contact and verifies every later proof / signed head against the
/// pinned copy, detecting a relay that subsequently rotates its keys. The STRONG model pins these
/// out-of-band (see `docs/USERNAMES.md`); serving them is a documented convenience, not a trust
/// claim. Public key material only — it never lets the relay decrypt anything.
async fn vrf_key(State(state): State<KtState>) -> Response {
    let dir = state.dir.lock().await;
    match dir.public_key().await {
        Ok(vrf) => Json(VrfKeyResponse {
            vrf_public_key: hex::encode(&vrf),
            log_public_key: hex::encode(&dir.log_public_key()),
        })
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `POST /username/claim` — bind a handle to the claimer's account id, first-claimant-wins, and
/// commit it to the KT log so later lookups carry an inclusion proof.
///
/// The first-claimant-wins decision and the KT publish run under the SAME held registry lock, so two
/// concurrent claims for one handle cannot both win, and the KT log is written BEFORE the guard is
/// committed (a KT failure leaves the guard untouched — no orphaned binding). Fails closed: 400 on a
/// malformed request or illegal handle, 403 on a bad proof, 409 if the handle is taken by another
/// account, 503 if the registry is full, the KT publish fails, or the durable write-through fails.
async fn username_claim(
    State(state): State<KtState>,
    Json(req): Json<ClaimUsernameRequest>,
) -> Response {
    // Normalize first; an illegal handle is a 400 before any crypto.
    let Some(name) = mercury_keys::normalize_username(&req.username) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    // Decode + width-check the key and signature.
    let parsed = (|| {
        let identity_pub: [u8; 32] = hex::decode(&req.identity_pub)
            .ok()?
            .as_slice()
            .try_into()
            .ok()?;
        let pop_sig: [u8; 64] = hex::decode(&req.pop_sig).ok()?.as_slice().try_into().ok()?;
        Some((identity_pub, pop_sig))
    })();
    let Some((identity_pub, pop_sig)) = parsed else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    // Authorize: a valid claim proof yields the account id derived from the proving key. A `None`
    // is an unauthorized claim -> 403.
    let Some(account_id) = mercury_keys::verify_username_claim(&identity_pub, &name, &pop_sig)
    else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Hold the registry lock across the whole decision + KT publish so first-claimant-wins is atomic.
    let mut users = state.usernames.lock().await;
    match users.lookup(&name) {
        Some(existing) if existing == account_id => {
            // Idempotent: this account already owns the handle. No KT re-publish.
            Json(ClaimUsernameResponse {
                username: name,
                created: false,
            })
            .into_response()
        }
        Some(_) => StatusCode::CONFLICT.into_response(),
        None => {
            if users.len() >= crate::username::MAX_USERNAMES {
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
            // Advance the KT log AND persist the new epoch atomically — both under the dir lock, so
            // a concurrent reader never sees (and a client never pins) a tree head at an epoch that
            // is not yet durable. Snapshotting BEFORE the durable username put below keeps the
            // on-disk AKD >= the username store, so the only crash residual is a tolerated orphan KT
            // label (handled below + reconciled-around on boot), never a username binding the
            // restored AKD lacks. snapshot() is a no-op unless MERCURY_KT_SNAPSHOT enabled persistence.
            {
                let mut dir = state.dir.lock().await;
                if dir
                    .register(&kt_username_label(&name), &account_id)
                    .await
                    .is_err()
                {
                    return StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
                if let Err(e) = dir.snapshot().await {
                    // The epoch advanced in memory but did NOT reach disk. Do not make the binding
                    // durable in the username store (that would expose an epoch a restart would
                    // lose); fail closed with a retryable 503. On restart the prior snapshot is
                    // restored, so the prior epoch is the basis again and a retry re-registers
                    // cleanly; the un-persisted in-memory epoch is discarded on the next boot.
                    eprintln!("mercury-relay: KT snapshot after register failed: {e}");
                    return StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
            }
            // Now commit the first-claimant-wins guard (durably, when a store is attached).
            match users.claim(&name, account_id) {
                ClaimResult::Created => Json(ClaimUsernameResponse {
                    username: name,
                    created: true,
                })
                .into_response(),
                // The durable write-through failed AFTER the KT register above succeeded, so the
                // log now holds a label with NO registry binding. That orphan is honest and
                // harmless: lookups go through the registry guard first, so an orphaned KT label
                // is unreachable, and the handle stays claimable (a retry by anyone re-registers
                // the label — AKD supports re-publishing a label as a new version). What we must
                // NOT do is insert into the in-memory map without the disk commit (a restart would
                // silently revoke the claim) — hence fail closed with a retryable 503.
                ClaimResult::PersistFailed => StatusCode::SERVICE_UNAVAILABLE.into_response(),
                // We held the lock since the `None` check, so only the arms above are reachable;
                // treat anything else as a fail-closed server error rather than a silent success.
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
}

/// `GET /username/{name}` — resolve a handle to its account id plus a KT inclusion proof. Returns
/// 404 if the handle is unregistered or malformed; the client verifies the proof against its pinned
/// VRF key before trusting the binding.
async fn username_lookup(State(state): State<KtState>, Path(name): Path<String>) -> Response {
    let Some(name) = mercury_keys::normalize_username(&name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // Read the binding, then release the registry lock before proving (the proof is read-only).
    let account_id = {
        let users = state.usernames.lock().await;
        users.lookup(&name)
    };
    let Some(account_id) = account_id else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let dir = state.dir.lock().await;
    match dir.prove_inclusion(&kt_username_label(&name)).await {
        Ok((proof, checkpoint)) => Json(UsernameLookupResponse {
            account_id: hex::encode(&account_id),
            epoch: checkpoint.epoch(),
            root_hash: hex::encode(&checkpoint.hash()),
            proof,
        })
        .into_response(),
        // The guard says the handle is registered, so a proof failure here is a genuine directory
        // error, not an absent label -> opaque 404 (fail-closed; the client simply can't resolve it).
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistency_span_is_capped_and_rejects_inverted_ranges() {
        // The witness's single-epoch advance is always servable.
        assert!(consistency_span_ok(0, 1));
        assert!(consistency_span_ok(1_000, 1_001));
        // Up to and including the cap is servable...
        assert!(consistency_span_ok(100, 100 + MAX_CONSISTENCY_SPAN));
        // ...one past it is refused (the cheap-request / expensive-proof amplifier)...
        assert!(!consistency_span_ok(100, 100 + MAX_CONSISTENCY_SPAN + 1));
        // ...and the full-history span an attacker would request is refused outright.
        assert!(!consistency_span_ok(0, u64::MAX));
        // An inverted range (from > to) is refused without underflow.
        assert!(!consistency_span_ok(10, 9));
    }
}
