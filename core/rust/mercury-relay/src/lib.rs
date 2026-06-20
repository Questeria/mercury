//! Mercury delivery relay.
//!
//! A dumb ciphertext router. The relay never sees plaintext, stores only
//! opaque ciphertext + sealed headers under short TTLs, and enforces
//! content-free metadata. Every admission decision is delegated to the
//! authoritative `mercury-core` relay gates — this crate never re-implements
//! or bypasses the policy:
//!
//! - [`mercury_core::evaluate_relay_submission`] — submission contract
//! - [`mercury_core::evaluate_relay_queue`] — queue state machine
//!   (enqueue / deliver / expire / delete)
//! - [`mercury_core::evaluate_delivery_ack`] — delivery acknowledgement
//! - [`mercury_core::evaluate_authenticated_relay_source`] — transport-auth
//!   admission for poll / ack / delete
//!
//! The relay fails closed: any non-`accepted` gate decision becomes a 4xx and
//! nothing is persisted or returned beyond an opaque status + reason code.

#![forbid(unsafe_code)]

pub mod directory;
pub mod hex;
pub mod http;
pub mod kt;
pub mod pairing;
pub mod push;
pub mod rate;
pub mod redb_store;
pub mod store;
pub mod username;
pub mod username_store;

pub use directory::InMemoryDirectoryStore;
pub use http::{RelayState, router};
pub use kt::{
    ConsistencyResponse, InclusionResponse, KeyHistoryResponse, KtState, SignedTreeHeadResponse,
    UsernameLookupResponse, VrfKeyResponse, WitnessCosignRequest, WitnessCosignatureJson,
    WitnessJson, WitnessedSthResponse, kt_router,
};
pub use pairing::{PairingStore, generate_pairing_code};
pub use push::{InProcessWaker, NoopPushSender, PushSender};
pub use rate::RateLimiter;
pub use redb_store::RedbQueueStore;
pub use store::{InMemoryQueueStore, QueueRecord, QueueStore, spawn_sweeper};
pub use username::{ClaimResult, UsernameRegistry, kt_username_label};
pub use username_store::DurableUsernameStore;
