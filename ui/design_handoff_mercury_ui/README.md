# Handoff: Mercury — Secure Messaging UI (message thread + supporting surfaces)

## Overview

Mercury is a secure, end-to-end-encrypted messaging client. This handoff covers the
**message thread** surface and its supporting surfaces (trust/verification, AI grant
management, recovery, sync remediation, profiles, settings, notifications).

The defining product constraint — and the reason this prototype exists — is that
**every user-facing action is gated by a `PlatformDecisionView` returned from
`mercury-core`** (the Rust policy core). The UI's job is to *render* those decisions
faithfully, never to compute or second-guess them. The prototype ships a faithful
in-memory **simulator** of that decision boundary so the UI can be built and tested
before the real FFI binding exists.

This package is the design reference for recreating that UI in a production codebase.

## About the Design Files

The files in this bundle are **design references created in HTML/React-via-Babel** —
prototypes showing intended look and behavior. **They are not production code to copy
directly.** They use in-browser Babel transpilation, inline-style objects, and a
mock decision engine — none of which belong in a shipping app.

Your task is to **recreate these designs in the target codebase's environment**, using
its established patterns, component library, and styling system. If no front-end
environment exists yet, choose the most appropriate stack for the platform (the
integration report suggests the team has not yet committed — see "Open Engineering
Items"). The integration report (`30_UI_INTEGRATION_REPORT.md`, included) is the
source of truth for *behavior*; this prototype is the source of truth for *look and
interaction*.

## Fidelity

**High-fidelity (hifi).** Final colors, typography, spacing, motion, and interaction
states are all specified and intentional. Recreate the UI pixel-accurately using the
target codebase's libraries — but adapt the *architecture* (state, styling approach,
component boundaries) to that codebase. Do not port inline-style objects verbatim;
translate the design tokens (below) into the codebase's system.

---

## The decision boundary (read this first)

Everything hinges on one data shape, the `PlatformDecisionView`. The prototype mirrors
it exactly in `mercury-core.jsx`. Every view has this shape:

```jsonc
{
  "source": "client_bootstrap | outbound_send | client_receive | client_policy",
  "accepted": true,
  "reason_code": 0,            // wire-level integer — DO NOT branch on this in UI
  "reason_label": "ACCEPTED",  // canonical string — UI displays/branches on THIS
  "can_open_message_ui": false,
  "can_start_sync": false,
  "can_send": false,
  "can_receive": false,
  "can_persist_ciphertext": false,
  "requires_sync": false,
  "requires_recovery": false,
  "requires_client_retry": false,
  "requires_user_action": false
}
```

**Hard integration rules (from the report — these are non-negotiable):**

1. UI must not call lower-level policy functions directly when a platform view is available.
2. UI must not map raw numeric `reason_code`s itself — use `reason_label`.
3. UI must not convert a rejected state into an accepted one for convenience.
4. UI must not store plaintext message/media/prompt/AI-transcript data in durable storage.
5. UI must not show notification previews unless receive + bootstrap gates allow the message surface.
6. UI must preserve AI participant visibility and grant boundaries.
7. High-security mode is stricter than standard (future binding state).

**Reason labels the UI handles** (label → gloss):

| reason_label | source | meaning surfaced |
|---|---|---|
| `ACCEPTED` | any | proceed |
| `SYNC_INCOMPLETE` | client_bootstrap | keep message UI closed, offer sync |
| `RECOVERY_REQUIRED` | client_bootstrap | keep message UI closed, offer recovery |
| `ORDERING_GAP` | client_receive | retry/fetch; render no plaintext |
| `SENDER_TRUST_REJECTED` | client_receive | withhold inbound plaintext, user-action path |
| `TOFU_PENDING` | outbound_send | first-use verification before send |
| `KEY_STALE` | outbound_send | re-handshake required, block send |
| `RECIPIENT_TRUST_REJECTED` | outbound_send | block send |
| `AI_GRANT_ABSENT` / `AI_GRANT_REVOKED` / `AI_GRANT_EXPIRED` | outbound_send | block `@ai` send |

