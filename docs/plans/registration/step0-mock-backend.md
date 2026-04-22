# Step 0 — Mock Peer Backend

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Stand up a lightweight mock GraphQL server that faithfully simulates the three registration-related mutations — including their exact field names, response envelopes, and response codes — so that all subsequent Leptos steps can be developed and tested entirely offline.

---

## 0.1 — Mutations to mock (exact shapes from `js/register/register.js`)

### Mutation 1: `verifyReferralString`

```graphql
mutation VerifyReferralString($referralString: String!) {
  verifyReferralString(referralString: $referralString) {
    ResponseCode
    affectedRows {
      uid
      username
      slug
      img
    }
    status          # "success" | "error"
  }
}
```

| Scenario | `status` | `ResponseCode` | `affectedRows` |
|----------|----------|-----------------|-----------------|
| Valid UUID that exists | `"success"` | `"11011"` (Referral info fetched) | Array with one user object |
| Valid UUID format, not found | `"error"` | `"31010"` (Invalid referral string) | `null` |
| Malformed string (non-UUID) | `"error"` | `"31010"` | `null` |
| Internal server error path | `"error"` | `"41013"` | `null` |

### Mutation 2: `register`

```graphql
mutation Register($input: RegistrationInput!) {
  register(input: $input) {
    status          # "success" | "error"
    ResponseCode
    userid
  }
}
```

`RegistrationInput` fields (from the existing JS `variables.input`):

```graphql
input RegistrationInput {
  email: String!
  password: String!
  username: String!
  pkey: String          # nullable, always sent as null currently
  referralUuid: ID      # optional per API spec; JS always sends it
}
```

| Scenario | `status` | `ResponseCode` | `userid` |
|----------|----------|-----------------|----------|
| Success | `"success"` | `"10601"` | New UUID string |
| Email already registered | `"error"` | `"30601"` | `null` |
| Internal server error | `"error"` | `"40601"` | `null` |
| Failed to generate user ID | `"error"` | `"40602"` | `null` |

### Mutation 3: `verifyAccount`

```graphql
mutation VerifiedAccount($userId: ID!) {
  verifyAccount(userid: $userId) {
    status          # "success" | "error"
    ResponseCode
  }
}
```

| Scenario | `status` | `ResponseCode` |
|----------|----------|-----------------|
| Success | `"success"` | `"10701"` |
| Already verified | `"success"` | `"30701"` |
| Internal server error | `"error"` | `"40701"` |

---

## 0.2 — Directory structure

```
packages/mock_backend/
├── package.json
├── server.js              ← Express + express-graphql entry point
├── schema.graphql         ← Full type definitions & mutations
├── resolvers.js           ← Stateful resolver logic (in-memory store)
├── state.js               ← Shared in-memory state (registered emails, known referrals)
├── README.md              ← How to start, seed data, and test
└── fixtures/
    ├── referral_success.json
    ├── referral_invalid.json
    ├── register_success.json
    ├── register_duplicate_email.json
    ├── verify_success.json
    └── verify_already_verified.json
```

---

## 0.3 — `schema.graphql` (complete)

```graphql
type DefaultResponse {
  status: String!
  RequestId: String!
  ResponseCode: String!
  ResponseMessage: String!
}

type ReferralUser {
  uid: ID!
  username: String!
  slug: String!
  img: String
}

type ReferralResponse {
  status: String!            # @deprecated — use meta.status
  ResponseCode: String!      # @deprecated — use meta.ResponseCode
  affectedRows: [ReferralUser]
  meta: DefaultResponse!
}

input RegistrationInput {
  email: String!
  password: String!
  username: String!
  pkey: String
  referralUuid: ID           # optional per API spec
}

type RegisterResponse {
  status: String!            # @deprecated — use meta.status
  ResponseCode: String!      # @deprecated — use meta.ResponseCode
  userid: String
  meta: DefaultResponse!
}

type VerifyAccountResponse {
  status: String!            # @deprecated — use meta.status
  ResponseCode: String!      # @deprecated — use meta.ResponseCode
  meta: DefaultResponse!
}

type Mutation {
  verifyReferralString(referralString: String!): ReferralResponse!
  register(input: RegistrationInput!): RegisterResponse!
  verifyAccount(userid: ID!): VerifyAccountResponse!
}

# A minimal Query type is required by the spec even if unused
type Query {
  _health: Boolean
}
```

