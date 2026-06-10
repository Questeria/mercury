# Helix decider coverage

What `mercury-core` decision logic is mirrored as a Helix policy today, what is not, and how the
two relate. This is a coverage map, not a claim of completeness — it states exactly what is pinned.

## The pattern

A Helix policy is a **pure-scalar mirror** of a `mercury-core` decision combinator. mercury-core's
`evaluate_*` functions take rich-typed inputs (structs, enums, sub-decisions); the Helix policy
consumes a **scalar reduction** of those inputs (small integer codes + boolean predicates) and
reproduces the reason + decision fields. The reduction itself lives in Rust at the boundary, and is
held faithful three ways, all in CI:

1. **Rust pin** (`core/rust/mercury-core/tests/<p>_vectors.rs`) — reconstructs the REAL input from
   each golden vector's scalars and asserts the REAL `evaluate_*` output equals the vector.
2. **Exhaustive differential** (`core/rust/mercury-core/tests/decider_exhaustive_diff.rs`) — for the
   deep deciders, re-derives the staged reduction in Rust and checks it against the REAL `evaluate_*`
   over the **entire** reduced input domain (every variant combination).
3. **Helix proof + drift gate** — the `.hx` compiles + runs (exit 42) under the pinned `helixc`, and
   `gen_<p>_vectors.py --check` fails CI if the vectors/test/manifest drift from the spec.

Important honesty note: the reductions are **faithful + pinned + exhaustively differentiated, but
they are scalar reductions, not the raw rich-typed structs**. Where a reduction collapses an integer
range to a predicate (e.g. `recovery_key_entropy_bits >= 192` → `entropy_ok`), the differential
tests the decision boundary representatives, not every integer — faithful because the real function's
behaviour depends only on the predicate.

## Mirrored as standalone pinned deciders (5 of 63 `evaluate_*`)

| `evaluate_*` (lib.rs) | Helix policy | reasons | exhaustive differential |
|---|---|---|---|
| `evaluate_outbound_send` (10841) | `outbound_decide` | 6 | 144 combos |
| `evaluate_client_receive` (17813) | `receive_decide` | 13 | 20 480 combos |
| `evaluate_client_bootstrap` (21528) | `bootstrap_decide` | 21 | 409 600 combos |
| `evaluate_inbound_sync` (30212) | `inbound_sync` | 9 | 256 combos |
| `evaluate_account_recovery` (21949) | `account_recovery` | 13 | 20 480 combos |

These are the client-lifecycle + outbound/inbound decision combinators. Each is pinned to its REAL
function and exhaustively differentiated.

## Policy-layer + view policies (7 more Helix policies)

These predate the decider mirrors and cover the policy-composition / capability-view surface rather
than a single `evaluate_*` each (their manifests cite the shared `vectors/` corpus + the
`gen_helix_tests.py` Rust↔Helix differential, not one combinator):

- `platform_decision` — mirrors the `PlatformDecisionView` projection (the capability view over the
  bootstrap / outbound / receive / policy decisions).
- `policy_pipeline` — the reason composition of `evaluate_policy` (30504); the full `evaluate_policy`
  is pinned in Rust by `core/rust/mercury-core/tests/core_policy_vectors.rs` against `vectors/core_policy/`.
- `envelope`, `room_epoch`, `ai_grant`, `ai_grant_lifecycle` — the component reason codes that feed
  the policy pipeline (envelope structure, room-epoch transition, AI-grant, AI-grant-lifecycle).
- `relay_submit` — the relay-submission policy surface behind `evaluate_relay_submission` (15190);
  the full combinator is pinned in Rust by `core/rust/mercury-core/tests/relay_submission.rs`.

Total: **12 Helix policies** (5 pinned deciders + 7 policy-layer/view).

## Not yet mirrored (~57 `evaluate_*`)

The remaining combinators are not yet mirrored as standalone pinned Helix deciders. None are blocked
— they are simply not done yet, and most would take the same staged-reduction treatment. The family
groupings below are approximate (some `evaluate_*` are exercised by mercury-core's own gate tests but
have no Helix policy); the authoritative coverage is `grep "pub fn evaluate_"` plus the per-policy
manifests and the `*_vectors.rs` pins. Grouped:

- **Sub-decisions consumed by the mirrored deciders (indirectly exercised, not independently pinned
  as a standalone Helix decider):** `evaluate_device_trust` (1676), `evaluate_key_transparency`
  (1824), `evaluate_room_membership_transition` (2431), `evaluate_delivery_ack` (17623),
  `evaluate_relay_queue` (15259). The deciders reduce these to scalars; mirroring them standalone
  would pin the sub-decisions themselves. (`evaluate_relay_submission` (15190) belongs with the
  `relay_submit` policy above, not here.)
- **MLS group-chat family (~15):** `evaluate_group_chat`, `evaluate_group_message_transcript`,
  `evaluate_group_relay_envelope`, and the `evaluate_mls_*` admission / store-write / replay
  combinators (provider security/adapter/evidence, key-package, welcome, commit, membership).
- **Sealed-audit family (~16):** the `evaluate_sealed_audit_*` chain / store-write / witness /
  proof / verifier-snapshot / incident / recovery-export / database-adapter / private-report
  combinators.
- **Local-store family (~9):** `evaluate_local_store_*` (write, write-request, sealing-request,
  open-request, unlock, production-open, keychain-unlock, database-security, database-adapter).
- **Media-object family (~3):** `evaluate_media_object_store`, `..._index_store_write`,
  `..._index_production_open`.
- **Anonymous-credential family (~6):** `evaluate_anonymous_*` (issuer-witness-audit, issuer-trust,
  group-membership-proof, rate-limit-nullifier, nullifier-store-write).
- **Misc:** `evaluate_ai_participant_action` (879), `evaluate_ai_connector` (1163),
  `evaluate_secure_backup_restore` (22374), `evaluate_authenticated_relay_source` (29971).
  (`evaluate_policy` (30504) is covered above by `policy_pipeline` + `core_policy_vectors.rs`.)

Some of these carry richer or larger state than the 6-int-param Helix codegen cap allows in one
function; those would need the same **staging** the receive / bootstrap / account_recovery deciders
use (stage-reason functions + a first-non-zero composer). A few that lean on byte content or opaque
non-scalar state would need a judgement call on whether a faithful scalar reduction exists at all.

## How to add another

1. Read the `evaluate_*` body; reduce its inputs to scalars (+ stage if >6 per function).
2. Write `tools/check_<p>_vectors.py` (the spec) and self-verify.
3. Write `helix/policy/<p>.hx`, `tools/gen_<p>_vectors.py` (vectors + test + manifest),
   `core/rust/mercury-core/tests/<p>_vectors.rs` (pin to the REAL fn), and extend
   `decider_exhaustive_diff.rs`.
4. Wire `tools/run_helix_checks.{sh,ps1}` + `.github/workflows/ci.yml`.
5. Verify (Helix exit 42 + Rust pin + differential + full suite) and audit.
