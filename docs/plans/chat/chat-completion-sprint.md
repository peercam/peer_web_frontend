# Chat — Completion Sprint Plan

**Feature:** Chat (#8)
**Priority:** Next — closes the three unchecked client scope items on the 🟡 Chat row and retires the ❌ Firebase API row in favour of "Real-time chat transport: Polling (v1)" per the [ADR](../../adr-chat-realtime-transport.md)
**Status:** 🟡 Core implemented — polling transport, unread, and search remain
**Created:** 2026-04-21
**Plan Quality Target:** ⭐⭐⭐⭐⭐

---

## Summary

Chat shipped its core UI and GraphQL layer (page + 7 components + 151L API + state module + 851L SCSS) on the 🟡 Core Implemented tier. The remaining gaps are the three items still unchecked in the parent plan's scope list:

1. **Real-time plumbing** — polling transport against GraphQL per the [ADR](../../adr-chat-realtime-transport.md), visibility-aware, with optimistic-send dedup
2. **Unread message indicators** — both chat-list badge and global nav badge
3. **Chat search / filter logic** — the input and tabs exist in `chat_list.rs`, but the filter function is a stub

Secondary polish carried over from the parent plan's "not yet implemented" list:

4. **Connection-lost banner** wired to the polling error channel
5. **Send-failure retry affordance** (already have toast; need a "Retry" button on the failed bubble)
6. **E2E coverage** for the behaviours above

**Verdict (revised 2026-04-21):** The **mock backend** has full GraphQL persistence (Phase 4 ✅). The **real `peer_backend`** does **not** — see [Blocker Resolution](#blocker-resolution-2026-04-21) below, and the accepted [ADR](../../adr-chat-realtime-transport.md). This sprint therefore splits into three tracks:

- **Track C (client, in-sprint):** polling-based eventual consistency, unread, search, retry, banner. Shippable against the mock today.
- **Track A (backend persistence, prerequisite for prod, out-of-sprint):** `sendChatMessage` / `createChat` / `listChats` / `listChatMessages` / `markChatRead` resolvers + mappers against the existing Postgres `chats` / `chatmessages` / `chatparticipants` tables. Tracked in a sibling plan (TBD: `track-a-backend-persistence.md`); named owner required before Chat can be promoted to ✅ end-to-end.
- **Track B (sub-second push, future, out-of-sprint):** GraphQL subscriptions are the preferred upgrade path per the ADR. Not implemented in this sprint. A separate plan will land when product demand justifies the work.

Finishing Track C promotes the **client** to eventually-consistent real-time and closes the three unchecked client scope items in the parent plan. It does **not** by itself flip the feature-level Chat row to ✅ in [feature-convergence.md](../../feature-convergence.md) — that requires Track A. The "Firebase" row is retired in favour of a "Real-time chat transport" row set to "Polling (v1)" per the ADR.

---

## Blocker Resolution (2026-04-21)

The original kickoff blocker asked whether the production backend mirrors `sendChatMessage` writes to Firestore. Investigation of the `peer_backend` repo produced the following evidence:

| Check | Result |
|-------|--------|
| Firestore / Firebase Admin SDK in `composer.json` | ❌ Absent (only `firebase/php-jwt`, unrelated) |
| Repo-wide search for `firestore`, `google/cloud`, `kreait`, `firebase-admin` | 0 matches |
| Service-account credentials / Firestore config | None present |
| `sendChatMessage` / `createChat` / chat resolver in `src/` | Not found |
| GraphQL chat schema / resolver files | None |
| Chat-adjacent code | Only `ValidateChatMessages` / `ValidateChatStructure` input filters (not wired to any mutation) |
| Postgres schema (`001_structure.sql`) | ✅ `chats`, `chatmessages`, `chatparticipants` tables exist but are unused by application code |
| `json/response-codes-editable.json` | Contains chat success/error copy → feature was stubbed, not implemented |

**Decision matrix:**

| Option | Viable? | Notes |
|---|---|---|
| A. Backend writes directly to Firestore on `sendChatMessage` | ❌ Not today | Would require new SDK dependency, service-account provisioning, and a writer. The mutation itself does not exist. |
| **B. Backend writes only to Postgres; optional out-of-process mirror to Firestore** | ✅ **Chosen** | Schema already exists; mirror (if ever needed) is a separate workstream. |
| C. Dual-write from PHP to both stores | ⚠️ Rejected | No legacy Firestore readers found in this repo; dual-write adds a consistency failure mode for no user-visible benefit. |

**Resolution:** proceed with **Option B**. Legacy `js/firebase_config.js` / `js/chat/loader.js` describe an aspirational client that never had a live backend write path — citations to "legacy Firestore patterns" elsewhere in this plan are design inspiration, not lived architecture. Full reasoning lives in [docs/adr-chat-realtime-transport.md](../../adr-chat-realtime-transport.md).

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page** | [src/pages/chat.rs](../../../src/pages/chat.rs) | 141 | ✅ Auth guard, layout, contacts overlay orchestration |
| **Route** | [src/app.rs](../../../src/app.rs) | — | ✅ `/chat` registered |
| **State** | [src/state/chat.rs](../../../src/state/chat.rs) | — | ✅ `ChatContext` — chat list, active chat, messages, current user signals |
| **API — list_chats** | [src/api/chat.rs](../../../src/api/chat.rs) | — | ✅ Server fn → `LIST_CHATS_QUERY` |
| **API — send_message** | [src/api/chat.rs](../../../src/api/chat.rs) | — | ✅ Server fn → `SEND_MESSAGE_MUTATION` |
| **API — create_chat** | [src/api/chat.rs](../../../src/api/chat.rs) | — | ✅ Server fn → `CREATE_CHAT_MUTATION` |
| **Models** | [src/models/chat.rs](../../../src/models/chat.rs) | — | ✅ `Chat`, `ChatMessage`, `ChatParticipant`, `ChatType`, `format_message_time` |
| **Component — chat_list** | [src/components/chat/chat_list.rs](../../../src/components/chat/chat_list.rs) | ~220 | ✅ Private/Group tabs, [+] button, skeleton states — **search input rendered but not filtering** |
| **Component — chat_item** | [src/components/chat/chat_item.rs](../../../src/components/chat/chat_item.rs) | — | ✅ Avatar, display name, last message preview, relative time — **no unread badge** |
| **Component — chat_container** | [src/components/chat/chat_container.rs](../../../src/components/chat/chat_container.rs) | ~85 | ✅ Header + messages + input composition |
| **Component — chat_messages** | [src/components/chat/chat_messages.rs](../../../src/components/chat/chat_messages.rs) | ~80 | ✅ Bubble rendering, auto-scroll anchor |
| **Component — chat_input** | [src/components/chat/chat_input.rs](../../../src/components/chat/chat_input.rs) | ~124 | ✅ 500-char limit, Enter-to-send, optimistic append |
| **Component — contacts_overlay** | [src/components/chat/contacts_overlay.rs](../../../src/components/chat/contacts_overlay.rs) | — | ✅ Friend list, single/multi select |
| **Component — group_review** | [src/components/chat/group_review.rs](../../../src/components/chat/group_review.rs) | — | ✅ Name + image step |
| **GraphQL** | [src/api/graphql.rs](../../../src/api/graphql.rs) | — | ✅ `LIST_CHATS_QUERY`, `SEND_MESSAGE_MUTATION`, `CREATE_CHAT_MUTATION` |
| **SCSS** | [style/chat.scss](../../../style/chat.scss) | 851 | ✅ Desktop + mobile layout |
| **Mock Backend** | [packages/mock_backend](../../../packages/mock_backend) | — | ✅ Phase 4 done — `listChats`, `createChat`, `sendChatMessage` + 49 tests |

### What Remains 🔲

| # | Task | File(s) | Effort | Blocks |
|---|------|---------|--------|--------|
| 1 | **Polling transport** (visibility-aware intervals for chat list and active chat; `listChatMessages(since)` query added to mock + client) | `src/state/chat.rs`, `src/api/chat.rs`, `src/api/graphql.rs`, `packages/mock_backend/src/schema/query/chat.rs` | M | Tasks 2–5 |
| 2 | **Unread count signal + badge rendering** (per-chat badge + global nav badge) | `src/state/chat.rs`, `src/components/chat/chat_item.rs`, `src/components/chat/chat_list.rs`, `src/components/widgets/main_menu.rs` | M | — |
| 3 | **Mark-as-read on chat open** (localStorage + server mutation) | `src/state/chat.rs`, `src/api/chat.rs` (new `markChatRead` server fn), mock backend | S | Task 2 correctness |
| 4 | **Chat search wiring** (filter `Chat` list by display name, last message, participant username — case-insensitive) | `src/components/chat/chat_list.rs`, `src/state/chat.rs` | S | — |
| 5 | **Connection-lost banner** (polling error → banner + "Retry now" CTA; `Degraded` strip explaining polling cadence) | `src/components/chat/chat_container.rs`, `src/state/chat.rs` | S | — |
| 6 | **Send-failure retry** (persist failed messages with `status: Failed`, render with retry button, re-enqueue on click; backoff on repeat failure) | `src/components/chat/chat_input.rs`, `src/components/chat/chat_messages.rs`, `src/state/chat.rs` | M | — |
| 7 | **E2E tests** (Playwright: send-then-poll delivery between two browsers, unread badge, polling-error banner, retry, search filter) | `end2end/tests/chat.spec.ts` | L | — |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

**Critical path:** 1 → 2 → 3 → 4 → 5 → 6 → 7

**Deferred (not in this sprint — see [ADR](../../adr-chat-realtime-transport.md)):** Firebase JS-interop bootstrap, Firestore `onSnapshot` listeners, mock Firestore SSE fakes, in-process broadcast hook. These were drafted in an earlier revision of this plan; they remain useful as design references if a sub-second push transport is ever needed, but Track B (GraphQL subscriptions) is the preferred upgrade path and will land under its own plan.

---

## Task Details

### Task 1 — Polling Transport (Primary)

**Goal:** Achieve eventual consistency via periodic `list_chats` re-fetches and, for the active chat, a new `listChatMessages` query. Per the [ADR](../../adr-chat-realtime-transport.md), polling is the **primary** client transport for v1 — not a fallback.

> **Track A dependency.** `listChatMessages` must be added to **both** the mock backend **and** `peer_backend`. The client will work against the mock today; shipping to prod requires the real resolver.

**Steps:**

1. Add `LIST_CHAT_MESSAGES_QUERY` to `graphql.rs` and `list_chat_messages(chat_id, since: Option<String>)` server fn in `api/chat.rs`.
2. Add the same resolver to the mock backend (trivial — it already stores `chat_messages` per chat).
3. In `ChatContext`, on mount, start two `leptos::leptos_dom::helpers::set_interval` timers:
   - **15s** for the chat list (`list_chats`) — keeps sidebar order and previews fresh. Chosen as a balance between responsiveness for non-active conversations and backend load; with N active users this is N/15 QPS against `list_chats`. Expose as `CHAT_LIST_POLL_INTERVAL` (env-overridable) so ops can tune it without a redeploy.
   - **5s** for the active chat (`list_chat_messages(chat_id, since)`) — gives conversational latency within the "feels live" band (users typically tolerate up to ~3s; 5s keeps us close without flooding). Exposed as `CHAT_ACTIVE_POLL_INTERVAL`.
4. Stop timers on unmount.

**`since` semantics:** the server treats `since` as **exclusive** (`created_at > since`) for efficiency, and the client tracks `newest_seen = max(existing)` and applies **client-side dedup by id** on every poll response. Using `>=` would double-fetch the most recent row; using `>` without client-dedup loses messages on clock skew. Do both.

**Optimistic → canonical id swap** (client-side dedup):

1. `chat_input` assembles an optimistic `ChatMessage` with `id = format!("tmp:{}", Uuid::new_v4())` and `status = Sending`, appends it to `messages`.
2. `send_message` server fn returns the canonical `ChatMessage { id, created_at, ... }`. On success, replace the entry whose id starts with `tmp:` and whose `(sender_id, content, created_at_client)` matches — overwrite `id` and set `status = Sent`. Do **not** remove the bubble; the next poll for that id will then dedupe in step (4).
3. On `send_message` failure, transition the optimistic entry to `status = Failed` (Task 6).
4. Poll response merge rules, in order:
   - If any existing entry has `id == msg.id` → update fields in place (content may have been normalized server-side) and return.
   - If any entry has `id.starts_with("tmp:")` AND `sender_id == msg.sender_id` AND `content == msg.content` AND `(msg.created_at - entry.created_at_client).abs() < 10s` → treat as the canonical version of an in-flight optimistic send, swap the id, return.
   - Else → append and sort by `created_at`.
   - After append, if `msg.sender_id != current_user_id` → call `maybe_increment_unread(msg)` (Task 2).

**Visibility pause:** subscribe to `visibilitychange`; when `document.visibilityState === "hidden"`, pause both intervals. Resume + do one immediate catch-up fetch on `visible`.

**Error handling:** any poll error → bump `connection_state` (Task 5). Successful poll → `Connected`.

**Done when:** two browsers can see each other's messages within ≤ 5s (active chat) and ≤ 15s (sidebar); backgrounded tabs stop polling within 1s of visibility change; no duplicate bubbles from the sender's optimistic append; a forced 500 on `list_chat_messages` trips the banner, and recovery clears it.

---

### Deferred Firebase tasks (design reference, not in this sprint)

The sections that previously described **Firebase bootstrap**, **chat-level `onSnapshot` listener**, **chat-list `onSnapshot` listener**, **mock Firestore SSE endpoints**, and the **in-process broadcast hook** have been removed from the critical path per the [ADR](../../adr-chat-realtime-transport.md). They remain useful input if the GraphQL-subscriptions upgrade in Track B is ever revisited in favour of a third-party transport; until then, consult git history (`git log -- docs/plans/chat/chat-completion-sprint.md`) for the full drafts.

Short form of what was there:

- **Firebase bootstrap** \u2014 hydrate-only dynamic `import()`, `<meta name="firebase-config">` populated from SSR env vars, idempotent `firebase::init()`. Public web API keys are not secrets; GCP service-account keys must never be exposed.
- **`onSnapshot` listeners** \u2014 one per active chat's `messages` subcollection, one global on the user's `chats` collection. `StoredValue<Option<FirestoreUnsubscribe>>` + `on_cleanup` to avoid leaks. Dedup against optimistic sends using the Task 1 id-swap rules.
- **Mock Firestore SSE fake** \u2014 `GET /mock/firestore/messages/:chatId` + `GET /mock/firestore/chats`, gated on `MOCK_FIREBASE=1`, with `?fail=once\|always` injection. `MockState::broadcast` called from `sendChatMessage` / `createChat`.

If any of this is ever needed, promote it into a fresh plan rather than resurrecting the inline drafts.

---

### Task 2 — Unread Count Signal + Badges

**Goal:** Show per-chat unread counts in the sidebar and a single aggregate badge in the global nav.

**Target files for the global badge:** the primary nav lives in [src/components/widgets/main_menu.rs](../../../src/components/widgets/main_menu.rs) (mounted into every page's right sidebar, including `/chat`). The badge renders next to the Chat menu item there.



**Goal:** Show per-chat unread counts in the sidebar and a single aggregate badge in the global nav.

**Target files for the global badge:** the primary nav lives in [src/components/widgets/main_menu.rs](../../../src/components/widgets/main_menu.rs) (mounted into every page's right sidebar, including `/chat`). The badge renders next to the Chat menu item there.

**Design:**

- Add `unread_counts: RwSignal<HashMap<String, u32>>` and `last_read_at: RwSignal<HashMap<String, DateTime<Utc>>>` to `ChatContext`.
- Persistence: mirror `last_read_at` per chat in `localStorage` as `chat:last-read:<chatId> = <ISO timestamp>` — survives reloads, doesn't need a backend round-trip for reads.
- **`maybe_increment_unread(msg)` is the single entry point** (called from Task 1's poll-merge path). Logic, in strict order:
  1. If `msg.sender_id == current_user_id` → return (never unread your own messages).
  2. Read `last_read = last_read_at.get(msg.chat_id).unwrap_or(DateTime::UNIX_EPOCH)`.
  3. If `msg.created_at <= last_read` → return (already read on another device or prior session).
  4. Increment `unread_counts[msg.chat_id]`.
- **Chat-open ordering fixes the race:** when `active_chat_id` changes to X and the tab is visible, run synchronously **before** any in-flight poll handlers for X can append:
  1. `last_read_at.update(|m| m.insert(X, Utc::now()))` — bumps the high-water mark first.
  2. `unread_counts.update(|m| m.insert(X, 0))`.
  3. `localStorage.setItem("chat:last-read:" + X, now.to_rfc3339())`.
  4. Fire-and-forget `mark_chat_read(X)` server fn (Task 3).
  Because step 1 lands before any subsequent `maybe_increment_unread`, a message that arrives mid-open is correctly suppressed by rule 3.
- **Tab-hidden case:** `document.hidden` is **not** a suppression condition. Unread is strictly a function of `last_read_at`. Opening a chat while the tab is backgrounded (e.g. via deep link from a push notification) still clears the badge.
- In `chat_item.rs`: conditionally render `<span class="unread-badge" aria-label="{n} unread messages">{n}</span>` when `n > 0`; cap display at "99+" (with `aria-label="99 or more unread messages"` when capped, so screen readers don't announce "ninety-nine plus").
- In the header/nav: derive `total_unread = unread_counts.values().sum()` and render a single badge with the same a11y treatment.

**Done when:** Receiving a message in a non-active chat increments that chat's badge; opening it clears the badge; reloading the page preserves the cleared state.

---

### Task 3 — Mark-as-Read Persistence

**Goal:** Back the localStorage last-read timestamp with a server mutation so the count is correct across devices.

> **Track A dependency.** `markChatRead` must be implemented in **both** the mock backend **and** `peer_backend` (resolver + mapper writing to a new `chat_last_read` table or a `last_read_at` column on `chatparticipants`). Out-of-scope for this client sprint; listed here so the two tracks stay coordinated.

**Steps:**

1. Add `MARK_CHAT_READ_MUTATION` to `graphql.rs` → `mutation { markChatRead(chatid: ID!) { meta { ResponseCode } } }`.
2. Add `mark_chat_read(chat_id)` server fn to `api/chat.rs`.
3. Add the resolver to the mock backend: record `last_read_at[(user_id, chat_id)] = now()` in `MockState`.
4. Amend `listChats` in the mock to include `unreadCount` **and** `lastReadAt` computed from server-side `last_read_at` — frontend uses both on first load to seed `unread_counts` and reconcile with localStorage (see reconciliation rule below).
5. Extend `Chat` model with `unread_count: u32` and `last_read_at: Option<DateTime<Utc>>` (default 0 / None via serde) — backward-compatible.

**First-load reconciliation** (resolves cross-device races):

```
effective_last_read = max(
    localStorage["chat:last-read:" + chat_id],
    chat.last_read_at_from_server,
)
write back to both localStorage and last_read_at signal
```

If `effective_last_read > chat.last_read_at_from_server`, fire `mark_chat_read(chat_id)` once to heal the server. `unread_counts` is then recomputed from whatever cached messages exist; the seeded `chat.unread_count` is only used when no local messages are available yet.

**Done when:** Seed data includes a user with 2 unread messages; on login, the sidebar badge shows 2 before any Firestore events arrive.

---

### Task 4 — Chat Search

**Goal:** Filter the sidebar list by the search input's text against display name, participant usernames, and last-message preview.

**Steps:**

1. Promote the local `search_query` signal in `chat_list.rs` to `ChatContext::search_query` so it can participate in the polling/listener flow.
2. Derive a `filtered_chats` memo: if query is empty → all chats (respecting active tab); else → case-insensitive substring match against `chat.display_name(uid)`, every `p.username` in `chatparticipants`, and `chat.last_message`.
3. Render empty state ("No chats match '<query>'") when the filter yields zero results but the unfiltered list is non-empty.

**Done when:** Typing "ali" filters the sidebar to chats containing "alice", "kali", etc.; clearing the box restores the full list; empty-state copy appears correctly.

---

### Task 5 — Connection-Lost Banner

**Goal:** Surface transport failures to the user with a retry affordance.

**Design:**

- Add `connection_state: RwSignal<ConnectionState>` to `ChatContext` with variants `Connected | Lost`.
- Consecutive poll failures (≥ 2 in a row) → `Lost`. One-off transient failures do not flip state, to avoid banner flicker.
- Successful poll → `Connected`.
- In `chat_container.rs`, render a sticky top banner when `Lost`: "Connection lost. Messages may not be up to date. [Retry now]". Clicking Retry fires an immediate out-of-band poll; success restores `Connected`.
- Always render a subtle, dismissible info strip on first mount explaining the cadence: "Messages refresh every ~5s while a chat is open." (Shown once per session; suppressed via `sessionStorage` after dismissal.)

**Done when:** Killing the network in DevTools shows the banner within ~10s (two missed 5s polls); restoring + clicking Retry clears it.

---

### Task 6 — Send-Failure Retry

**Goal:** Failed sends stay visible and retryable instead of silently vanishing.

**Design:**

- Extend the optimistic `ChatMessage` with a non-serialised `status: MessageStatus` field: `Sending | Sent | Failed`.
- On `send_message` success → perform the Task 1 id-swap in place (tmp id → canonical id, status → `Sent`). Do **not** remove and re-add; the next poll will dedupe by the swapped id.
- On `send_message` failure → transition status to `Failed` **and** fire the existing error toast. Track `retry_count` on the bubble (non-serialised, starts at 0).
- In `chat_messages.rs`, render a small "Retry" button + exclamation icon on `Failed` bubbles.
- **Retry click handler:** transition the *same* bubble back to `status = Sending` (keep the tmp id), re-invoke `send_message` with the same content. On success, same id-swap as the original send. On failure, increment `retry_count` and return to `Failed`.
- **Repeat-failure backoff:** after `retry_count ≥ 2`, the Retry button stays visible but is disabled for a cool-off window (`min(2^retry_count, 30)s`, capped); a small hint reads "Try again in Ns". This prevents mash-retry storms when the backend is down, without ever removing the user's ability to try. No automatic background retries — user is always in control.
- `Failed` bubbles are right-aligned like `Sent` but styled with `opacity: 0.6` and a red dot — spec this in the SCSS.

**Done when:** Toggling network off mid-send leaves the message visible with a Retry button; clicking Retry with network restored delivers it; a second immediate failure disables Retry for the backoff window.

---

### Task 7 — E2E Tests

**File:** `end2end/tests/chat.spec.ts`

**Scenarios:**

1. **polled-delivery** — two browser contexts (userA, userB), both on `/chat` with chat between them active for both, userA sends "hello", assert userB's active view shows the bubble within ≤ 6s (one active-chat poll cycle + slack) and userB's sidebar preview updates within ≤ 16s.
2. **unread-badge** — userB on `/chat` but chat is not active; userA sends → userB's chat item shows badge "1" after at most one list-poll cycle; userB clicks chat → badge disappears; reload → still no badge.
3. **polling-error-banner** — intercept `listChatMessages` to return 500 twice in a row; assert the banner renders; release the intercept; click **Retry now**; assert banner clears.
4. **retry-failed-send** — intercept the `sendChatMessage` network request once to return 500; assert bubble shows Retry; release the intercept; click Retry; assert success.
5. **search-filter** — three seeded chats (alice, bob, alice-and-bob-group); typing "ali" shows 2, typing "xyz" shows empty state, clearing shows 3.

**CI integration:** runs against the mock backend. No real Firebase project involved.

---

## Mock Backend Additions

All additions are small — Phase 4 already did the heavy lifting for persistence.

| Operation | Type | Auth | Task |
|-----------|------|------|------|
| `listChatMessages(chatid, since)` | Query | Yes | 1 |
| `markChatRead(chatid)` | Mutation | Yes | 3 |
| `Chat.unreadCount` + `Chat.lastReadAt` fields | Type extension | — | 3 |

**Test count delta:** +6 integration tests (2 for `listChatMessages`, 2 for `markChatRead`, 2 for `unreadCount` / `lastReadAt` in `listChats`).

---

## Out of Scope

Explicitly **not** in this sprint (already "Out of Scope" in the parent plan — listed here only to prevent scope creep):

- Media messages (images, audio, video) — future feature
- Message reactions/emojis
- Message editing/deletion
- Typing indicators
- Read receipts (per-recipient, distinct from unread counts)
- Push notifications (requires FCM + service worker extension)
- Group admin features
- Voice/video calls

---

## Security & Privacy Considerations

| Concern | Mitigation |
|---------|------------|
| **Cross-user message read** | GraphQL resolvers must enforce `current_user ∈ chat.participants` on `listChats`, `listChatMessages`, `sendChatMessage`, `markChatRead`. Called out as a Track A acceptance criterion. |
| **Unread count leak** | `markChatRead` mutation checks the caller is a participant before writing. |
| **XSS in message content** | Leptos `view!` escapes by default; no `inner_html` / `dangerous_set_inner_html` added. |
| **CSRF on new mutations** | Same JWT-in-cookie + `SameSite=Lax` posture as existing mutations — no new surface. |
| **Polling amplification (DoS vector)** | Intervals are env-configurable; visibility-pause reduces idle load; `since`-exclusive keeps payloads bounded. Server should rate-limit `list_chat_messages` per-user if abuse is observed. |

---

## Definition of Done

### Client sprint DoD (Track C — in-scope)

- [ ] Task 1: Polling transport — visibility-aware, configurable intervals, optimistic id-swap, no duplicate bubbles, verified across two browsers against the mock
- [ ] Task 2: Unread badges — per-chat + global nav (via `main_menu.rs`), "99+" cap, localStorage persistence
- [ ] Task 3: `markChatRead` server fn + mock resolver + seeded test data; first-load reconciliation healing stale local state
- [ ] Task 4: Search filters sidebar; empty-state copy present
- [ ] Task 5: Connection-lost banner + Retry-now CTA; debounced (≥ 2 consecutive failures) to avoid flicker
- [ ] Task 6: Failed bubbles render Retry button; retry re-sends with the same content; repeat-failure backoff in effect
- [ ] Task 7: 5 new Playwright specs pass in CI against the mock
- [ ] Parent plan [chat-implementation.md](chat-implementation.md) — three unchecked **client** scope items (🔲 Real-time plumbing, 🔲 Unread, 🔲 Search) marked [x]
- [ ] Parent plan status reflects client-side completion (e.g. 🟡 Client Complete / Backend Pending)
- [ ] `cargo build --features ssr` + `cargo build --features hydrate --target wasm32-unknown-unknown` both clean
- [ ] `cargo clippy --all-targets --features ssr -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `cargo test -p mock_backend` passes — target: 272 total (266 baseline + 6 new)
- [ ] Unit test: `MessageStatus::Failed` bubble renders Retry button (component-level test in `chat_messages.rs`)

### Feature-level DoD (requires Track A — out-of-scope for this sprint)

- [ ] **Track A:** `sendChatMessage`, `createChat`, `listChats`, `listChatMessages`, `markChatRead` implemented in `peer_backend` against Postgres; integration tests against the real backend. Tracked in a sibling plan (TBD: `track-a-backend-persistence.md`) — named owner required.
- [ ] Parent plan [chat-implementation.md](chat-implementation.md) status promoted to ✅ Implemented
- [ ] [feature-convergence.md](../../feature-convergence.md) updated:
  - Pages table: Chat row → ✅ Implemented; "Gaps" line removed
  - API Layer table: "Firebase" row retired and replaced with "Real-time chat transport → Polling (v1)" per the ADR
  - Summary counts and convergence % recalculated
  - Migration priority #8 marked complete
  - Changelog entry under today's date listing all new/modified files

### Track B DoD (future — GraphQL subscriptions upgrade)

Not in this sprint. Captured in the ADR; a dedicated plan will define DoD when the work is scheduled.

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Optimistic-vs-poll duplicate bubbles | High | Low | Two-layer dedup: (a) `sendChatMessage` response swaps `tmp:<uuid>` → canonical id in place; (b) poll-merge also matches by `(sender_id, content, created_at ± 10s)` as a backstop if a poll lands before the mutation response. See Task 1 "Optimistic → canonical id swap". |
| Polling stampedes if user leaves tab open for hours | Low | Low | `document.visibilityState === "hidden"` → pause polling; resume on `visibilitychange` |
| Poll backend load at scale | Medium | Medium | Intervals env-configurable (`CHAT_LIST_POLL_INTERVAL`, `CHAT_ACTIVE_POLL_INTERVAL`); `since`-exclusive semantics keep active-chat payloads small; visibility-pause reduces idle load. |
| localStorage quota exceeded | Negligible | Negligible | Only one tiny timestamp per chat; bounded |
| Track A slips, blocking prod ship | Medium | High | Track C is already shippable against the mock; dev can proceed. Production blocker escalated in feature-level DoD — do not ship until Track A owner is named and resolvers land. |

---

## Open Questions

1. **Should unread counts survive `deleteAccount`?** Current design ties them to `user_id`; if the account is recreated with the same id the counts would come back. Recommend clearing `last_read_at` rows on delete — trivial addition to the Phase 1 delete resolver.
2. **Character limit** — parent plan says 500; verify against the backend mutation validator (`ValidateChatMessages`). If the backend allows more, raise client limit to match or document the intentional client-side cap.
3. **Icon for Retry** — reuse `svg/refresh.svg` or introduce a new one? Prefer reuse.

---

## Appendix A — Why Chat Is the Right "Next"

- **Client feature closes:** Chat's three unchecked scope items (real-time plumbing, unread, search) all land in one sprint.
- **Mock backend is ready:** Phase 4 persistence is done; client development is not blocked.
- **User-visible impact:** messaging is table stakes; the current 🟡 state is the most obviously incomplete feature to an end user.
- **Risk is bounded:** all Track C work is pure Rust/signal + mock additions. No new JS-interop surface, no new infra.
- **Clears the path for Track A:** a well-defined client contract (including `listChatMessages` and `markChatRead`) gives the backend team a concrete target.

---

## Appendix B — File Manifest (Estimated)

**New files:**

- `end2end/tests/chat.spec.ts` (~180L)

**Modified files:**

- `src/state/chat.rs` (+~130L — polling timers, unread, connection state, search)
- `src/api/chat.rs` (+~40L — `mark_chat_read`, `list_chat_messages`)
- `src/api/graphql.rs` (+~15L — two new operation constants)
- `src/models/chat.rs` (+~10L — `unread_count`, `MessageStatus`)
- `src/components/chat/chat_list.rs` (+~30L — search filter, context wiring)
- `src/components/chat/chat_item.rs` (+~15L — unread badge)
- `src/components/chat/chat_container.rs` (+~25L — connection banner)
- `src/components/chat/chat_input.rs` (+~20L — retry plumbing)
- `src/components/chat/chat_messages.rs` (+~25L — failed-bubble retry button)
- `src/components/widgets/main_menu.rs` (+~15L — global unread badge)
- `src/pages/chat.rs` (+~5L — start polling on mount)
- `style/chat.scss` (+~80L — badge, banner, failed bubble)
- `packages/mock_backend/src/state.rs` (+~15L — `last_read_at`, `chat_unread_count`)
- `packages/mock_backend/src/schema/mutation/chat.rs` (+~40L — `markChatRead`)
- `packages/mock_backend/src/schema/query/chat.rs` (+~40L — `listChatMessages`, `unreadCount` field)
- `packages/mock_backend/src/seed.rs` (+~10L — seeded unread state)
- `packages/mock_backend/tests/chat.rs` (+~90L — 6 new tests)

**Docs updated:**

- `docs/plans/chat/chat-implementation.md` (scope checkboxes, status)
- `docs/feature-convergence.md` (summary, Pages row, API row, priority, changelog)

---

*End of sprint plan.*
