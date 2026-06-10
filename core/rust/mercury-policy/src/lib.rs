#![forbid(unsafe_code)]

pub const ACCEPT: i32 = 0;
pub const BAD_VERSION: i32 = 1;
pub const UNSUPPORTED_SUITE: i32 = 2;
pub const DOWNGRADED_SUITE: i32 = 3;
pub const BAD_CONVERSATION_ID_LEN: i32 = 4;
pub const BAD_SENDER_ACCOUNT_ID_LEN: i32 = 5;
pub const BAD_SENDER_DEVICE_ID_LEN: i32 = 6;
pub const BAD_EPOCH: i32 = 7;
pub const BAD_SEQUENCE: i32 = 8;
pub const BAD_MESSAGE_KIND: i32 = 9;
pub const PAYLOAD_TOO_LARGE: i32 = 10;
pub const UNKNOWN_CRITICAL_FLAG: i32 = 11;
pub const RESERVED_AI_KIND: i32 = 12;

pub const AUDIT_ACCEPTED_MESSAGE: i32 = 1;
pub const AUDIT_POLICY_REJECT: i32 = 2;
pub const AUDIT_DOWNGRADE_ATTEMPT: i32 = 3;
pub const AUDIT_SIZE_REJECT: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeInput {
    pub version: i32,
    pub suite_id: i32,
    pub conversation_id_len: i32,
    pub sender_account_id_len: i32,
    pub sender_device_id_len: i32,
    pub epoch: i32,
    pub sequence: i32,
    pub message_kind: i32,
    pub payload_len: i32,
    pub critical_flags: i32,
    pub noncritical_flags: i32,
    pub expected_epoch: i32,
    pub expected_sequence: i32,
    pub min_suite_id: i32,
    pub max_payload_len: i32,
}

pub fn supported_version(version: i32) -> bool {
    version == 1
}

pub fn supported_suite(suite_id: i32) -> bool {
    suite_id == 257 || suite_id == 513
}

pub fn valid_id_len(id_len: i32) -> bool {
    (8..=128).contains(&id_len)
}

pub fn known_message_kind(message_kind: i32) -> bool {
    matches!(message_kind, 1 | 2 | 3)
}

pub fn valid_payload_len(payload_len: i32, max_payload_len: i32) -> bool {
    payload_len >= 0 && max_payload_len >= 0 && payload_len <= max_payload_len
}

pub fn known_critical_flags(critical_flags: i32) -> bool {
    (0..=3).contains(&critical_flags)
}

pub fn validate_identity_v1(
    version: i32,
    suite_id: i32,
    min_suite_id: i32,
    conversation_id_len: i32,
    sender_account_id_len: i32,
    sender_device_id_len: i32,
) -> i32 {
    if !supported_version(version) {
        BAD_VERSION
    } else if !supported_suite(suite_id) {
        UNSUPPORTED_SUITE
    } else if suite_id < min_suite_id {
        DOWNGRADED_SUITE
    } else if !valid_id_len(conversation_id_len) {
        BAD_CONVERSATION_ID_LEN
    } else if !valid_id_len(sender_account_id_len) {
        BAD_SENDER_ACCOUNT_ID_LEN
    } else if !valid_id_len(sender_device_id_len) {
        BAD_SENDER_DEVICE_ID_LEN
    } else {
        ACCEPT
    }
}

pub fn validate_order_v1(
    epoch: i32,
    sequence: i32,
    expected_epoch: i32,
    expected_sequence: i32,
) -> i32 {
    if epoch != expected_epoch {
        BAD_EPOCH
    } else if sequence != expected_sequence {
        BAD_SEQUENCE
    } else {
        ACCEPT
    }
}

pub fn validate_content_v1(
    message_kind: i32,
    payload_len: i32,
    critical_flags: i32,
    _noncritical_flags: i32,
    max_payload_len: i32,
) -> i32 {
    if !known_message_kind(message_kind) {
        BAD_MESSAGE_KIND
    } else if message_kind == 3 {
        RESERVED_AI_KIND
    } else if !valid_payload_len(payload_len, max_payload_len) {
        PAYLOAD_TOO_LARGE
    } else if !known_critical_flags(critical_flags) {
        UNKNOWN_CRITICAL_FLAG
    } else {
        ACCEPT
    }
}

pub fn first_reject(identity_reason: i32, order_reason: i32, content_reason: i32) -> i32 {
    if identity_reason != ACCEPT {
        identity_reason
    } else if order_reason != ACCEPT {
        order_reason
    } else {
        content_reason
    }
}

pub fn audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        ACCEPT => AUDIT_ACCEPTED_MESSAGE,
        DOWNGRADED_SUITE => AUDIT_DOWNGRADE_ATTEMPT,
        PAYLOAD_TOO_LARGE => AUDIT_SIZE_REJECT,
        _ => AUDIT_POLICY_REJECT,
    }
}

