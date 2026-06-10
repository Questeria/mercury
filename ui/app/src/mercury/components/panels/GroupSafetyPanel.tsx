// Group: membership + honest group-security panel.
//
// LIVE mode (`controller` + `group` props present — the live app with a REAL MLS group active):
// shows the REAL roster from the backend's GroupInfo (creator badge, "you", pending invitees),
// states exactly what v1 group crypto does and does not provide (real MLS, classical-not-PQ,
// creator-administered, 16-member cap, fan-out metadata — see docs/GROUPS.md), and drives the
// real leave/close action with a confirm step. Nothing simulated.
//
// DEMO mode (no controller / no real group): the original mockup content, unchanged. The live
// app keeps the "Preview" banner on this path so the mockup can never pass for the real thing.

import { useState } from "react";

import { MERCURY_PEOPLE } from "../../simulator";
import { FU_AI, FU_TRUST, toneVar } from "../../strings";
import type { LiveConversations } from "../../liveConversations";
import type { GroupInfo } from "../../messaging";
import type { AiState, Person, TrustState } from "../../types";
import { Avatar } from "../Avatar";
import { PanelShell } from "./PanelShell";
import { Bullet, GhostButton, HistoryRow, IridButton, ScopeTile, Section, StatusRow } from "./primitives";
import styles from "./panels.module.css";

interface GroupSafetyPanelProps {
  mobile: boolean;
  onClose: () => void;
  /** DEMO (simulator) props — used only when `controller`/`group` are absent. */
  trust?: TrustState;
  ai?: AiState;
  onRotateEpoch?: () => void;
  onOpenTrust?: () => void;
  /** LIVE props — `controller` + `group` + `activePeer` together select the real panel. */
  controller?: LiveConversations;
  /** The ACTIVE conversation's real group metadata (the active conversation IS this group). */
  group?: GroupInfo | null;
  /** The active conversation id = the group id (lowercase hex). */
  activePeer?: string;
  /** Username aliases (peer hex -> handle) for member display names. */
  aliases?: Record<string, string>;
  /** Your own account id, to mark "you" and detect the creator role. */
  accountId?: string | null;
}

export function GroupSafetyPanel(props: GroupSafetyPanelProps) {
  if (props.controller && props.group && props.activePeer) {
    return (
      <LiveGroupPanel
        mobile={props.mobile}
        onClose={props.onClose}
        controller={props.controller}
        group={props.group}
        groupId={props.activePeer}
        aliases={props.aliases ?? {}}
        accountId={props.accountId ?? null}
      />
    );
  }
  return (
    <DemoGroupSafetyPanel
      mobile={props.mobile}
      onClose={props.onClose}
      trust={props.trust ?? "trusted"}
      ai={props.ai ?? "absent"}
      onRotateEpoch={props.onRotateEpoch ?? (() => {})}
      onOpenTrust={props.onOpenTrust ?? (() => {})}
    />
  );
}

// ---------------------------------------------------------------- live (real backend)

function shortHex(id: string): string {
  return id.length > 16 ? `${id.slice(0, 6)}…${id.slice(-4)}` : id;
}

/** Display Person for a member id — alias/@handle beats 64 hex; hue derived from the id the same
 *  way the conversation layer does, so a member renders with consistent colour everywhere. */
function memberPerson(id: string, aliases: Record<string, string>): Person {
  const alias = aliases[id];
  return {
    id,
    name: alias ? `@${alias}` : shortHex(id),
    short: (alias ?? id).slice(0, 2).toUpperCase(),
    hue: Math.round((Number.parseInt(id.slice(0, 2) || "0", 16) / 255) * 360),
  };
}

