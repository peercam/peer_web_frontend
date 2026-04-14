# Mock Peer Backend (Rust)

A lightweight mock GraphQL server that simulates registration, authentication, session management, and account operations for offline Leptos development and testing.

## Quick Start

```bash
cd tests/mock_backend
cargo run
```

The server starts at **http://localhost:4000/graphql**.

## Run Tests

```bash
cargo test
```

Runs 33 integration tests covering all mutations, queries, auth flows, and edge cases.

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/graphql` | POST | GraphQL queries/mutations |
| `/reset` | POST | Reset state for test isolation |

## Supported Operations

### Queries
- `_health: Boolean`

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

## Notes

- **In-memory state:** Restarting the server or calling `POST /reset` resets all state (registered emails, verified users, tokens, passwords, deleted users) back to seed defaults.
- **CORS:** `Access-Control-Allow-Origin: *` is enabled for cross-origin requests from the Leptos dev server.
- **Port:** 4000 (configurable via `PORT` env var).
- **Library usage:** `mock_backend::app()` and `mock_backend::app_with_state()` are exported for in-process testing.
- **Token format:** `mock-access-<uid>-<timestamp>-<seq>` / `mock-refresh-<uid>-<timestamp>-<seq>` (deterministic, no crypto).