pub fn validate_envelope(input: EnvelopeInput) -> i32 {
    first_reject(
        validate_identity_v1(
            input.version,
            input.suite_id,
            input.min_suite_id,
            input.conversation_id_len,
            input.sender_account_id_len,
            input.sender_device_id_len,
        ),
        validate_order_v1(
            input.epoch,
            input.sequence,
            input.expected_epoch,
            input.expected_sequence,
        ),
        validate_content_v1(
            input.message_kind,
            input.payload_len,
            input.critical_flags,
            input.noncritical_flags,
            input.max_payload_len,
        ),
    )
}

pub const AI_GRANT_ACCEPT: i32 = 0;
pub const AI_GRANT_BAD_VERSION: i32 = 1;
pub const AI_GRANT_BAD_PRINCIPAL_KIND: i32 = 2;
pub const AI_GRANT_BAD_AI_MODE: i32 = 3;
pub const AI_GRANT_AI_BLOCKED_ROOM: i32 = 4;
pub const AI_GRANT_BAD_TTL: i32 = 5;
pub const AI_GRANT_TTL_TOO_LONG: i32 = 6;
pub const AI_GRANT_INSUFFICIENT_APPROVERS: i32 = 7;
pub const AI_GRANT_BAD_READ_SCOPE: i32 = 8;
pub const AI_GRANT_READ_SCOPE_TOO_BROAD: i32 = 9;
pub const AI_GRANT_BAD_WRITE_SCOPE: i32 = 10;
pub const AI_GRANT_WRITE_SCOPE_TOO_BROAD: i32 = 11;
pub const AI_GRANT_BAD_TOOL_SCOPE: i32 = 12;
pub const AI_GRANT_TOOL_SCOPE_TOO_BROAD: i32 = 13;
pub const AI_GRANT_BAD_RETENTION_MODE: i32 = 14;
pub const AI_GRANT_PROMPT_STORE_FORBIDDEN: i32 = 15;
pub const AI_GRANT_TRAINING_FORBIDDEN: i32 = 16;
pub const AI_GRANT_REMOTE_PROVIDER_FORBIDDEN: i32 = 17;
pub const AI_GRANT_HIGHSEC_LOCAL_ONLY: i32 = 18;
pub const AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED: i32 = 19;
pub const AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN: i32 = 20;

pub const AI_GRANT_AUDIT_ACCEPTED_AI_GRANT: i32 = 1;
pub const AI_GRANT_AUDIT_AI_POLICY_REJECT: i32 = 2;
pub const AI_GRANT_AUDIT_AI_PRIVACY_REJECT: i32 = 3;
pub const AI_GRANT_AUDIT_AI_HIGHSEC_REJECT: i32 = 4;
pub const AI_GRANT_AUDIT_AI_RETENTION_REJECT: i32 = 5;
pub const AI_GRANT_AUDIT_AI_TOOL_REJECT: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiGrantInput {
    pub version: i32,
    pub principal_kind: i32,
    pub room_mode: i32,
    pub ai_mode: i32,
    pub ttl_s: i32,
    pub approver_count: i32,
    pub read_scope: i32,
    pub write_scope: i32,
    pub tool_scope: i32,
    pub retention_mode: i32,
    pub training_allowed: i32,
    pub prompt_store_allowed: i32,
}

pub fn ai_grant_valid_ai_mode(ai_mode: i32) -> bool {
    (1..=3).contains(&ai_mode)
}

pub fn ai_grant_validate_subject_v1(
    version: i32,
    principal_kind: i32,
    room_mode: i32,
    ai_mode: i32,
    ttl_s: i32,
    approver_count: i32,
) -> i32 {
    if version != 1 {
        AI_GRANT_BAD_VERSION
    } else if !(2..=4).contains(&principal_kind) {
        AI_GRANT_BAD_PRINCIPAL_KIND
    } else if !ai_grant_valid_ai_mode(ai_mode) {
        AI_GRANT_BAD_AI_MODE
    } else if principal_kind == 4 && ai_mode == 1 {
        AI_GRANT_BAD_AI_MODE
    } else if !(1..=4).contains(&room_mode) || room_mode == 4 {
        AI_GRANT_AI_BLOCKED_ROOM
    } else if ttl_s < 1 {
        AI_GRANT_BAD_TTL
    } else if ttl_s > 900 {
        AI_GRANT_TTL_TOO_LONG
    } else if approver_count < 1 {
        AI_GRANT_INSUFFICIENT_APPROVERS
    } else if ai_mode == 3 && room_mode > 1 {
        AI_GRANT_REMOTE_PROVIDER_FORBIDDEN
    } else {
        AI_GRANT_ACCEPT
    }
}