function LiveGroupPanel({
  mobile,
  onClose,
  controller,
  group,
  groupId,
  aliases,
  accountId,
}: {
  mobile: boolean;
  onClose: () => void;
  controller: LiveConversations;
  group: GroupInfo;
  groupId: string;
  aliases: Record<string, string>;
  accountId: string | null;
}) {
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const youAreCreator = accountId != null && group.creator === accountId;
  // As creator, leaving CLOSES the group for everyone (v1 is creator-administered). Once the
  // group is already closed, the action just removes it from your list.
  const leaveLabel = group.ended ? "Remove group" : youAreCreator ? "Close group for everyone" : "Leave group";

  const leave = () => {
    if (busy) return;
    setBusy(true);
    void (async () => {
      // leaveGroup reports failures via the app's error banner; either way this panel is done.
      await controller.leaveGroup(groupId);
      onClose();
    })();
  };

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Group"
      title={group.name}
      footer={
        confirming ? (
          <>
            <GhostButton onClick={() => setConfirming(false)}>Cancel</GhostButton>
            <GhostButton danger onClick={leave}>
              {busy
                ? "Working…"
                : group.ended
                  ? "Remove — confirm"
                  : youAreCreator
                    ? "Close for everyone — confirm"
                    : "Leave — confirm"}
            </GhostButton>
          </>
        ) : (
          <>
            <GhostButton onClick={onClose}>Done</GhostButton>
            <GhostButton danger onClick={() => setConfirming(true)}>
              {leaveLabel}
            </GhostButton>
          </>
        )
      }
    >
      <StatusRow
        tone={group.ended ? "bad" : "ok"}
        label={group.ended ? "Group closed" : "Group active"}
        sub={
          group.ended
            ? "closed by its creator — no new messages"
            : "real MLS group · creator-administered"
        }
      />

      <Section kicker="At a glance">
        <div className={styles.featureGrid3}>
          <ScopeTile
            label="Members"
            value={`${group.members.length}/16`}
            tone="ok"
            note="cap incl. you"
          />
          <ScopeTile
            label="Invited"
            value={String(group.pending.length)}
            tone={group.pending.length > 0 ? "warn" : undefined}
            note={group.pending.length > 0 ? "waiting to join" : "none waiting"}
          />
          <ScopeTile
            label="Status"
            value={group.ended ? "closed" : "open"}
            tone={group.ended ? "bad" : "ok"}
            note={youAreCreator ? "you administer it" : "creator administers"}
          />
        </div>
      </Section>

      <Section kicker="Members">
        <div className={styles.colGap}>
          {group.members.map((id) => {
            const self = accountId != null && id === accountId;
            const creator = id === group.creator;
            const p = memberPerson(id, aliases);
            return (
              <div className={styles.attachmentRow} key={id}>
                <Avatar person={p} size={26} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div className={styles.attachmentName}>
                    {creator && (
                      <span title="Group creator" aria-label="Group creator" style={{ marginRight: 5 }}>
                        👑
                      </span>
                    )}
                    {p.name}
                    {self && (
                      <span style={{ marginLeft: 6, fontSize: 10, fontWeight: 700, color: "var(--mc-muted)" }}>
                        you
                      </span>
                    )}
                  </div>
                  <div className={styles.attachmentMeta}>
                    {creator ? "creator · admits joins, processes leaves, can close" : "member"}
                  </div>
                </div>
                <span style={{ color: toneVar("ok"), fontSize: 11, fontWeight: 700 }}>in</span>
              </div>
            );
          })}
          {group.pending.map((id) => {
            const p = memberPerson(id, aliases);
            return (
              <div className={styles.attachmentRow} key={`pending-${id}`} style={{ opacity: 0.75 }}>
                <Avatar person={p} size={26} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div className={styles.attachmentName}>{p.name}</div>
                  <div className={styles.attachmentMeta}>
                    invited — waiting for their app to respond (they may be offline)
                  </div>
                </div>
                <span style={{ color: toneVar("warn"), fontSize: 11, fontWeight: 700 }}>invited</span>
              </div>
            );
          })}
        </div>
      </Section>

      <Section kicker="What protects this group">
        <div className={styles.colGap}>
          <Bullet
            tone="ok"
            text="Real MLS encryption (OpenMLS). Membership changes rotate the group key, so someone who leaves loses access to everything sent afterwards."
          />
          <Bullet
            tone="warn"
            text="Classical, not post-quantum (v1): the published PQ-MLS provider currently pins dependencies with open security advisories, so Mercury ships audited classical MLS instead of claiming fake PQ. Details: docs/GROUPS.md."
          />
          <Bullet
            tone="ok"
            text="Invitations and messages travel inside the existing sealed 1:1 channels — the relay sees only ciphertext and gains no new surface."
          />
          <Bullet text="Creator-administered (v1): the creator admits joins, processes leaves, and is the only one who can close the group. If the creator is offline, joins and leaves queue." />
          <Bullet text="16-member cap: one group message is one ciphertext fanned out per member, so the relay (which reads nothing) can still observe the fan-out timing pattern. The cap keeps that honest." />
        </div>
      </Section>
    </PanelShell>
  );
}