The expected production binding functions (names approximate, FFI not yet fixed):
`mercury_bootstrap_status()`, `mercury_prepare_send()`,
`mercury_accept_received_ciphertext()`, `mercury_policy_status()` — each returns a
`PlatformDecisionView`. **Keep all simulation behind this same shape so it can be
swapped for the real binding.** The prototype already does this: see `useMercuryThread`.

---

## Screens / Views

The prototype renders one product, "Mercury", as paired **mobile (iOS, 402×874)** and
**desktop (macOS, 1160×748)** frames. Layout differs by breakpoint; content/behavior is
shared. There is one primary surface (the thread) plus nine overlay panels.

### 1. Message thread (primary)

- **Purpose:** read and send messages in a room (`# mercury-core`), with the decision
  state for every action made legible.
- **Desktop layout:** three columns — left **Rail** (232px, collapsible), center
  **Thread** (flex), right **Inspector** (290px, collapsible). All three columns are
  `position: relative; z-index` above a shared animated background.
- **Mobile layout:** single column. Rail and Inspector become slide-in overlays
  (left and right respectively); a third overlay layer hosts the detail panels as
  bottom sheets.
- **Thread regions, top to bottom:**
  - **Header** (sticky, blurred surface): `# mercury-core` title; left a `rooms`
    toggle pill, right the user avatar + a gear (Settings) button + an `inspect`
    toggle pill. Below the title row, a **data strip** of monospace `key=value` chips:
    `trust=…`, `ai=…`, `peers=4`, `encryption=end-to-end`, `bootstrap=ACCEPTED`. Each
    chip is a button that opens its detail panel; each shows a `›` affordance and a
    themed hover tooltip. The trust chip's status dot pulses when trust changes.
  - **Message list** (scrolls; auto-scrolls only when already near bottom; day
    dividers between dates; empty-state when zero messages):
    - **Incoming bubble:** left-aligned, avatar + name + per-author tint. Author color
      derives from a hue (Rin = hue 16 warm, Jules = hue 152 teal). Consecutive
      messages from the same author group (avatar/name omitted). Timestamp shown
      under *every* bubble. AI messages: iridescent outline ring + iridescent name +
      character-by-character streaming + "scoped · read-only" tag.
    - **Outgoing bubble:** right-aligned, surface fill with iridescent outline ring,
      delivery footer (`○ sending` → `● delivered · HH:MM:SS`, plus reason_label if
      not ACCEPTED).
    - **System / gate banners:** ordering-gap (amber, spinner, `requires_client_retry`),
      send-blocked (red, shows withheld text + `reason_label · rc · ciphertext not
      persisted`), inbound-withheld (dashed, `SENDER_TRUST_REJECTED`).
  - **TOFU sheet** (when a send is parked on first-use verification): amber, shows
    safety-number, Confirm & send / Cancel.
  - **Composer** (sticky, blurred): a monospace prefix cell `~/mercury-core ›`, a
    textarea, and a send button. The send button carries the iridescent outline when
    `can_send`, a lock icon when blocked. Below it, a monospace diagnostic line:
    `outbound_send · <reason_label> · rc=<n>` plus conditional `requires_user_action`,
    `no persistence`, `@ai → scoped context`. A blinking cursor shows when the input
    is empty and sending is allowed. Enter sends; Shift+Enter newlines.

### 2. Bootstrap lock (replaces the thread when `can_open_message_ui = false`)

- **Purpose:** keep the message surface closed until `client_bootstrap` accepts.
- Shows reason (`SYNC_INCOMPLETE` → "Finishing sync"; `RECOVERY_REQUIRED` → "Recovery
  required"), a monospace `reason_label · rc` chip, and CTAs: "Continue sync" (opens
  Sync panel) and/or "Open recovery" (opens Recovery panel).

### 3. Rooms rail / overlay

- Wordmark "Mercury" (iridescent text) + version; search affordance (`⌘K`, decorative
  in prototype); a `ROOMS` kicker; numbered room rows (`01 mercury-core` …) with a
  trust dot; a footer "You" button (avatar + device id) that opens your profile.

### 4. Inspector ("Binding inspector")

