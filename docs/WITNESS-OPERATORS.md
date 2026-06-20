# Running a Mercury KT witness (or auditor)

This is the operator runbook for `mercury-kt-witness`. If you want to understand *why* witnessing
exists and what it does (and does not) guarantee, read [`WITNESSING.md`](WITNESSING.md) first. This
document is the *how*.

## Who should run this — and why it only helps if you are independent

A witness co-signs the relay's published key-transparency tree heads. A client that requires a
**quorum** of witness cosignatures will reject a forked head that no witness vouched for — that is the
defense against a relay showing different histories to different people (a *split view*).

The protection is worth **exactly** the independence of the witnesses. A witness run by the relay
operator, on the relay's own host, adds nothing — the same party that could equivocate also controls
the witness. So this is for **independent third parties**: run it on infrastructure you control,
separate from the relay operator, ideally one of several unaffiliated operators. The binary makes the
witness *honest* (it refuses to co-sign an inconsistent head); it cannot make it *independent*. That
part is on you.

`mercury_kt::kt_witness_status` requires **≥ 2 independent operators** plus an auditor signature
before it reports `QuorumSatisfied`, so a single witness — or several keys held by one party — is not
a quorum.

## What it does each run

`mercury-kt-witness` is **single-shot**: one round per invocation, so you schedule it on a timer.
Each run it:

1. Fetches the relay's current published head (`GET /kt/sth/witnessed`) and verifies its signature
   against the log key you pinned.
2. Compares it to the head it last co-signed (persisted in the `--state` file). For a single-epoch
   advance it fetches and verifies the append-only consistency proof binding *your* last root to the
   new one; a bigger gap, a rollback, a same-epoch fork, or a failed proof is **refused**.
3. Only if the head is a verified append-only extension: signs it and submits the cosignature
   (`POST /kt/witness/cosign`, or `/kt/auditor/cosign` with `--auditor`), then advances the state
   file — *after* the relay accepts it.

It **fails closed** everywhere: any network or parse error, or any inconsistency, exits non-zero
**without** co-signing.

## Setup

### 1. Build it

Either build the binary directly:

```
cargo build -p mercury-witness --bin mercury-kt-witness --release
# -> target/release/mercury-kt-witness
```

…or build the container image (build context is the repo root):

```
docker build -f deploy/witness/Dockerfile -t mercury-witness .
```

### 2. Generate your keypair

```
mercury-kt-witness keygen > witness.key
chmod 600 witness.key
```

The **signing key** is written to `witness.key` (stdout); the **public key** + guidance are printed
to stderr. Keep `witness.key` secret. Send the **public key** to the relay operator over a trusted,
out-of-band channel.

### 3. Get added to the relay's allow-list

The relay operator adds your public key to their configuration:

- as a **witness**: `MERCURY_KT_WITNESSES="0:<your-pubkey>,1:<other-operator-pubkey>,..."` — the
  position in that list is your `--index`, which you must pin (agree on it out-of-band); or
- as the **auditor**: `MERCURY_KT_AUDITOR="<your-pubkey>"` — then run with `--auditor` (no `--index`).

### 4. Pin the relay's log key (strongly recommended)

Obtain the relay's 64-hex Ed25519 **log public key** out-of-band and pass it as `--log-key`. Without
it the witness trusts-on-first-use the key the relay serves at `/kt/vrf-key` and prints a warning —
acceptable for a first bring-up, but a relay that controls that endpoint controls your root of trust,
which defeats the point.

## Run it on a timer

### systemd (recommended)

`/etc/systemd/system/mercury-witness.service`:

```ini
[Unit]
Description=Mercury KT witness — one co-sign round
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
User=mercury-witness
ExecStart=/usr/local/bin/mercury-kt-witness \
    --relay https://relay.example.com \
    --key-file /var/lib/mercury-witness/witness.key \
    --index 0 \
    --state /var/lib/mercury-witness/state.json \
    --log-key 0000000000000000000000000000000000000000000000000000000000000000
# Exit 3 (an equivocation signal) is recorded as a failure you can alert on; 4 (fell behind) and
# 70 (transient) are non-fatal to the timer.
SuccessExitStatus=0 4 70
```

`/etc/systemd/system/mercury-witness.timer`:

```ini
[Unit]
Description=Run the Mercury KT witness every 5 minutes

[Timer]
OnBootSec=2min
OnUnitActiveSec=5min
Persistent=true

[Install]
WantedBy=timers.target
```

```
systemctl enable --now mercury-witness.timer
```

### cron

```
*/5 * * * * mercury-kt-witness --relay https://relay.example.com --key-file ~/witness.key --index 0 --state ~/witness-state.json --log-key <64-hex> >> ~/witness.log 2>&1
```

## Exit codes — what to alert on

| Code | Meaning | Action |
|------|---------|--------|
| `0`  | Co-signed (or already up to date) | none |
| `3`  | **Equivocation signal** — a rollback, a same-epoch fork, an inconsistent extension, or a bad log signature | **PAGE.** The relay served a head inconsistent with what you vouched for. Do not delete state; investigate. |
| `4`  | Fell more than one epoch behind; the span cannot be verified | Investigate the gap, then delete the `--state` file to re-bootstrap from the current head |
| `64` | Usage error (bad flags) | Fix the command |
| `70` | Transient error (relay unreachable, parse failure) | Retry on the next tick; no action if isolated |

An exit `3` is the signal the whole system exists to produce. Wire it to your alerting.

## Honest residuals

- **Independence is operational, not code.** Run this on your own infrastructure, separate from the
  relay operator. The strongest deployments have several unaffiliated operators.
- **A quorum needs ≥ 2 independent operators + an auditor**, and the **client** must be configured to
  require that quorum (`kt_witness_status`) — a witnessed bundle that no client gates on changes
  nothing. See [`WITNESSING.md`](WITNESSING.md).
- **State is local.** The `--state` file is this witness's memory of what it last vouched for. Losing
  it means the next run trusts-on-first-use the then-current head (it cannot retroactively verify the
  gap). Keep it on durable storage; back it up if you care about continuity across host loss.
