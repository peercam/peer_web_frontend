# ADR: Chat Real-Time Transport

**Status:** Accepted
**Date:** 2026-04-21
**Deciders:** Chat sprint team
**Related:** [docs/plans/chat/chat-completion-sprint.md](plans/chat/chat-completion-sprint.md), [docs/plans/chat/chat-implementation.md](plans/chat/chat-implementation.md), [docs/adr-mock-backend-rust-rewrite.md](adr-mock-backend-rust-rewrite.md)

---

## Context

The Chat feature in `peer-web` was planned around Firebase Firestore as its real-time transport, following the pattern of the legacy PHP/JS client (`js/chat/*.js`, `js/firebase_config.js`). The chat completion sprint paused at a kickoff blocker: does the production backend mirror `sendChatMessage` writes into Firestore?

A repo-wide audit of `peer_backend` on 2026-04-21 established:

- No Firestore / Firebase Admin SDK is present (`composer.json` ships only `firebase/php-jwt`, which is JWT-only and unrelated).
- A case-insensitive search for `firestore`, `google/cloud`, `kreait`, `firebase-admin` across the repo returns **zero matches**.
- No service-account credentials, Firestore config, or env vars exist.
- No `sendChatMessage` / `createChat` / `listChats` resolver, mapper, or GraphQL schema for chat exists in `src/`. Chat is **not implemented** on the backend.
- The Postgres schema (`chats`, `chatmessages`, `chatparticipants`) is provisioned in `001_structure.sql` but unused by application code.
- Input filters (`ValidateChatMessages`, `ValidateChatStructure`) and response-code copy exist, confirming the feature was stubbed and never finished.

Conclusion: there is no legacy production write path to Firestore in this codebase. Any such flow, if it ever existed, lives outside `peer_backend`.

## Decision

**Adopt Option B: Postgres as the canonical write path; real-time transport is client-only, polling-first.**

1. **Persistence:** `sendChatMessage` / `createChat` / `listChats` / `listChatMessages` / `markChatRead` will be implemented as GraphQL operations in `peer_backend` writing to the existing Postgres tables. This is tracked as **Track A** of the chat completion sprint.
2. **Real-time transport (v1):** the client uses **polling** against the GraphQL backend (15s for the chat list, 5s for the active chat; visibility-aware). Polling is the **primary** transport, not a fallback.
3. **Real-time transport (future — preferred upgrade):** **GraphQL subscriptions** served by `peer_backend` over WebSocket (`graphql-ws`) or SSE (`graphql-sse`). This keeps the client on a single protocol (GraphQL) for queries, mutations, **and** push, and aligns with the mock backend's `async-graphql` stack, which supports subscriptions natively. Alternative upgrade paths — a bespoke WS/SSE endpoint or an out-of-process Firestore mirror fed from a Postgres change stream / outbox — remain on the table but are lower-preference (see below).
4. **No dual-write.** The PHP/Rust request path will not write to both Postgres and Firestore.

## Rejected alternatives

### Option A — Backend writes directly to Firestore on `sendChatMessage`

Rejected. Requires adding a Firestore SDK (e.g. `google/cloud-firestore` + gRPC), provisioning service-account credentials, and writing a new mapper — all before the mutation itself exists. Couples the request path to a vendor and a second data store with no compensating user benefit.

### Option C — Dual-write from the backend to Postgres **and** Firestore

Rejected. Adds a consistency failure mode (partial writes, drift) for a benefit that does not materialise: there are no legacy Firestore-reading clients in production. Only worth reconsidering if such clients are discovered; none were found in this repo.

### Firestore-as-transport without backend mirror

Rejected for v1. Would mean the client listens to Firestore while the backend writes to Postgres, with nothing bridging them. This cannot work and is listed only to rule it out explicitly.

## Considered upgrade paths (post-v1)

When product demand justifies sub-second push, the following options will be evaluated. Ranking reflects current preference; it is not yet a decision.

### 1. GraphQL subscriptions (preferred)