---

## 0.4 — Resolver logic (pseudocode)

```
state = {
  knownReferrals: Set<UUID>  — pre-seeded with 2-3 valid codes,
  registeredEmails: Set<String> — starts empty,
  verifiedUsers: Set<UUID> — starts empty,
}
```

### `verifyReferralString(referralString)`

1. Parse `referralString`. If it matches the UUID regex `/^[0-9a-f]{8}-…$/i` **and** is in `knownReferrals` → return fixture `referral_success.json`.
2. Otherwise → return fixture `referral_invalid.json` with `ResponseCode: "31010"`.

Seed codes (hard-coded in `state.js`):

```
85d5f836-b1f5-4c4e-9381-1b058e13df93   ← "primary test referral"
a1b2c3d4-e5f6-7890-abcd-ef1234567890   ← "secondary test referral"
```

### `register(input)`

1. If `input.email` is already in `registeredEmails` → return `ResponseCode: "30601"`, `status: "error"`.
2. If `input.email` starts with `fail@` → simulate internal error → `ResponseCode: "40601"`.
3. Otherwise → generate a random UUID as `userid`, add `input.email` to `registeredEmails`, return `ResponseCode: "10601"`, `status: "success"`, `userid`.

### `verifyAccount(userid)`

1. If `userid` is already in `verifiedUsers` → return `ResponseCode: "30701"` (already verified).
2. Otherwise → add to `verifiedUsers`, return `ResponseCode: "10701"`.

---

## 0.5 — Fixture files (exact JSON shapes)

**`fixtures/referral_success.json`**

```json
{
  "data": {
    "verifyReferralString": {
      "ResponseCode": "11011",
      "affectedRows": [
        {
          "uid": "usr_mock_001",
          "username": "peerTester",
          "slug": "peertester",
          "img": "https://via.placeholder.com/96"
        }
      ],
      "status": "success",
      "meta": {
        "status": "success",
        "RequestId": "mock-req-001",
        "ResponseCode": "11011",
        "ResponseMessage": "Referral info fetched successfully"
      }
    }
  }
}
```

**`fixtures/referral_invalid.json`**

```json
{
  "data": {
    "verifyReferralString": {
      "ResponseCode": "31010",
      "affectedRows": null,
      "status": "error",
      "meta": {
        "status": "error",
        "RequestId": "mock-req-002",
        "ResponseCode": "31010",
        "ResponseMessage": "Invalid referral string"
      }
    }
  }
}
```

**`fixtures/register_success.json`** (template — `userid` is replaced at runtime)

```json
{
  "data": {
    "register": {
      "status": "success",
      "ResponseCode": "10601",
      "userid": "{{GENERATED_UUID}}",
      "meta": {
        "status": "success",
        "RequestId": "mock-req-003",
        "ResponseCode": "10601",
        "ResponseMessage": "User registered successfully"
      }
    }
  }
}
```

**`fixtures/register_duplicate_email.json`**

```json
{
  "data": {
    "register": {
      "status": "error",
      "ResponseCode": "30601",
      "userid": null,
      "meta": {
        "status": "error",
        "RequestId": "mock-req-004",
        "ResponseCode": "30601",
        "ResponseMessage": "Email is already registered"
      }
    }
  }
}
```

**`fixtures/verify_success.json`**

```json
{
  "data": {
    "verifyAccount": {
      "status": "success",
      "ResponseCode": "10701",
      "meta": {
        "status": "success",
        "RequestId": "mock-req-005",
        "ResponseCode": "10701",
        "ResponseMessage": "Account verified successfully"
      }
    }
  }
}
```

**`fixtures/verify_already_verified.json`**

```json
{
  "data": {
    "verifyAccount": {
      "status": "success",
      "ResponseCode": "30701",
      "meta": {
        "status": "success",
        "RequestId": "mock-req-006",
        "ResponseCode": "30701",
        "ResponseMessage": "Account is already verified"
      }
    }
  }
}
```

---

## 0.6 — `package.json`

```json
{
  "name": "peer-mock-backend",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "start": "node server.js",
    "test": "node test.js"
  },
  "dependencies": {
    "express": "^4.21.0",
    "express-graphql": "^0.12.0",
    "graphql": "^16.9.0",
    "uuid": "^10.0.0"
  }
}
```