// ---------------------------------------------------------------- demo (simulator mockup)

function DemoGroupSafetyPanel({
  mobile,
  onClose,
  trust,
  ai,
  onRotateEpoch,
  onOpenTrust,
}: {
  mobile: boolean;
  onClose: () => void;
  trust: TrustState;
  ai: AiState;
  onRotateEpoch: () => void;
  onOpenTrust: () => void;
}) {
  const [pendingInvite, setPendingInvite] = useState(true);
  const t = FU_TRUST[trust];
  const a = FU_AI[ai];
  const needsRotation = ai === "revoked" || trust === "stale" || trust === "rejected";

  return (
    <PanelShell
      mobile={mobile}
      onClose={onClose}
      eyebrow="Room safety"
      title="Membership and epoch"
      footer={
        <>
          <GhostButton onClick={onOpenTrust}>Verify devices</GhostButton>
          <IridButton
            onClick={() => {
              onRotateEpoch();
              setPendingInvite(false);
            }}
          >
            Rotate epoch
          </IridButton>
        </>
      }
    >
      <StatusRow
        tone={needsRotation ? "warn" : t.tone}
        label={needsRotation ? "Rotation recommended" : "Room epoch current"}
        sub={needsRotation ? "membership or AI access changed" : "all active members are on epoch 42"}
      />

      <Section kicker="Current epoch">
        <div className={styles.featureGrid3}>
          <ScopeTile label="Epoch" value="42" tone="ok" note="active send epoch" />
          <ScopeTile label="Members" value="3 human" tone={t.tone} note={t.label.toLowerCase()} />
          <ScopeTile label="AI" value={a.label} tone={a.tone} note={a.sub} />
        </div>
      </Section>

      <Section kicker="Members">
        <div className={styles.colGap}>
          {["me", "rin", "jules"].map((id) => {
            const p = MERCURY_PEOPLE[id];
            const warn = id === "jules" && trust === "unverified";
            return (
              <div className={styles.attachmentRow} key={id}>
                <Avatar person={p} size={26} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div className={styles.attachmentName}>{p.name}</div>
                  <div className={styles.attachmentMeta}>
                    {id === "me" ? "this device / verified" : warn ? "new device pending" : "all devices verified"}
                  </div>
                </div>
                <span style={{ color: warn ? toneVar("warn") : toneVar("ok"), fontSize: 11, fontWeight: 700 }}>
                  {warn ? "pending" : "ok"}
                </span>
              </div>
            );
          })}
          {ai !== "absent" && (
            <div className={styles.attachmentRow}>
              <Avatar person={MERCURY_PEOPLE.ai} size={26} />
              <div style={{ minWidth: 0, flex: 1 }}>
                <div className={styles.attachmentName}>Mercury AI</div>
                <div className={styles.attachmentMeta}>{a.sub}</div>
              </div>
              <span style={{ color: toneVar(a.tone), fontSize: 11, fontWeight: 700 }}>{ai}</span>
            </div>
          )}
        </div>
      </Section>

      <Section kicker="Pending changes">
        <div className={styles.colGap}>
          {pendingInvite ? (
            <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ink2)", lineHeight: 1.5 }}>
              Jules has one new device in TOFU. Sends are allowed only after the trust gate accepts or
              the room rotates away from the previous member set.
            </div>
          ) : (
            <div className={styles.card} style={{ fontSize: 11.5, color: "var(--mc-ok)", lineHeight: 1.5 }}>
              No pending membership changes remain in this room.
            </div>
          )}
        </div>
      </Section>

      <Section kicker="Audit trail">
        <div className={styles.timeline}>
          <HistoryRow when="now" by="room_epoch" event="epoch 42 active" tone="ok" />
          <HistoryRow when="12m ago" by="trust" event="Jules device c4d9 entered TOFU" tone="warn" />
          <HistoryRow when="3d ago" by="ai" event="AI grant revoked and access tombstoned" tone="bad" />
        </div>
      </Section>
    </PanelShell>
  );
}