- **Purpose:** make the live decision views visible — the brand point.
- Helix glyph header + "live". Three **DecisionCard**s (Bootstrap / Outbound send /
  Last receive): each shows an ACCEPT/REJECT chip, the reason_label, a plain-language
  explanation, the *true* effect flags as friendly bullets, and a collapsible "RAW
  FIELDS" dump of the full boolean view. Below: a Participants list (tap → profile),
  and a bottom **event tail** that appends a new row each time any decision view
  changes (`tail -f`-style, fresh rows highlight then fade).

### 5–11. Detail panels (modal on desktop, bottom sheet on mobile — `FusePanelShell`)

- **Trust:** room safety number grid (12 groups), per-participant device list with
  verification dots, key-transparency status, "Mark all verified" CTA.
- **AI grant:** status card (iridescent when granted), scope grid (Room/Mode/Read/Send/
  Tools/Expires), grant history; Revoke / Request CTAs that flip AI state.
- **Peers:** participant rows (tap → profile); "Verify devices" CTA → Trust.
- **Encryption:** plain-language explainer (incl. honest "metadata still server-visible"
  caveat), algorithm card (XChaCha20-Poly1305, Double Ratchet, Ed25519/X25519, sender
  keys, KT log), room fingerprint.
- **Bootstrap:** ACCEPT/REJECT status, live effect flags, history, "about this gate".
- **Profile** (adapts to self / peer / AI): hero avatar + status; self → devices +
  identity + recovery + notifications link; peer → devices + shared rooms + activity +
  "Verify in person"; AI → explainer + scope grid + "Manage grant".
- **Recovery:** 3-step wizard (phrase entry → animated restore progress → success with
  TOFU explanation), stepper.
- **Sync:** total + per-room animated progress bars ("N messages behind"); auto-completes.
- **Notifications:** preview/sound/AI-alert toggles; **live lock-screen preview demo**
  proving previews respect the gate — accepted message shows text, TOFU/rejected show
  `[withheld] …`, previews-off collapses all to "New message".
- **Settings hub:** identity card → profile; Appearance (Light/Dark/Auto segment);
  Security rows → Trust/Encryption/Recovery; Notifications row; About (build info).

---

## Interactions & Behavior

- **Send:** Enter (no shift). If `!can_send`, the attempt is recorded as a send-blocked
  banner and the draft cleared. If `requires_user_action` (TOFU), the draft is parked
  and the TOFU sheet appears; Confirm completes the send. Accepted sends animate
  `sending` → `delivered` after ~900ms.
- **AI:** typing `@ai …` is detected live (composer shows `@ai → scoped context`). If a
  grant is present, sending `@ai` schedules a streamed AI reply (~1.8s later). If absent/
  revoked/expired, the send is blocked at the outbound gate.
- **Inbound scheduler:** a scripted inbound arrives every ~11s, routed through the
  receive decision — accepted renders; `ORDERING_GAP` shows a gap banner then resolves
  after ~3s; `SENDER_TRUST_REJECTED` shows a withheld entry.
- **Scrolling:** auto-scroll to bottom on new content *only if* the user is within 80px
  of the bottom (sticky). Smooth behavior; `overscroll-behavior: contain`.
- **Panels:** open via chips / avatars / toggles. Backdrop click or `✕` closes. **Esc
  closes any open panel.** Mobile overlays slide (rail from left, inspector from right,
  detail sheets from bottom); desktop inspector/rail collapse inline, detail panels are
  centered modals that scale in.
- **Avatars are profile triggers everywhere** (thread, inspector, peers, rail footer).
- **Theme:** Light / Dark / Auto. Auto reads `prefers-color-scheme` and tracks live.

### Motion (durations / easing)

- Iridescent outline: animated conic-gradient *angle* (11s linear) — the ring geometry
  is static, only the colors drift. **Do not rotate the element** (a previous bug).
- Iridescent fills/text: `background-position` drift (12–14s linear).
- Background: 4 blurred pastel blobs drifting (26s ease-in-out).
- Message entrance: `translateY(6px)` + fade, 240–260ms ease-out.
- Trust dot pulse: scale 1→1.4→1, 550ms ease-out, keyed to trust change.
- AI streaming: ~18ms/tick, ~text.length/80 chars per tick, blinking caret at the edge.
- Panels: transform/opacity 220–260ms `cubic-bezier(.2,.7,.2,1)`.
- Tooltip: fade/translate in 140ms; hovered trigger raised to `z-index:50`.

