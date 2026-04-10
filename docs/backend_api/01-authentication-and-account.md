# Authentication & Account Management API

## Overview

Authentication is handled via JWT (RS256) tokens. All guest operations (registration, login, password reset) use the **guest schema** (`schemaguest.graphql`) and require no authentication. Once authenticated, tokens are used for all subsequent requests.

### Schema Access
- **Guest schema**: No authentication required
- **Authenticated schema**: Requires valid `Authorization: Bearer <accessToken>` header

---

## Token Lifecycle

| Token | Purpose | Signing Algorithm |
|-------|---------|-------------------|
| Access Token | Authorize API requests | RS256 |
| Refresh Token | Obtain new access/refresh token pair | RS256 |

### JWT Payload Structure
```json
{
  "iss": "<configured issuer>",
  "aud": "<configured audience>",
  "rol": "<user role bitmask>",
  "uid": "<user UUID>",
  "iat": "<issued at timestamp>",
  "exp": "<expiration timestamp>"
}
```

Token expiry is configured via environment variables:
- `TOKEN_EXPIRY` — access token validity in seconds (default: 3600)
- `REFRESH_TOKEN_EXPIRY` — refresh token validity in seconds (default: 604800)

---

## Mutations

### `register`

Creates a new user account. Requires email verification before login.

```graphql
mutation {
  register(input: RegistrationInput!): RegisterResponse!
}
```

#### Input: `RegistrationInput`

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `email` | `String!` | Yes | Valid email format |
| `password` | `String!` | Yes | 8–128 chars, must contain uppercase, lowercase, and digit (`^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).+$`) |
| `username` | `String!` | Yes | 3–23 chars, alphanumeric/underscores/hyphens (`^[a-zA-Z0-9_-]+$`) |
| `pkey` | `String` | No | Optional Solana public key (43–44 chars, base58) |
| `referralUuid` | `ID` | No | UUID of the referring user |

#### Response: `RegisterResponse`

```graphql
type RegisterResponse {
  meta: DefaultResponse!
  status: String!          # @deprecated — use meta.status
  ResponseCode: String     # @deprecated — use meta.ResponseCode
  userid: ID               # UUID of created user (on success)
}
```

#### Side Effects
- Creates user record with generated UUID and random 5-digit slug
- Creates associated records: `users_info`, `wallett`, `userpreferences`, `contactinfo`
- If `referralUuid` provided, creates referral relationship
- All operations wrapped in a database transaction

#### Response Codes

| Code | Description |
|------|-------------|
| `10601` | Registration successful |
| `30301` | Missing required fields |
| `30601` | Email already taken |
| `31007` | Invalid referral UUID (user not found) |
| `30202` | Invalid username format |
| `30103` | Invalid input format |

---

### `verifyAccount`

Activates a user account after registration.

