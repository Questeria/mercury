# Running the live two-user demo

This runs **real end-to-end-encrypted messaging** between two users in the browser: two
`mercury-app-server` dev shims (each holding its own identity + ratchet) talk through one
`mercury-relay`, and the React UI (`ui/app`) drives them over the `messaging.ts` binding. No
simulator — actual seal → relay → open.

> **Dev-only.** The `mercury-app-server` shim holds message keys and binds loopback only; the
> permissive CORS it serves is for the Vite dev origin. Production exposure is Tauri in-process
> IPC (same-origin, no open socket), driving the very same `AppController::handle_command`
> surface.

## 1. Start the backend (three terminals)

```sh
# Terminal 1 — the relay (opaque ciphertext router; binds 127.0.0.1:8787)
cargo run -p mercury-relay --bin mercury-relay-server

# Terminal 2 — Alice's app shim on :7879 (talks to the relay above)
cargo run -p mercury-app-server -- 7879

# Terminal 3 — Bob's app shim on :7880
cargo run -p mercury-app-server -- 7880
```

(Both shims default to `MERCURY_RELAY_URL=http://127.0.0.1:8787`.)

## 2. Start the UI (fourth terminal)

```sh
cd ui/app
npm install      # first time only
npm run dev      # Vite dev server on http://localhost:5173
```

## 3. Open two browser tabs

The UI runs LIVE when a backend is given via the `?backend=` query param (otherwise it shows the
decision-view simulator demo):

- **Alice:** http://localhost:5173/?backend=http://127.0.0.1:7879
- **Bob:** http://localhost:5173/?backend=http://127.0.0.1:7880

In each tab the top-right chip shows that user's **account id** (click to copy). In Alice's tab,
paste Bob's id into "Connect to a peer" and connect, then send a message. In Bob's tab, connect
to Alice's id — the message appears, decrypted. Reply, and it shows up for Alice. The UI never
holds keys or ciphertext; it only ever sees the plaintext you send/receive.

## No-browser smoke test

The same flow over `curl` (proves the stack without the UI), once the relay + both shims are up:

```sh
A=http://127.0.0.1:7879 ; B=http://127.0.0.1:7880
cmd() { curl -s -X POST "$1/command" -H 'content-type: application/json' -d "$2"; }

# identities
AID=$(cmd $A '{"cmd":"account_id"}' | python -c "import sys,json;print(json.load(sys.stdin)['result']['account_id'])")
BID=$(cmd $B '{"cmd":"account_id"}' | python -c "import sys,json;print(json.load(sys.stdin)['result']['account_id'])")

# publish cards, then Alice -> Bob
cmd $A '{"cmd":"publish_self"}' ; cmd $B '{"cmd":"publish_self"}'
cmd $A "{\"cmd\":\"add_contact\",\"account_id\":\"$BID\"}"
cmd $A "{\"cmd\":\"send\",\"peer\":\"$BID\",\"text\":\"hello from Alice\"}"

# Bob receives + decrypts
cmd $B '{"cmd":"poll"}'
# -> {"ok":true,"result":{"messages":[{"direction":"incoming","peer":"<alice>","seq":0,"text":"hello from Alice"}]}}
```

(Use ASCII in a Windows console — the console codepage can mangle multi-byte UTF-8 before `curl`
sends it; the shim then correctly rejects the non-UTF-8 body. A real browser sends proper UTF-8,
so emoji work in the UI.)
