//! Property tests for `validate_ai_grant`.
//!
//! Covers totality plus a targeted security invariant: a high-security room
//! (`room_mode == 3`) must never accept a non-local AI mode.

use mercury_policy::{
    AI_GRANT_ACCEPT, AI_GRANT_HIGHSEC_LOCAL_ONLY, AiGrantInput, ai_grant_validate_highsec_v1,
    validate_ai_grant,
};
use proptest::prelude::*;

fn any_ai_grant() -> impl Strategy<Value = AiGrantInput> {
    (
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
    )
        .prop_map(
            |(
                version,
                principal_kind,
                room_mode,
                ai_mode,
                ttl_s,
                approver_count,
                read_scope,
                write_scope,
                tool_scope,
                retention_mode,
                training_allowed,
                prompt_store_allowed,
            )| AiGrantInput {
                version,
                principal_kind,
                room_mode,
                ai_mode,
                ttl_s,
                approver_count,
                read_scope,
                write_scope,
                tool_scope,
                retention_mode,
                training_allowed,
                prompt_store_allowed,
            },
        )
}

proptest! {
    /// Property 1: total function — never panics over arbitrary `i32` fields.
    #[test]
    fn validate_ai_grant_never_panics(input in any_ai_grant()) {
        let _ = validate_ai_grant(input);
    }

    /// Security property: a high-security room (`room_mode == 3`) with a
    /// non-local AI mode (`ai_mode != 1`) must ALWAYS reject, and the highsec
    /// sub-validator must flag the local-only rule, regardless of any other
    /// field values.
    #[test]
    fn highsec_room_blocks_nonlocal_ai(mut input in any_ai_grant(), ai_mode in any::<i32>()) {
        input.room_mode = 3;
        // Constrain ai_mode to anything other than 1 (local).
        input.ai_mode = if ai_mode == 1 { 2 } else { ai_mode };

        // The grant pipeline never accepts this configuration.
        prop_assert_ne!(validate_ai_grant(input), AI_GRANT_ACCEPT);

        // And the dedicated highsec check is the one enforcing local-only.
        prop_assert_eq!(
            ai_grant_validate_highsec_v1(
                input.room_mode,
                input.principal_kind,
                input.ai_mode,
                input.write_scope,
                input.tool_scope,
                input.approver_count,
            ),
            AI_GRANT_HIGHSEC_LOCAL_ONLY
        );
    }
}