pub fn ai_grant_validate_scopes_v1(
    read_scope: i32,
    write_scope: i32,
    tool_scope: i32,
    retention_mode: i32,
    training_allowed: i32,
    prompt_store_allowed: i32,
) -> i32 {
    if !(0..=4).contains(&read_scope) {
        AI_GRANT_BAD_READ_SCOPE
    } else if read_scope == 4 {
        AI_GRANT_READ_SCOPE_TOO_BROAD
    } else if !(0..=3).contains(&write_scope) {
        AI_GRANT_BAD_WRITE_SCOPE
    } else if write_scope == 3 {
        AI_GRANT_WRITE_SCOPE_TOO_BROAD
    } else if !(0..=4).contains(&tool_scope) {
        AI_GRANT_BAD_TOOL_SCOPE
    } else if tool_scope > 2 {
        AI_GRANT_TOOL_SCOPE_TOO_BROAD
    } else if !(0..=3).contains(&retention_mode) {
        AI_GRANT_BAD_RETENTION_MODE
    } else if retention_mode > 1 || prompt_store_allowed != 0 {
        AI_GRANT_PROMPT_STORE_FORBIDDEN
    } else if training_allowed != 0 {
        AI_GRANT_TRAINING_FORBIDDEN
    } else {
        AI_GRANT_ACCEPT
    }
}

pub fn ai_grant_validate_highsec_v1(
    room_mode: i32,
    _principal_kind: i32,
    ai_mode: i32,
    write_scope: i32,
    tool_scope: i32,
    approver_count: i32,
) -> i32 {
    if room_mode != 3 {
        AI_GRANT_ACCEPT
    } else if ai_mode != 1 {
        AI_GRANT_HIGHSEC_LOCAL_ONLY
    } else if approver_count < 2 {
        AI_GRANT_INSUFFICIENT_APPROVERS
    } else if write_scope == 3 {
        AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED
    } else if tool_scope > 1 {
        AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN
    } else {
        AI_GRANT_ACCEPT
    }
}

pub fn ai_grant_first_reject(subject_reason: i32, scope_reason: i32, highsec_reason: i32) -> i32 {
    if subject_reason != AI_GRANT_ACCEPT {
        subject_reason
    } else if scope_reason != AI_GRANT_ACCEPT {
        scope_reason
    } else {
        highsec_reason
    }
}

pub fn ai_grant_audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        AI_GRANT_ACCEPT => AI_GRANT_AUDIT_ACCEPTED_AI_GRANT,
        AI_GRANT_REMOTE_PROVIDER_FORBIDDEN | AI_GRANT_READ_SCOPE_TOO_BROAD => {
            AI_GRANT_AUDIT_AI_PRIVACY_REJECT
        }
        AI_GRANT_HIGHSEC_LOCAL_ONLY
        | AI_GRANT_HIGHSEC_CONFIRM_SEND_REQUIRED
        | AI_GRANT_HIGHSEC_TOOLS_FORBIDDEN => AI_GRANT_AUDIT_AI_HIGHSEC_REJECT,
        AI_GRANT_PROMPT_STORE_FORBIDDEN | AI_GRANT_TRAINING_FORBIDDEN => {
            AI_GRANT_AUDIT_AI_RETENTION_REJECT
        }
        AI_GRANT_BAD_TOOL_SCOPE | AI_GRANT_TOOL_SCOPE_TOO_BROAD => AI_GRANT_AUDIT_AI_TOOL_REJECT,
        _ => AI_GRANT_AUDIT_AI_POLICY_REJECT,
    }
}

pub fn validate_ai_grant(input: AiGrantInput) -> i32 {
    ai_grant_first_reject(
        ai_grant_validate_subject_v1(
            input.version,
            input.principal_kind,
            input.room_mode,
            input.ai_mode,
            input.ttl_s,
            input.approver_count,
        ),
        ai_grant_validate_scopes_v1(
            input.read_scope,
            input.write_scope,
            input.tool_scope,
            input.retention_mode,
            input.training_allowed,
            input.prompt_store_allowed,
        ),
        ai_grant_validate_highsec_v1(
            input.room_mode,
            input.principal_kind,
            input.ai_mode,
            input.write_scope,
            input.tool_scope,
            input.approver_count,
        ),
    )
}

pub const AI_GRANT_LIFECYCLE_ACCEPT: i32 = 0;
pub const AI_GRANT_LIFECYCLE_BAD_VERSION: i32 = 1;
pub const AI_GRANT_LIFECYCLE_BAD_GRANT_STATE: i32 = 2;
pub const AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON: i32 = 3;
pub const AI_GRANT_LIFECYCLE_BAD_TIME: i32 = 4;
pub const AI_GRANT_LIFECYCLE_GRANT_EXPIRED: i32 = 5;
pub const AI_GRANT_LIFECYCLE_GRANT_REVOKED: i32 = 6;
pub const AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE: i32 = 7;
pub const AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE: i32 = 8;
pub const AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED: i32 = 9;
pub const AI_GRANT_LIFECYCLE_BAD_ACCESS_KIND: i32 = 10;

pub const AI_GRANT_LIFECYCLE_AUDIT_ACCEPTED_AI_GRANT_LIFECYCLE: i32 = 1;
pub const AI_GRANT_LIFECYCLE_AUDIT_AI_LIFECYCLE_POLICY_REJECT: i32 = 2;
pub const AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_EXPIRED_REJECT: i32 = 3;
pub const AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_REVOKED_REJECT: i32 = 4;
pub const AI_GRANT_LIFECYCLE_AUDIT_AI_POST_REVOKE_ACCESS_REJECT: i32 = 5;
pub const AI_GRANT_LIFECYCLE_AUDIT_AI_HIGHSEC_EPOCH_ROTATION_REQUIRED: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiGrantLifecycleInput {
    pub version: i32,
    pub grant_state: i32,
    pub revoke_reason: i32,
    pub now_s: i32,
    pub expires_at_s: i32,
    pub room_mode: i32,
    pub access_kind: i32,
    pub epoch_rotated: i32,
}