### Accessibility

- All interactive elements are real `<button>`s; `:focus-visible` outline (2px, accent
  color, 2px offset). Toggles use `role="switch" aria-checked`; theme segment uses
  `role="radiogroup"`/`radio`. Icon-only buttons have `aria-label`; toggles expose
  `aria-expanded`. Tooltips use a `data-tip` attribute (visual only — keep the
  `aria-label` for SR users). Esc closes panels.

---

## State Management

The single source of truth for thread behavior is the `useMercuryThread` hook
(`mercury-core.jsx`). It takes the current scenario inputs and returns the messages,
draft, the three decision views, and actions. **Replicate this hook's *interface* against
the real binding** — the inputs below are prototype simulation knobs that the real app
will derive from `mercury-core`:

- **Inputs (simulation):** `bootstrap` (`accepted`/`sync_incomplete`/`recovery_required`),
  `trust` (`trusted`/`sendable_not_full`/`unverified`/`stale`/`rejected`), `ai`
  (`granted`/`absent`/`revoked`/`expired`), `receiveMode` (`accepted`/`ordering_gap`),
  `senderTrust` (`trusted`/`rejected`), `forceSendOutcome` (demo override).
- **Returned state:** `messages[]`, `draft`, `bootstrapView`, `outboundView`,
  `receiveView`, `pendingTofu`, `retryingGap`, `unlocked`, `draftMentionsAi`.
- **Returned actions:** `send()`, `setDraft()`, `acceptTofu()`, `cancelTofu()`,
  `retryGap()`.
- **UI-only state (in the variant):** which detail panel is open (`activePanel`),
  rail/inspector open booleans, theme choice.

In production, the views come from `mercury_*` binding calls; the message list is the
render of accepted receive/send decisions; **plaintext is never persisted durably**
(rule 4) — treat the message array as ephemeral render state hydrated from the binding.

---

## Design Tokens

Two themes. Values are pulled from `FUSION_LIGHT` / `FUSION_DARK` in
`mercury-variant-fusion.jsx`.

### Colors — Light
| token | value | use |
|---|---|---|
| bg | `#FAFAFC` | app background |
| surface | `#FFFFFF` | bubbles, panels |
| surfaceWarm | `#F4F5F8` | chips, cards |
| surfaceMute | `#EEEFF3` | inset wells, tracks |
| ink | `#0B0E16` | primary text |
| ink2 | `#3F4554` | secondary text |
| ink3 | `#5B6373` | tertiary |
| muted | `#828A99` | labels, meta |
| dim | `#B0B6C2` | faint |
| border | `rgba(15,18,28,.07)` | hairline |
| borderStrong | `rgba(15,18,28,.14)` | stronger hairline |
| accent / ai | `#7B5CF0` | focus ring, AI hue |
| ok | `#1F8A5B` | accepted/verified |
| warn | `#B26314` | pending/retry |
| bad | `#B8281C` | rejected/blocked |
| focus | `#7B5CF0` | keyboard focus outline |

### Colors — Dark
| token | value |
|---|---|
| bg | `#0A0C13` |
| surface | `#13171F` |
| surfaceWarm | `#181C25` |
| surfaceMute | `#1E232E` |
| ink | `#EBEDF2` · ink2 `#BFC4D0` · ink3 `#8B92A0` · muted `#666E80` · dim `#404654` |
| border | `rgba(235,237,242,.08)` · borderStrong `rgba(235,237,242,.18)` |
| accent / ai | `#A89BFF` |
| ok | `#5DCE96` · warn `#F3B564` · bad `#F37A6B` |
| focus | `#A89BFF` |

### Iridescent (brand signature)
- **Outline ring** (conic, masked to a 1.25px border): stops
  `#C2A2FF, #93D7F5, #5BE1A8, #FFE49B, #FF99D6, #C2A2FF`, animated start *angle*.
- **Text gradient — light:** `linear-gradient(95deg,#6B3FE0,#1D88C7,#1E8F5A,#C13B7C,#6B3FE0)`
  (saturated so it stays legible on white).
