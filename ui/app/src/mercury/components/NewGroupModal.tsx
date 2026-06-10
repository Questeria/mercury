// New-group creation modal for the LIVE app. Lists the EXISTING 1:1 conversations as candidate
// members (the caller excludes group conversations), caps the selection at 15 others (the backend
// enforces 16 members including you), and drives `controller.createGroup` — which opens AND
// activates the new group on success, so this modal only has to close itself. Backend errors
// (non-contact member, cap, …) are readable and surface verbatim.
//
// Styling follows ConversationMenu's overlay/card inline-style pattern.

import { useState, type CSSProperties } from "react";

import type { LiveConversations } from "../liveConversations";

const MAX_NAME = 48;
const MAX_MEMBERS = 15; // others you can invite — the 16-member cap includes you

export interface GroupCandidate {
  /** 1:1 peer account id (lowercase hex). */
  id: string;
  /** Display name as the rail shows it: alias/@handle or shortened hex. */
  name: string;
}

interface NewGroupModalProps {
  controller: LiveConversations;
  candidates: GroupCandidate[];
  onClose: () => void;
  /** Called after a successful create (the group is already the active conversation). */
  onCreated: () => void;
}

export function NewGroupModal({ controller, candidates, onClose, onCreated }: NewGroupModalProps) {
  const [name, setName] = useState("");
  const [selected, setSelected] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggle = (id: string) => {
    setSelected((prev) =>
      prev.includes(id)
        ? prev.filter((x) => x !== id)
        : prev.length >= MAX_MEMBERS
          ? prev // full — the unchecked boxes are disabled anyway
          : [...prev, id],
    );
  };

  const canCreate = !busy && name.trim().length > 0 && selected.length > 0;

  const create = () => {
    if (!canCreate) return;
    setBusy(true);
    setError(null);
    void (async () => {
      try {
        await controller.createGroup(name.trim(), selected);
        onCreated(); // createGroup already opened + activated the conversation
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setBusy(false);
      }
    })();
  };

  return (
    <div role="dialog" aria-modal="true" aria-label="New group" style={overlay} onClick={onClose}>
      <div style={card} onClick={(e) => e.stopPropagation()}>
        <div style={head}>
          <span style={{ fontWeight: 700, fontSize: 14 }}>New group</span>
          <button type="button" style={closeBtn} onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>

        <section style={section}>
          <div style={kicker}>Group name</div>
          <input
            style={input}
            value={name}
            maxLength={MAX_NAME}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. weekend plans"
            aria-label="Group name"
            autoFocus
          />
        </section>

        <section style={section}>
          <div style={{ ...kicker, display: "flex", justifyContent: "space-between" }}>
            <span>Members</span>
            <span>
              {selected.length}/{MAX_MEMBERS}
            </span>
          </div>
          {candidates.length === 0 ? (
            <div style={emptyBox}>
              No 1:1 conversations yet. Groups are built from people you already added — connect
              with a contact first.
            </div>
          ) : (
            <div style={listBox}>
              {candidates.map((c) => {
                const on = selected.includes(c.id);
                const full = !on && selected.length >= MAX_MEMBERS;
                return (
                  <label
                    key={c.id}
                    style={{
                      ...rowStyle,
                      opacity: full ? 0.45 : 1,
                      cursor: full || busy ? "default" : "pointer",
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={on}
                      disabled={full || busy}
                      onChange={() => toggle(c.id)}
                      style={{ accentColor: "var(--mc-accent)", flexShrink: 0 }}
                    />
                    <span style={rowName}>{c.name}</span>
                  </label>
                );
              })}
            </div>
          )}
          <div style={note}>
            Members must already be contacts. Invitations are end-to-end encrypted; people join
            when their app responds (they may be offline).
          </div>
        </section>

        {error && (
          <div style={errBox} role="alert">
            {error}
          </div>
        )}

        <div style={{ display: "flex", gap: 8 }}>
          <button type="button" style={ghostBtn} onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button
            type="button"
            style={{
              ...primaryBtn,
              opacity: canCreate ? 1 : 0.55,
              cursor: canCreate ? "pointer" : "default",
            }}
            onClick={create}
            disabled={!canCreate}
          >
            {busy ? "Creating…" : "Create group"}
          </button>
        </div>
      </div>
    </div>
  );
}

const overlay: CSSProperties = {
  position: "fixed",
  inset: 0,
  zIndex: 60,
  background: "rgba(0,0,0,0.45)",
  backdropFilter: "blur(2px)",
  WebkitBackdropFilter: "blur(2px)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: 20,
};

const card: CSSProperties = {
  width: "min(440px, 92vw)",
  maxHeight: "86vh",
  overflowY: "auto",
  background: "var(--mc-surface)",
  color: "var(--mc-ink)",
  border: "1px solid var(--mc-border)",
  borderRadius: 16,
  padding: 18,
  boxShadow: "0 24px 64px rgba(0,0,0,0.4)",
};

const head: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  marginBottom: 14,
};

const closeBtn: CSSProperties = {
  border: "1px solid var(--mc-border)",
  background: "var(--mc-surface-warm)",
  color: "var(--mc-ink2)",
  borderRadius: 8,
  width: 28,
  height: 28,
  cursor: "pointer",
};

const section: CSSProperties = { marginBottom: 16 };

const kicker: CSSProperties = {
  fontSize: 10.5,
  letterSpacing: 1,
  textTransform: "uppercase",
  color: "var(--mc-muted)",
  marginBottom: 8,
};

const input: CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  font: "inherit",
  fontSize: 13,
  color: "var(--mc-ink)",
  background: "var(--mc-bg)",
  border: "1px solid var(--mc-border)",
  borderRadius: 9,
  padding: "8px 10px",
  outline: "none",
};

const listBox: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: 2,
  maxHeight: 220,
  overflowY: "auto",
  border: "1px solid var(--mc-border)",
  borderRadius: 10,
  padding: 6,
  background: "var(--mc-surface-warm)",
};

const rowStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 9,
  padding: "7px 8px",
  borderRadius: 8,
  fontSize: 12.5,
};

const rowName: CSSProperties = {
  minWidth: 0,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
  color: "var(--mc-ink)",
};

const emptyBox: CSSProperties = {
  padding: "12px 12px",
  fontSize: 12,
  color: "var(--mc-muted)",
  border: "1px dashed var(--mc-border-strong)",
  borderRadius: 10,
  lineHeight: 1.5,
};

const note: CSSProperties = {
  fontSize: 11.5,
  color: "var(--mc-muted)",
  marginTop: 9,
  lineHeight: 1.5,
};

const errBox: CSSProperties = {
  marginBottom: 12,
  padding: "9px 12px",
  borderRadius: 10,
  background: "var(--mc-bad-soft)",
  border: "1px solid color-mix(in srgb, var(--mc-bad) 35%, transparent)",
  color: "var(--mc-bad)",
  fontSize: 12.5,
  lineHeight: 1.45,
};

const ghostBtn: CSSProperties = {
  flex: 1,
  border: "1px solid var(--mc-border)",
  background: "var(--mc-surface-warm)",
  color: "var(--mc-ink)",
  borderRadius: 10,
  padding: "10px 12px",
  fontSize: 13,
  cursor: "pointer",
};

const primaryBtn: CSSProperties = {
  flex: 1,
  border: "none",
  background: "linear-gradient(135deg, var(--mc-accent), #4b7bff)",
  color: "#fff",
  borderRadius: 10,
  padding: "10px 12px",
  fontSize: 13,
  fontWeight: 600,
};