pub fn ai_grant_lifecycle_validate_state_v1(
    version: i32,
    grant_state: i32,
    revoke_reason: i32,
    now_s: i32,
    expires_at_s: i32,
    _room_mode: i32,
) -> i32 {
    if version != 1 {
        AI_GRANT_LIFECYCLE_BAD_VERSION
    } else if !(1..=2).contains(&grant_state) {
        AI_GRANT_LIFECYCLE_BAD_GRANT_STATE
    } else if now_s < 0 || expires_at_s <= 0 {
        AI_GRANT_LIFECYCLE_BAD_TIME
    } else if grant_state == 1 {
        if revoke_reason != 0 {
            AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON
        } else if now_s >= expires_at_s {
            AI_GRANT_LIFECYCLE_GRANT_EXPIRED
        } else {
            AI_GRANT_LIFECYCLE_ACCEPT
        }
    } else if !(1..=5).contains(&revoke_reason) {
        AI_GRANT_LIFECYCLE_BAD_REVOKE_REASON
    } else {
        AI_GRANT_LIFECYCLE_GRANT_REVOKED
    }
}

pub fn ai_grant_lifecycle_post_revoke_access(access_kind: i32) -> i32 {
    match access_kind {
        0 => AI_GRANT_LIFECYCLE_GRANT_REVOKED,
        1 => AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE,
        _ => AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE,
    }
}

pub fn ai_grant_lifecycle_validate_access_v1(
    lifecycle_reason: i32,
    access_kind: i32,
    room_mode: i32,
    epoch_rotated: i32,
) -> i32 {
    if !(0..=2).contains(&access_kind) {
        AI_GRANT_LIFECYCLE_BAD_ACCESS_KIND
    } else if lifecycle_reason != AI_GRANT_LIFECYCLE_GRANT_REVOKED {
        AI_GRANT_LIFECYCLE_ACCEPT
    } else if room_mode == 3 && epoch_rotated != 1 {
        AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED
    } else {
        ai_grant_lifecycle_post_revoke_access(access_kind)
    }
}

pub fn ai_grant_lifecycle_first_reject(state_reason: i32, access_reason: i32) -> i32 {
    if access_reason == AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED {
        access_reason
    } else if state_reason == AI_GRANT_LIFECYCLE_ACCEPT {
        access_reason
    } else if access_reason == AI_GRANT_LIFECYCLE_ACCEPT {
        state_reason
    } else {
        access_reason
    }
}

pub fn ai_grant_lifecycle_audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        AI_GRANT_LIFECYCLE_ACCEPT => AI_GRANT_LIFECYCLE_AUDIT_ACCEPTED_AI_GRANT_LIFECYCLE,
        AI_GRANT_LIFECYCLE_GRANT_EXPIRED => AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_EXPIRED_REJECT,
        AI_GRANT_LIFECYCLE_GRANT_REVOKED => AI_GRANT_LIFECYCLE_AUDIT_AI_GRANT_REVOKED_REJECT,
        AI_GRANT_LIFECYCLE_READ_AFTER_REVOKE | AI_GRANT_LIFECYCLE_WRITE_AFTER_REVOKE => {
            AI_GRANT_LIFECYCLE_AUDIT_AI_POST_REVOKE_ACCESS_REJECT
        }
        AI_GRANT_LIFECYCLE_HIGHSEC_EPOCH_ROTATION_REQUIRED => {
            AI_GRANT_LIFECYCLE_AUDIT_AI_HIGHSEC_EPOCH_ROTATION_REQUIRED
        }
        _ => AI_GRANT_LIFECYCLE_AUDIT_AI_LIFECYCLE_POLICY_REJECT,
    }
}

pub fn validate_ai_grant_lifecycle(input: AiGrantLifecycleInput) -> i32 {
    let state_reason = ai_grant_lifecycle_validate_state_v1(
        input.version,
        input.grant_state,
        input.revoke_reason,
        input.now_s,
        input.expires_at_s,
        input.room_mode,
    );
    ai_grant_lifecycle_first_reject(
        state_reason,
        ai_grant_lifecycle_validate_access_v1(
            state_reason,
            input.access_kind,
            input.room_mode,
            input.epoch_rotated,
        ),
    )
}