- **Text gradient — dark:** `linear-gradient(95deg,#C2A2FF,#93D7F5,#5BE1A8,#FF99D6,#C2A2FF)`.
- **Background blobs:** pastel pink/peach/mint/sky radial gradients, blurred 60px,
  opacity ~.55 (light) / ~.35 (dark).

### Per-author tint (group-chat color coding)
Derived from each participant's `hue`: bubble `hsl(h, 70%/35%, 95%/14%)`,
border `hsl(h, 55%/40%, 82%/28%)`, name `hsl(h, 60%, 38%/70%)` (light/dark).
Rin = hue 16, Jules = hue 152, You = hue 220, AI = hue 268 (rendered iridescent, not hsl).

### Typography
- **Body / UI:** Geist (400/500/600/700). Sans, `-0.012em` letter-spacing on titles.
- **Mono / diagnostic:** JetBrains Mono (400/500), with `ss01`,`cv11` features. Used for
  data-strip chips, reason labels, timestamps, rc= codes, composer prefix, room meta.
- Sizes: bubble text 12.5–14.5px; titles 16–18px; chips/diagnostics 10.5–11.5px;
  kickers 9.5–10px uppercase, ~1.1–1.3 letter-spacing.

### Radii / shadow / spacing
- Radii: bubbles 12–18 (with one corner reduced to 4–6), chips/pills 7, panels 12–16,
  buttons 8–10, avatars 50%.
- Panel shadow: `0 24px 64px rgba(0,0,0,.32)`. Bubble shadow (light): `0 6px 18px
  rgba(0,0,0,.05)`. Tooltip: `0 8px 24px rgba(0,0,0,.12)`.
- Spacing: 6/8/10/12/14/16/18px rhythm; thread gap 6–12px; panel section gap 14–16px.

### Iconography
All icons are inline SVG (no icon font): helix (inspector/identity mark), gear
(settings), rooms (sidebar rect), search, shield (trust), lock (composer-blocked /
bootstrap), arrow-up (send), sync/refresh (empty state), `✕` (close), `›` `▸` `▾`
chevrons. Traffic-light dots for the macOS frame. No emoji.

---

## Assets

No external image/font binaries are required by the source. Fonts come from Google
Fonts (Geist, JetBrains Mono) via `@import` in component CSS and a `<link>` in the
host (the standalone export inlines them). The bundler thumbnail is an inline SVG helix.
Replace device frames (`ios-frame.jsx`, `macos-window.jsx`) with the target platform's
real chrome — they are prototype scaffolding, not part of the product UI.

---

## Files

Recreate from these (in dependency order):

| File | Role |
|---|---|
| `30_UI_INTEGRATION_REPORT.md` | **Behavior source of truth.** Decision contract, state mappings, sample payloads, integration rules, test scenarios. Read first. |
| `mercury-core.jsx` | The decision simulator: `PlatformDecisionView` builders, `useMercuryThread` hook, sample people, reason dictionary. **This is the interface to replicate against the real binding.** |
| `mercury-variant-fusion.jsx` | The main UI: rail, header + data strip, thread, message bubbles, composer, inspector, theme palettes, tooltips, focus styles. |
| `mercury-variant-fusion-panels.jsx` | All overlay panels: shell, trust, AI grant, peers, encryption, bootstrap, profile, recovery, sync, notifications, settings. |
| `Mercury UI.html` | Host: wires the design-canvas, device frames, tweak controls (scenario knobs), theme resolution (`useResolvedDark`). |
| `ios-frame.jsx`, `macos-window.jsx`, `design-canvas.jsx`, `tweaks-panel.jsx` | **Prototype scaffolding only** — device bezels, the side-by-side canvas, and the scenario-tweak panel. Not part of the product; do not port. |
| `Mercury-UI-standalone.html` | Self-contained offline build for reference viewing (fonts embedded). Open in a browser to interact with the prototype. |

### How to explore the prototype
Open `Mercury-UI-standalone.html` in a browser. Use the **Tweaks** panel (top-right) to
drive the scenario knobs (bootstrap / trust / AI / receive mode / sender trust / forced
send outcome / theme) and watch every surface and the inspector respond. This is the
fastest way to see all states the UI must handle.