---

## 0.7 — Implementation sub-tasks (checklist)

- [ ] **0.7a** Create `packages/mock_backend/` directory and `package.json`.
- [ ] **0.7b** Write `schema.graphql` with the three mutations and all types as specified in §0.3.
- [ ] **0.7c** Write `state.js` — exports the in-memory sets with seed data.
- [ ] **0.7d** Write `resolvers.js` — implement the three resolver functions with the logic from §0.4.
- [ ] **0.7e** Write `server.js` — Express app that loads the schema, wires resolvers, and listens on port 4000.
- [ ] **0.7f** Create all six fixture JSON files in `fixtures/`.
- [ ] **0.7g** Write `test.js` — a lightweight smoke-test script that starts the server, sends the five curl-equivalent requests below, asserts status codes and response shapes, then exits.
- [ ] **0.7h** Write `README.md` with setup instructions (`npm install && npm start`), seed data reference, and example curl commands.
- [ ] **0.7i** Add `packages/mock_backend/node_modules/` to `.gitignore`.

---

## 0.8 — Verification test cases

After running `npm start` in `packages/mock_backend/`, each of these should pass:

| # | Test | curl command (abbreviated) | Expected response (key fields) |
|---|------|---------------------------|-------------------------------|
| 1 | Valid referral | `mutation { verifyReferralString(referralString: "85d5f836-...") { status ResponseCode affectedRows { uid username } } }` | `status: "success"`, `ResponseCode: "11011"`, `affectedRows` has 1 entry |
| 2 | Invalid referral | `verifyReferralString(referralString: "not-a-uuid")` | `status: "error"`, `ResponseCode: "31010"`, `affectedRows: null` |
| 3 | Register success | `register(input: { email: "new@test.com", password: "Abcd1234", username: "newuser", pkey: null, referralUuid: "85d5f836-..." })` | `status: "success"`, `ResponseCode: "10601"`, `userid` is a UUID |
| 4 | Duplicate email | Same register call with `email: "new@test.com"` again | `status: "error"`, `ResponseCode: "30601"`, `userid: null` |
| 5 | Verify account | `verifyAccount(userid: "<userid from test 3>")` | `status: "success"`, `ResponseCode: "10701"` |
| 6 | Already verified | Same `verifyAccount` call again | `status: "success"`, `ResponseCode: "30701"` |
| 7 | Simulated error | `register(input: { email: "fail@test.com", ... })` | `status: "error"`, `ResponseCode: "40601"` |

**Primary smoke-test command:**

```bash
cd packages/mock_backend && npm install && npm start &
sleep 2
curl -s -X POST http://localhost:4000/graphql \
  -H 'Content-Type: application/json' \
  -d '{"query":"mutation { verifyReferralString(referralString: \"85d5f836-b1f5-4c4e-9381-1b058e13df93\") { status ResponseCode affectedRows { uid username slug img } } }"}'
# Expected: {"data":{"verifyReferralString":{"status":"success","ResponseCode":"11011","affectedRows":[{"uid":"usr_mock_001","username":"peerTester","slug":"peertester","img":"https://via.placeholder.com/96"}]}}}
```

---

## 0.9 — Design decisions & notes

- **Why Express + express-graphql?** Minimal setup, widely understood, zero config. Apollo Server is an alternative but heavier for a mock.
- **Why in-memory state instead of a database?** The mock is ephemeral and test-only. Restarting the server resets all data, which is desirable for repeatable test runs.
- **`fail@` email convention:** Any email starting with `fail@` triggers the `40601` error path. This lets E2E tests exercise error handling without special flags.
- **CORS:** The mock server should add `Access-Control-Allow-Origin: *` headers so the Leptos dev server (port 3000) can call it directly during SSR and from the browser during CSR.
- **Port:** 4000 is chosen to avoid collision with the Leptos dev server (3000) and common dev tool ports.
- **No authentication on the mock.** The real `verifyReferralString` and `register` mutations don't require a JWT (they're pre-auth flows). The mock should accept requests with or without an `Authorization` header.
- **Fixture files serve as documentation.** Even though resolvers construct responses programmatically, the fixture JSONs act as the canonical reference for the response shapes that the Leptos `models/user.rs` types must deserialize.