pub const ROOM_EPOCH_ACCEPT: i32 = 0;
pub const ROOM_EPOCH_BAD_VERSION: i32 = 1;
pub const ROOM_EPOCH_BAD_ROOM_MODE: i32 = 2;
pub const ROOM_EPOCH_BAD_EPOCH: i32 = 3;
pub const ROOM_EPOCH_STALE_EPOCH: i32 = 4;
pub const ROOM_EPOCH_FUTURE_EPOCH: i32 = 5;
pub const ROOM_EPOCH_BAD_DEVICE_KIND: i32 = 6;
pub const ROOM_EPOCH_BAD_DEVICE_STATE: i32 = 7;
pub const ROOM_EPOCH_BAD_REVOKED_AT_EPOCH: i32 = 8;
pub const ROOM_EPOCH_BAD_ACCESS_KIND: i32 = 9;
pub const ROOM_EPOCH_DEVICE_REMOVED: i32 = 10;
pub const ROOM_EPOCH_DEVICE_COMPROMISED: i32 = 11;
pub const ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED: i32 = 12;
pub const ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM: i32 = 13;

pub const ROOM_EPOCH_AUDIT_ACCEPTED_ROOM_EPOCH: i32 = 1;
pub const ROOM_EPOCH_AUDIT_ROOM_EPOCH_POLICY_REJECT: i32 = 2;
pub const ROOM_EPOCH_AUDIT_ROOM_EPOCH_REPLAY_REJECT: i32 = 3;
pub const ROOM_EPOCH_AUDIT_ROOM_DEVICE_MEMBERSHIP_REJECT: i32 = 4;
pub const ROOM_EPOCH_AUDIT_ROOM_HIGHSEC_ROTATION_REJECT: i32 = 5;
pub const ROOM_EPOCH_AUDIT_ROOM_AI_POLICY_REJECT: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomEpochInput {
    pub version: i32,
    pub room_mode: i32,
    pub device_kind: i32,
    pub device_state: i32,
    pub current_epoch: i32,
    pub message_epoch: i32,
    pub min_accepted_epoch: i32,
    pub revoked_at_epoch: i32,
    pub access_kind: i32,
}

pub fn room_epoch_validate_epoch_v1(
    version: i32,
    room_mode: i32,
    current_epoch: i32,
    message_epoch: i32,
    min_accepted_epoch: i32,
) -> i32 {
    if version != 1 {
        ROOM_EPOCH_BAD_VERSION
    } else if !(1..=4).contains(&room_mode) {
        ROOM_EPOCH_BAD_ROOM_MODE
    } else if current_epoch < 1 || min_accepted_epoch < 1 || current_epoch < min_accepted_epoch {
        ROOM_EPOCH_BAD_EPOCH
    } else if message_epoch < min_accepted_epoch || message_epoch < current_epoch {
        ROOM_EPOCH_STALE_EPOCH
    } else if message_epoch > current_epoch {
        ROOM_EPOCH_FUTURE_EPOCH
    } else {
        ROOM_EPOCH_ACCEPT
    }
}

pub fn room_epoch_validate_device_v1(
    device_kind: i32,
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if !(1..=2).contains(&device_kind) {
        ROOM_EPOCH_BAD_DEVICE_KIND
    } else if !(1..=3).contains(&device_state) {
        ROOM_EPOCH_BAD_DEVICE_STATE
    } else if !(0..=2).contains(&access_kind) {
        ROOM_EPOCH_BAD_ACCESS_KIND
    } else if device_state == 1 {
        if revoked_at_epoch != 0 {
            ROOM_EPOCH_BAD_REVOKED_AT_EPOCH
        } else {
            ROOM_EPOCH_ACCEPT
        }
    } else if revoked_at_epoch < 1 || revoked_at_epoch > current_epoch {
        ROOM_EPOCH_BAD_REVOKED_AT_EPOCH
    } else if device_state == 2 {
        ROOM_EPOCH_DEVICE_REMOVED
    } else {
        ROOM_EPOCH_DEVICE_COMPROMISED
    }
}

pub fn room_epoch_validate_highsec_v1(
    room_mode: i32,
    device_kind: i32,
    device_state: i32,
    revoked_at_epoch: i32,
    current_epoch: i32,
    access_kind: i32,
) -> i32 {
    if room_mode == 4 && device_kind == 2 {
        ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM
    } else if room_mode != 3 {
        ROOM_EPOCH_ACCEPT
    } else if !(0..=2).contains(&access_kind)
        || device_state == 1
        || revoked_at_epoch < 1
        || revoked_at_epoch > current_epoch
    {
        ROOM_EPOCH_ACCEPT
    } else if current_epoch <= revoked_at_epoch {
        ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED
    } else {
        ROOM_EPOCH_ACCEPT
    }
}

pub fn room_epoch_first_reject(epoch_reason: i32, device_reason: i32, highsec_reason: i32) -> i32 {
    if epoch_reason != ROOM_EPOCH_ACCEPT {
        epoch_reason
    } else if highsec_reason == ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED {
        highsec_reason
    } else if device_reason != ROOM_EPOCH_ACCEPT {
        device_reason
    } else {
        highsec_reason
    }
}