Extend the existing GraphQL schema with `subscription { chatMessageAdded(chatid: ID!): ChatMessage }` and `subscription { chatUpdated: Chat }` (scoped to the authenticated user's participant set). Transport over `graphql-ws` (WebSocket) or `graphql-sse`.

**Why preferred:**

- **One protocol, one auth story.** The client already speaks GraphQL with a JWT-cookie auth posture; subscriptions inherit it unchanged.
- **Contract continuity.** `ChatMessage` / `Chat` types are shared between `list*` queries, `send*` mutations, and subscriptions — no parallel schema, no translation layer, no drift risk.
- **Mock parity is cheap.** `async-graphql` (the mock's stack) supports subscriptions first-class; the mock can publish on a `tokio::sync::broadcast` channel from the existing send resolver (this is exactly what Task 10b in the sprint already proposes, just exposed via GraphQL instead of a bespoke SSE route). The current `/mock/firestore/*` SSE design in Task 10a can be retired in favour of GraphQL-over-SSE, collapsing two endpoints into one.
- **Tooling.** Rust-side client libraries (`graphql-client`, `cynic`, `async-graphql` client helpers) handle framing and reconnect; we avoid hand-rolling `EventSource` dedup logic.
- **Backpressure & fan-out.** `async-graphql` + `tokio::sync::broadcast` gives lossy-on-slow-subscriber semantics out of the box, which matches chat's delivery requirements (clients reconcile via a catch-up query on reconnect).

**Trade-offs to resolve before adopting:**

- **Backend lift.** `peer_backend` must expose a subscription transport (WS or SSE endpoint) and a publish hook in the `sendChatMessage` / `createChat` resolvers. Scope: comparable to adding one new mutation plus connection plumbing.
- **Scaling model.** A single-process broadcast channel does not survive horizontal scaling; a multi-instance deployment needs a fan-out layer (Postgres `LISTEN/NOTIFY`, Redis pub/sub, or NATS) between the write path and the subscription resolver. Acceptable if single-instance is the v1.1 target; otherwise the fan-out must land first.
- **Connection management.** WebSocket upgrades require proxy/CDN awareness (idle timeouts, sticky sessions if state-bearing). SSE sidesteps the WS upgrade but inherits HTTP/2 stream-count limits in the browser. Either is tractable; both are friction vs polling.
- **Auth refresh.** Long-lived connections must handle JWT expiry — either via periodic `connection_init` re-auth messages (`graphql-ws` supports this) or by tearing down and reconnecting on 401. Needs a small protocol decision, not a research project.

### 2. Bespoke WS / SSE endpoint

A non-GraphQL push endpoint (e.g. `GET /events`). Simpler to stand up than a full subscription transport, but duplicates auth plumbing and creates a parallel schema for payloads. Lower preference than option 1 because the simplicity gain is small and the contract-drift cost is permanent.

### 3. Out-of-process Firestore mirror

Postgres → Firestore one-way replication (Cloud Function on a change stream, or an outbox worker), with the client reading Firestore directly for push. Lowest preference unless external consumers (mobile apps, third-party integrations) also need to read the mirror — in which case the mirror justifies its own existence and chat becomes a secondary consumer. Not worth building for chat alone.

## Consequences

### Positive

- **Unblocks the client sprint immediately.** Track C (listeners, unread, search, retry, banner) can proceed against the mock backend, which already encodes the contract.
- **Single source of truth.** Postgres is canonical; there is no replication lag to reason about.
- **No vendor lock-in.** The client's transport layer is abstracted behind `subscribe_messages` / `subscribe_chats` helpers; swapping polling for SSE or Firestore later is a localised change.
- **Security surface shrinks.** No Firestore rules to author, no service-account keys to rotate, no public web API keys to explain.

### Negative / costs

- **No sub-second push in v1.** Users see new messages in up to 5 seconds (active chat) or 15 seconds (sidebar). Acceptable for a social-app chat; not acceptable for a customer-support or trading use case.
- **Polling bandwidth.** Mitigated by visibility-pause and `since`-based incremental fetches. Expected cost is comparable to a heartbeat.
- **The "Firebase ❌ → ✅" headline in [feature-convergence.md](feature-convergence.md) no longer applies.** The API-layer row is renamed to "Real-time chat transport" and its status tracks the chosen transport, not a vendor.

### Neutral

- Legacy citations to `js/chat/loader.js` Firestore patterns remain useful as **design inspiration** for snapshot dedup rules, but are no longer evidence of a lived architecture.

## Compliance / rollout

- Client code MUST go through the `utils::firebase::*` (to be renamed to `utils::chat_transport::*`) helpers; direct `fetch` loops in components are not allowed.
- `listChatMessages` MUST support a `since` parameter (exclusive) for incremental polling; servers MAY ignore it only if they always return the full window.
- Any future Firestore mirror MUST be unidirectional (Postgres → Firestore) and MUST NOT be on the user-request path.

## Open items

- Name an owner for Track A (`peer_backend` persistence resolvers).
- Decide whether `last_read_at` is stored on `chatparticipants` or in a new `chat_last_read` table — tracked with Track A, not in this ADR.
- Revisit transport choice once persistence ships and real latency data is available.
- **GraphQL subscriptions pre-work (non-blocking for v1):** confirm that `peer_backend`'s GraphQL stack can host a subscription transport without a major framework change. If it can, prototype `chatMessageAdded` behind a feature flag during Track A so the upgrade is a configuration flip rather than a second project.
- Decide the multi-instance fan-out mechanism (Postgres `LISTEN/NOTIFY` vs Redis vs NATS) **before** subscriptions are promoted from experimental to primary transport.
