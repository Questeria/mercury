//! Property tests for `policy_pipeline_decide_v1`.
//!
//! The pipeline first validates that each component reason code is within the
//! contract range, then composes a first-reject over the components. These
//! properties pin the contract-rejection behaviour and the human-actor AI
//! guard.

use mercury_policy::{
    POLICY_PIPELINE_ACCEPT, POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN,
    POLICY_PIPELINE_BAD_AI_GRANT_REASON, POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON,
    POLICY_PIPELINE_BAD_ENVELOPE_REASON, POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON,
    PolicyPipelineInput, policy_pipeline_decide_v1,
};
use proptest::prelude::*;

proptest! {
    /// Total function: never panics over arbitrary `i32` fields.
    #[test]
    fn pipeline_never_panics(
        version in any::<i32>(),
        actor_kind in any::<i32>(),
        envelope_reason in any::<i32>(),
        room_epoch_reason in any::<i32>(),
        ai_grant_reason in any::<i32>(),
        ai_lifecycle_reason in any::<i32>(),
    ) {
        let _ = policy_pipeline_decide_v1(PolicyPipelineInput {
            version,
            actor_kind,
            envelope_reason,
            room_epoch_reason,
            ai_grant_reason,
            ai_lifecycle_reason,
        });
    }

    /// An out-of-range component reason yields a contract `BAD_*` rejection,
    /// never a downstream ACCEPT. Version and actor are held valid so the
    /// failure is attributable to the out-of-range component reason itself.
    #[test]
    fn out_of_range_component_reason_is_bad_reason(
        actor_kind in 1i32..=3,
        which in 0u8..4,
        // A clearly out-of-contract magnitude (ranges top out at 20).
        bad in 100i32..1_000,
    ) {
        let mut input = PolicyPipelineInput {
            version: 1,
            actor_kind,
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        };
        let expected = match which {
            0 => { input.envelope_reason = bad; POLICY_PIPELINE_BAD_ENVELOPE_REASON }
            1 => { input.room_epoch_reason = bad; POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON }
            2 => { input.ai_grant_reason = bad; POLICY_PIPELINE_BAD_AI_GRANT_REASON }
            _ => { input.ai_lifecycle_reason = bad; POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON }
        };
        let decision = policy_pipeline_decide_v1(input);
        prop_assert_ne!(decision, POLICY_PIPELINE_ACCEPT);
        prop_assert_eq!(decision, expected);
    }

    /// A human actor (`actor_kind == 1`) carrying any nonzero in-range AI grant
    /// or AI lifecycle reason is rejected with the AI-component-for-human code.
    #[test]
    fn human_actor_with_ai_component_rejected(
        ai_grant_reason in 0i32..=20,
        ai_lifecycle_reason in 0i32..=10,
    ) {
        prop_assume!(ai_grant_reason != 0 || ai_lifecycle_reason != 0);
        let decision = policy_pipeline_decide_v1(PolicyPipelineInput {
            version: 1,
            actor_kind: 1, // human
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason,
            ai_lifecycle_reason,
        });
        prop_assert_eq!(decision, POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN);
    }
}