pub fn room_epoch_audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        ROOM_EPOCH_ACCEPT => ROOM_EPOCH_AUDIT_ACCEPTED_ROOM_EPOCH,
        ROOM_EPOCH_STALE_EPOCH | ROOM_EPOCH_FUTURE_EPOCH => {
            ROOM_EPOCH_AUDIT_ROOM_EPOCH_REPLAY_REJECT
        }
        ROOM_EPOCH_DEVICE_REMOVED | ROOM_EPOCH_DEVICE_COMPROMISED => {
            ROOM_EPOCH_AUDIT_ROOM_DEVICE_MEMBERSHIP_REJECT
        }
        ROOM_EPOCH_HIGHSEC_EPOCH_ROTATION_REQUIRED => ROOM_EPOCH_AUDIT_ROOM_HIGHSEC_ROTATION_REJECT,
        ROOM_EPOCH_AI_DEVICE_BLOCKED_ROOM => ROOM_EPOCH_AUDIT_ROOM_AI_POLICY_REJECT,
        _ => ROOM_EPOCH_AUDIT_ROOM_EPOCH_POLICY_REJECT,
    }
}

pub fn validate_room_epoch(input: RoomEpochInput) -> i32 {
    room_epoch_first_reject(
        room_epoch_validate_epoch_v1(
            input.version,
            input.room_mode,
            input.current_epoch,
            input.message_epoch,
            input.min_accepted_epoch,
        ),
        room_epoch_validate_device_v1(
            input.device_kind,
            input.device_state,
            input.revoked_at_epoch,
            input.current_epoch,
            input.access_kind,
        ),
        room_epoch_validate_highsec_v1(
            input.room_mode,
            input.device_kind,
            input.device_state,
            input.revoked_at_epoch,
            input.current_epoch,
            input.access_kind,
        ),
    )
}

pub const POLICY_PIPELINE_ACCEPT: i32 = 0;
pub const POLICY_PIPELINE_BAD_VERSION: i32 = 1;
pub const POLICY_PIPELINE_BAD_ACTOR_KIND: i32 = 2;
pub const POLICY_PIPELINE_BAD_ENVELOPE_REASON: i32 = 3;
pub const POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON: i32 = 4;
pub const POLICY_PIPELINE_BAD_AI_GRANT_REASON: i32 = 5;
pub const POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON: i32 = 6;
pub const POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN: i32 = 7;
pub const POLICY_PIPELINE_ENVELOPE_REJECT: i32 = 8;
pub const POLICY_PIPELINE_ROOM_EPOCH_REJECT: i32 = 9;
pub const POLICY_PIPELINE_AI_GRANT_REJECT: i32 = 10;
pub const POLICY_PIPELINE_AI_LIFECYCLE_REJECT: i32 = 11;
pub const POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT: i32 = 12;
pub const POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED: i32 = 13;

pub const POLICY_PIPELINE_AUDIT_ACCEPTED_POLICY_DECISION: i32 = 1;
pub const POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT: i32 = 2;
pub const POLICY_PIPELINE_AUDIT_MESSAGE_POLICY_REJECT: i32 = 3;
pub const POLICY_PIPELINE_AUDIT_ROOM_POLICY_REJECT: i32 = 4;
pub const POLICY_PIPELINE_AUDIT_AI_POLICY_REJECT: i32 = 5;
pub const POLICY_PIPELINE_AUDIT_AI_LIFECYCLE_POLICY_REJECT: i32 = 6;
pub const POLICY_PIPELINE_AUDIT_AI_POST_REVOKE_REJECT: i32 = 7;
pub const POLICY_PIPELINE_AUDIT_AI_HIGHSEC_ROTATION_REJECT: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyPipelineInput {
    pub version: i32,
    pub actor_kind: i32,
    pub envelope_reason: i32,
    pub room_epoch_reason: i32,
    pub ai_grant_reason: i32,
    pub ai_lifecycle_reason: i32,
}

pub fn policy_pipeline_validate_inputs_v1(
    version: i32,
    actor_kind: i32,
    envelope_reason: i32,
    room_epoch_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if version != 1 {
        POLICY_PIPELINE_BAD_VERSION
    } else if !(1..=3).contains(&actor_kind) {
        POLICY_PIPELINE_BAD_ACTOR_KIND
    } else if !(0..=12).contains(&envelope_reason) {
        POLICY_PIPELINE_BAD_ENVELOPE_REASON
    } else if !(0..=13).contains(&room_epoch_reason) {
        POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON
    } else if !(0..=20).contains(&ai_grant_reason) {
        POLICY_PIPELINE_BAD_AI_GRANT_REASON
    } else if !(0..=10).contains(&ai_lifecycle_reason) {
        POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON
    } else {
        POLICY_PIPELINE_ACCEPT
    }
}

pub fn policy_pipeline_validate_actor_components(
    actor_kind: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if actor_kind == 1 && (ai_grant_reason != 0 || ai_lifecycle_reason != 0) {
        POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN
    } else {
        POLICY_PIPELINE_ACCEPT
    }
}

pub fn policy_pipeline_map_ai_lifecycle(ai_lifecycle_reason: i32) -> i32 {
    match ai_lifecycle_reason {
        0 => POLICY_PIPELINE_ACCEPT,
        9 => POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED,
        7 | 8 => POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT,
        _ => POLICY_PIPELINE_AI_LIFECYCLE_REJECT,
    }
}

