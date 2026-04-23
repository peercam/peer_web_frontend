# Mock Peer Backend (Rust)

A lightweight mock GraphQL server that simulates registration, authentication, session management, user profiles, posts, comments, and chat for offline Leptos development and testing.

## Quick Start

```bash
cd packages/mock_backend
cargo run
```

The server starts at **http://localhost:4000/graphql**.

## Run Tests

```bash
cargo test
```

Runs 170 integration tests covering all mutations, queries, auth flows, and edge cases.

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/graphql` | POST | GraphQL queries/mutations |
| `/reset` | POST | Reset state for test isolation |

## Supported Operations

### Queries
- `_health: Boolean`
- `listComments(postid: ID!, contentFilterBy: String, commentOffset: Int, commentLimit: Int): CommentListResponse!`
- `listChildComments(parent: ID!, offset: Int, limit: Int): CommentListResponse!`
- `listChats(limit: Int, offset: Int): ListChatsResponse!`

### Mutations — Registration (Phase 0)
- `verifyReferralString(referralString: String!): ReferralResponse!`
- `register(input: RegistrationInput!): RegisterResponse!`
- `verifyAccount(userid: ID!): VerifyAccountResponse!`

### Mutations — Auth & Session (Phase 1)
- `login(email: String!, password: String!): AuthPayload!`
- `refreshToken(refreshToken: String!): AuthPayload!`
- `logout(refreshToken: String!): LogoutPayload!`
- `deleteAccount(password: String!): DefaultResponse!` *(requires auth)*
- `updatePassword(password: String!, expassword: String!): DefaultResponse!` *(requires auth)*
- `requestPasswordReset(email: String!): ResetPasswordRequestResponse!`
- `resetPasswordTokenVerify(token: String!): DefaultResponse!`
- `resetPassword(token: String!, password: String!): DefaultResponse!`
- `contactus(name: String!, email: String!, message: String!): ContactusResponse!`

### Mutations — Comments (Phase 4)
- `createComment(action: CommentType!, postid: ID!, content: String!, parentid: ID): CreateCommentResponse!` *(requires auth)*
- `likeComment(commentid: ID!): DefaultResponse!` *(requires auth)*
- `unlikeComment(commentid: ID!): DefaultResponse!` *(requires auth)*
- `reportComment(commentid: ID!): DefaultResponse!` *(requires auth)*

### Mutations — Chat (Phase 4)
- `createChat(input: CreateChatInput!): CreateChatResponse!` *(requires auth)*
- `sendChatMessage(chatid: ID!, content: String!): SendMessageResponse!` *(requires auth)*

## Authentication

Protected mutations require an `Authorization: Bearer <token>` header. Obtain tokens by calling `login`. The mock validates tokens by map lookup (no JWT signing).

```bash
# Login to get tokens
curl -s http://localhost:4000/graphql -X POST \
  -H 'Content-Type: application/json' \
  -d '{"query":"mutation { login(email: \"test@peer.com\", password: \"TestPass123\") { accessToken refreshToken } }"}'

# Use access token for protected mutations
curl -s http://localhost:4000/graphql -X POST \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <accessToken>' \
  -d '{"query":"mutation { deleteAccount(password: \"TestPass123\") { status ResponseCode } }"}'
```

## Seed Data

### Known Referral Codes

| UUID | Note |
|------|------|
| `85d5f836-b1f5-4c4e-9381-1b058e13df93` | Primary test referral |
| `a1b2c3d4-e5f6-7890-abcd-ef1234567890` | Secondary test referral |

### Seeded Users

| Email | Password | Username | Verified | UUID |
|-------|----------|----------|----------|------|
| `test@peer.com` | `TestPass123` | peerTester | ✅ Yes | `00000000-0000-4000-a000-000000000001` |
| `unverified@peer.com` | `TestPass456` | newSignup | ❌ No | `00000000-0000-4000-a000-000000000002` |

### Special Email Conventions

- Any email starting with `fail@` triggers a simulated internal server error (`40601`).

### Seed Comments (Phase 4)

| UUID | Post | Author | Type |
|------|------|--------|------|
| `20000000-...-000001` | Post 1 | alice | Top-level |
| `20000000-...-000002` | Post 1 | peerTester | Top-level |
| `20000000-...-000003` | Post 3 | peerTester | Top-level |
| `20000000-...-000004` | Post 5 | alice | Top-level |
| `20000000-...-000005` | Post 1 | peerTester | Reply to comment 1 |
| `20000000-...-000006` | Post 3 | alice | Reply to comment 3 |

Pre-populated likes: peerTester→comment 4, alice→comment 3.

### Seed Chats (Phase 4)

| UUID | Type | Participants |
|------|------|-------------|
| `30000000-...-000001` | Private | peerTester, alice |
| `30000000-...-000002` | Group ("Rust Developers") | peerTester, alice, bob |

5 seed messages: 3 in private chat, 2 in group chat.

## Notes

- **In-memory state:** Restarting the server or calling `POST /reset` resets all state (registered emails, verified users, tokens, passwords, deleted users) back to seed defaults.
- **CORS:** `Access-Control-Allow-Origin: *` is enabled for cross-origin requests from the Leptos dev server.
- **Port:** 4000 (configurable via `PORT` env var).
- **Library usage:** `mock_backend::app()` and `mock_backend::app_with_state()` are exported for in-process testing.
- **Token format:** `mock-access-<uid>-<timestamp>-<seq>` / `mock-refresh-<uid>-<timestamp>-<seq>` (deterministic, no crypto).