```graphql
mutation {
  verifyAccount(userid: ID!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `userid` | `ID!` | Yes | Valid UUID |

#### Response Codes

| Code | Description |
|------|-------------|
| `10701` | Account verified successfully |
| `30101` | Missing userid field |
| `30201` | Invalid UUID format |
| `30701` | Account already verified |
| `31007` | User not found |
| `40701` | Internal server error |

---

### `login`

Authenticates a user and returns JWT tokens.

```graphql
mutation {
  login(email: String!, password: String!): AuthPayload!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `email` | `String!` | Yes |
| `password` | `String!` | Yes |

#### Response: `AuthPayload`

```graphql
type AuthPayload {
  meta: DefaultResponse!
  status: String!           # @deprecated — use meta.status
  ResponseCode: String      # @deprecated — use meta.ResponseCode
  accessToken: String       # JWT access token (on success)
  refreshToken: String      # JWT refresh token (on success)
}
```

#### Validation
- Checks user exists and is verified
- Verifies password hash
- Checks account is not deleted (status ≠ 6) or suspended

#### Response Codes

| Code | Description |
|------|-------------|
| `10801` | Login successful |
| `30801` | Invalid credentials (missing fields, bad email, wrong password, or deleted account) |
| `60801` | Account not verified |
| `40801` | Internal server error |

---

### `refreshToken`

Exchanges a valid refresh token for a new token pair.

```graphql
mutation {
  refreshToken(refreshToken: String!): AuthPayload!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `refreshToken` | `String!` | Yes |

#### Response: `AuthPayload`

Returns new `accessToken` and `refreshToken` on success.

#### Validation
- Validates refresh token signature and expiration
- Verifies associated user still exists and is active

#### Response Codes

| Code | Description |
|------|-------------|
| `10901` | Token refreshed successfully |
| `30101` | Missing refresh token |
| `30901` | Invalid, expired, or revoked refresh token |
| `40901` | Internal server error |

---

### `requestPasswordReset`

Initiates a password reset flow by sending an email with a reset token.

```graphql
mutation {
  requestPasswordReset(email: String!): ResetPasswordRequestResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `email` | `String!` | Yes |

#### Response: `ResetPasswordRequestResponse`

```graphql
type ResetPasswordRequestResponse {
  meta: DefaultResponse!
  status: String!           # @deprecated — use meta.status
  ResponseCode: String      # @deprecated — use meta.ResponseCode
  nextAttemptAt: String     # Timestamp when next attempt is allowed
}
```

#### Rate Limiting
- 1st attempt: 60-second cooldown
- 2nd attempt: 600-second cooldown
- 3rd+ attempt: locked (requires support)

#### Response Codes

| Code | Description |
|------|-------------|
| `11901` | Email sent (if account exists) |
| `30104` | Invalid email format |

> **Security Note**: Always returns success response regardless of whether the email exists, preventing user enumeration.

---

### `resetPasswordTokenVerify`

Validates a password reset token.

```graphql
mutation {
  resetPasswordTokenVerify(token: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `token` | `String!` | Yes |

#### Response Codes

| Code | Description |
|------|-------------|
| `11902` | Token is valid |
| `31904` | Invalid or expired reset token |
| `41004` | Internal server error |

---

### `resetPassword`

Sets a new password using a valid reset token.

```graphql
mutation {
  resetPassword(token: String!, password: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `token` | `String!` | Yes | Valid reset token |
| `password` | `String!` | Yes | 8–128 chars, uppercase + lowercase + digit |

#### Side Effects
- Updates password hash
- Deletes stored access and refresh tokens from the database

> **Note**: Server-side token revocation depends on DB token validation being enabled. See [Session Management Caveats](#session-management-caveats) below.

#### Response Codes

| Code | Description |
|------|-------------|
| `11005` | Password updated successfully |
| `31904` | Invalid or expired reset token |
| `21001` | No user found for token (silent success) |
| `41004` | Internal server error |

---

### `deleteAccount`

Soft-deletes the authenticated user's account (sets status to 6).

```graphql
mutation {
  deleteAccount(password: String!): DefaultResponse!
}
```

**Requires authentication.**

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `password` | `String!` | Yes |

#### Validation
- Verifies password matches current account password

#### Response Codes

| Code | Description |
|------|-------------|
| `11012` | Account deleted successfully |
| `60501` | Not authenticated |
| `30101` | Missing password field |
| `31001` | Password does not match |
| `21001` | User not found (silent success) |
| `41011` | Internal server error |

---

### `contactus`

Sends a contact form message (guest-accessible).

```graphql
mutation {
  contactus(name: String!, email: String!, message: String!): ContactusResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `name` | `String!` | Yes | 3–53 characters |
| `email` | `String!` | Yes | Valid email format |
| `message` | `String!` | Yes | 3–500 characters |

#### Response: `ContactusResponse`

```graphql
type ContactusResponse {
  meta: DefaultResponse!
  status: String!           # @deprecated — use meta.status
  ResponseCode: String      # @deprecated — use meta.ResponseCode
  affectedRows: ContactusResponsePayload
}

type ContactusResponsePayload {
  msgid: Decimal
  email: String
  name: String
  message: String
  ip: String
  createdat: String
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `10401` | Message sent successfully |
| `30101` | Missing required fields |

---

### `verifyReferralString`

Validates a referral link string and returns the referrer's info (guest-accessible).

```graphql
mutation {
  verifyReferralString(referralString: String!): ReferralResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `referralString` | `String!` | Yes |

#### Response: `ReferralResponse`

```graphql
type ReferralResponse {
  meta: DefaultResponse!
  affectedRows: ReferralInfo
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11011` | Referral info retrieved successfully |
| `31007` | No valid referral information found |
| `41013` | Error retrieving referral info |

---

## Queries

### `guestListPost`

Returns a single post's data without authentication (guest-accessible).

```graphql
query {
  guestListPost(postid: ID!): PostResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `postid` | `ID!` | Yes |

---

### `hello`

Returns system information. Available in both guest and authenticated schemas.

```graphql
query {
  hello: HelloResponse
}
```

#### Response: `HelloResponse`

```graphql
type HelloResponse {
  currentuserid: ID             # Current user's UUID (null for guests)
  currentVersion: String        # API version
  wikiLink: String              # Documentation link
  lastMergedPullRequestNumber: String
  companyAccountId: ID          # Peer company account UUID
  # Admin/Moderator-only (extended via admin_schema/moderator_schema):
  userroles: Int                # User role bitmask
  userRoleString: String        # Human-readable role string
}
```

---

## Role-Based Access Control

User roles are stored as a bitmask in `roles_mask`:

| Role | Bitmask | Schema Access |
|------|---------|---------------|
| `USER` | `0` | `schema.graphql` (base authenticated) |
| `SYSTEM_ACCOUNT` | `1` | LP Account (internal) |
| `COMPANY_ACCOUNT` | `2` | Peer Bank Account (internal) |
| `BURN_ACCOUNT` | `4` | Token burn operations (internal) |
| `WEB3_BRIDGE_USER` | `8` | `bridge_schema.graphql` only |
| `ADMIN` | `16` | `schema.graphql` + `admin_schema.graphql` |
| `PEER_SHOP` | `32` | Shop operations (internal) |
| `MODERATOR` | `256` | `schema.graphql` + `moderator_schema.graphql` |

Schema loading per role:
- **Guest** (no token) → `schemaguest.graphql`
- **User** (role 0) → `schema.graphql`
- **Admin** (role 16) → `schema.graphql` + `admin_schema.graphql`
- **Moderator** (role 256) → `schema.graphql` + `moderator_schema.graphql`
- **Bridge** (role 8) → `bridge_schema.graphql`

---

## Standard Response Structure

All responses include a `meta` object (type `DefaultResponse`):

```graphql
type DefaultResponse {
  status: String!           # "success" or "error"
  RequestId: String!        # Unique request UUID for tracing
  ResponseCode: String!     # Numeric code (see response code tables)
  ResponseMessage: String!  # Human-readable description
}
```

### Response Code Ranges

| Range | Category |
|-------|----------|
| `10xxx` | Success |
| `11xxx` | Success with data |
| `12xxx` | Success (commerce/special) |
| `21xxx` | Success but no data found |
| `30xxx` | Input validation error |
| `31xxx` | Business logic error |
| `32xxx` | State conflict error |
| `40xxx` | Server/database error |
| `41xxx` | Operation failed |
| `51xxx` | Insufficient resources |
| `60xxx` | Authorization error |
| `62xxx` | Moderation authorization error |

---

## Session Management Caveats

> **Important**: Server-side token revocation is currently **disabled**. The DB-level checks that validate whether an access token or refresh token still exists in the database are commented out in the codebase (`accessTokenValidForUser`, `refreshTokenValidForUser`). This means:
> - Tokens deleted from the DB (e.g. after `resetPassword`) remain valid until their JWT expiration time.
> - There is no way to force-logout a user before their access token expires.
> - The `refreshToken` mutation does not verify the old refresh token exists in DB before issuing new tokens.
>
> Until this is re-enabled, token expiry (`TOKEN_EXPIRY`, `REFRESH_TOKEN_EXPIRY`) is the only revocation mechanism.

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | All GraphQL responses (errors encoded in response body) |
| `400` | Malformed request (invalid JSON, missing query) |
| `401` | Invalid or expired access token |
| `429` | Rate limit exceeded |