pub fn policy_pipeline_component_first_reject(
    envelope_reason: i32,
    room_epoch_reason: i32,
    actor_component_reason: i32,
    ai_grant_reason: i32,
    ai_lifecycle_reason: i32,
) -> i32 {
    if envelope_reason != 0 {
        POLICY_PIPELINE_ENVELOPE_REJECT
    } else if room_epoch_reason != 0 {
        POLICY_PIPELINE_ROOM_EPOCH_REJECT
    } else if actor_component_reason != 0 {
        actor_component_reason
    } else if ai_grant_reason != 0 {
        POLICY_PIPELINE_AI_GRANT_REJECT
    } else {
        policy_pipeline_map_ai_lifecycle(ai_lifecycle_reason)
    }
}

pub fn policy_pipeline_decide_v1(input: PolicyPipelineInput) -> i32 {
    let input_reason = policy_pipeline_validate_inputs_v1(
        input.version,
        input.actor_kind,
        input.envelope_reason,
        input.room_epoch_reason,
        input.ai_grant_reason,
        input.ai_lifecycle_reason,
    );
    if input_reason != POLICY_PIPELINE_ACCEPT {
        input_reason
    } else {
        policy_pipeline_component_first_reject(
            input.envelope_reason,
            input.room_epoch_reason,
            policy_pipeline_validate_actor_components(
                input.actor_kind,
                input.ai_grant_reason,
                input.ai_lifecycle_reason,
            ),
            input.ai_grant_reason,
            input.ai_lifecycle_reason,
        )
    }
}

pub fn policy_pipeline_audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        POLICY_PIPELINE_ACCEPT => POLICY_PIPELINE_AUDIT_ACCEPTED_POLICY_DECISION,
        POLICY_PIPELINE_BAD_VERSION
        | POLICY_PIPELINE_BAD_ACTOR_KIND
        | POLICY_PIPELINE_BAD_ENVELOPE_REASON
        | POLICY_PIPELINE_BAD_ROOM_EPOCH_REASON
        | POLICY_PIPELINE_BAD_AI_GRANT_REASON
        | POLICY_PIPELINE_BAD_AI_LIFECYCLE_REASON => POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT,
        POLICY_PIPELINE_AI_COMPONENT_FOR_HUMAN | POLICY_PIPELINE_AI_GRANT_REJECT => {
            POLICY_PIPELINE_AUDIT_AI_POLICY_REJECT
        }
        POLICY_PIPELINE_ENVELOPE_REJECT => POLICY_PIPELINE_AUDIT_MESSAGE_POLICY_REJECT,
        POLICY_PIPELINE_ROOM_EPOCH_REJECT => POLICY_PIPELINE_AUDIT_ROOM_POLICY_REJECT,
        POLICY_PIPELINE_AI_LIFECYCLE_REJECT => POLICY_PIPELINE_AUDIT_AI_LIFECYCLE_POLICY_REJECT,
        POLICY_PIPELINE_AI_POST_REVOKE_ACCESS_REJECT => POLICY_PIPELINE_AUDIT_AI_POST_REVOKE_REJECT,
        POLICY_PIPELINE_AI_HIGHSEC_ROTATION_REQUIRED => {
            POLICY_PIPELINE_AUDIT_AI_HIGHSEC_ROTATION_REJECT
        }
        _ => POLICY_PIPELINE_AUDIT_PIPELINE_CONTRACT_REJECT,
    }
}

pub const RELAY_SUBMIT_ACCEPT: i32 = 0;
pub const RELAY_SUBMIT_BAD_VERSION: i32 = 1;
pub const RELAY_SUBMIT_BAD_SEND_GATE_REASON: i32 = 2;
pub const RELAY_SUBMIT_SEND_GATE_REJECTED: i32 = 3;
pub const RELAY_SUBMIT_BAD_ROUTE_ID_LEN: i32 = 4;
pub const RELAY_SUBMIT_BAD_REPLAY_TOKEN_LEN: i32 = 5;
pub const RELAY_SUBMIT_BAD_TTL: i32 = 6;
pub const RELAY_SUBMIT_TTL_TOO_LONG: i32 = 7;
pub const RELAY_SUBMIT_BAD_CIPHERTEXT_LEN: i32 = 8;
pub const RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE: i32 = 9;
pub const RELAY_SUBMIT_BAD_SEALED_HEADER_LEN: i32 = 10;
pub const RELAY_SUBMIT_SEALED_HEADER_TOO_LARGE: i32 = 11;
pub const RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN: i32 = 12;
pub const RELAY_SUBMIT_BAD_PADDING_BUCKET: i32 = 13;

pub const RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION: i32 = 1;
pub const RELAY_SUBMIT_AUDIT_RELAY_CONTRACT_REJECT: i32 = 2;
pub const RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT: i32 = 3;
pub const RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT: i32 = 4;
pub const RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT: i32 = 5;
pub const RELAY_SUBMIT_AUDIT_RELAY_SIZE_REJECT: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelaySubmitInput {
    pub version: i32,
    pub send_gate_reason: i32,
    pub route_id_len: i32,
    pub replay_token_len: i32,
    pub queue_ttl_s: i32,
    pub max_queue_ttl_s: i32,
    pub ciphertext_len: i32,
    pub max_ciphertext_len: i32,
    pub sealed_header_len: i32,
    pub plaintext_identity_fields: i32,
    pub padding_bucket: i32,
}

pub fn relay_submit_validate_send_gate(send_gate_reason: i32) -> i32 {
    if !(0..=5).contains(&send_gate_reason) {
        RELAY_SUBMIT_BAD_SEND_GATE_REASON
    } else if send_gate_reason != RELAY_SUBMIT_ACCEPT {
        RELAY_SUBMIT_SEND_GATE_REJECTED
    } else {
        RELAY_SUBMIT_ACCEPT
    }
}

pub fn relay_submit_validate_metadata(
    route_id_len: i32,
    replay_token_len: i32,
    sealed_header_len: i32,
    plaintext_identity_fields: i32,
    padding_bucket: i32,
) -> i32 {
    if !(16..=128).contains(&route_id_len) {
        RELAY_SUBMIT_BAD_ROUTE_ID_LEN
    } else if replay_token_len != 32 {
        RELAY_SUBMIT_BAD_REPLAY_TOKEN_LEN
    } else if sealed_header_len < 16 {
        RELAY_SUBMIT_BAD_SEALED_HEADER_LEN
    } else if sealed_header_len > 4096 {
        RELAY_SUBMIT_SEALED_HEADER_TOO_LARGE
    } else if plaintext_identity_fields != 0 {
        RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN
    } else if !(1..=8).contains(&padding_bucket) {
        RELAY_SUBMIT_BAD_PADDING_BUCKET
    } else {
        RELAY_SUBMIT_ACCEPT
    }
}

pub fn relay_submit_validate_lifetime(queue_ttl_s: i32, max_queue_ttl_s: i32) -> i32 {
    if queue_ttl_s < 1 || max_queue_ttl_s < 1 {
        RELAY_SUBMIT_BAD_TTL
    } else if max_queue_ttl_s > 604800 || queue_ttl_s > max_queue_ttl_s {
        RELAY_SUBMIT_TTL_TOO_LONG
    } else {
        RELAY_SUBMIT_ACCEPT
    }
}

pub fn relay_submit_validate_ciphertext(ciphertext_len: i32, max_ciphertext_len: i32) -> i32 {
    if ciphertext_len < 1 || max_ciphertext_len < 1 {
        RELAY_SUBMIT_BAD_CIPHERTEXT_LEN
    } else if max_ciphertext_len > 4194304 || ciphertext_len > max_ciphertext_len {
        RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE
    } else {
        RELAY_SUBMIT_ACCEPT
    }
}

pub fn relay_submit_first_reject(
    version_reason: i32,
    send_gate_reason: i32,
    metadata_reason: i32,
    lifetime_reason: i32,
    ciphertext_reason: i32,
) -> i32 {
    if version_reason != RELAY_SUBMIT_ACCEPT {
        version_reason
    } else if send_gate_reason != RELAY_SUBMIT_ACCEPT {
        send_gate_reason
    } else if metadata_reason != RELAY_SUBMIT_ACCEPT {
        metadata_reason
    } else if lifetime_reason != RELAY_SUBMIT_ACCEPT {
        lifetime_reason
    } else {
        ciphertext_reason
    }
}

pub fn relay_submit_decide_v1(input: RelaySubmitInput) -> i32 {
    relay_submit_first_reject(
        if input.version == 1 {
            RELAY_SUBMIT_ACCEPT
        } else {
            RELAY_SUBMIT_BAD_VERSION
        },
        relay_submit_validate_send_gate(input.send_gate_reason),
        relay_submit_validate_metadata(
            input.route_id_len,
            input.replay_token_len,
            input.sealed_header_len,
            input.plaintext_identity_fields,
            input.padding_bucket,
        ),
        relay_submit_validate_lifetime(input.queue_ttl_s, input.max_queue_ttl_s),
        relay_submit_validate_ciphertext(input.ciphertext_len, input.max_ciphertext_len),
    )
}

pub fn relay_submit_audit_class_for_reason(reason_code: i32) -> i32 {
    match reason_code {
        RELAY_SUBMIT_ACCEPT => RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION,
        RELAY_SUBMIT_BAD_VERSION | RELAY_SUBMIT_BAD_SEND_GATE_REASON => {
            RELAY_SUBMIT_AUDIT_RELAY_CONTRACT_REJECT
        }
        RELAY_SUBMIT_SEND_GATE_REJECTED => RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT,
        RELAY_SUBMIT_BAD_TTL | RELAY_SUBMIT_TTL_TOO_LONG => {
            RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT
        }
        RELAY_SUBMIT_BAD_CIPHERTEXT_LEN | RELAY_SUBMIT_CIPHERTEXT_TOO_LARGE => {
            RELAY_SUBMIT_AUDIT_RELAY_SIZE_REJECT
        }
        _ => RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT,
    }
}
