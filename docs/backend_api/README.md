# Peer Backend API — Detailed Documentation

This directory contains the full specification of the Peer Backend GraphQL API, split by domain.

For a high-level overview, see [api.md](api.md).

---

## Documents

| # | Document | Description |
|---|----------|-------------|
| 01 | [Authentication & Account](01-authentication-and-account.md) | Registration, login, JWT tokens, password reset, account deletion, RBAC, response structure |
| 02 | [Users & Profiles](02-users-and-profiles.md) | User search, profiles, preferences, follow/block, referrals |
| 03 | [Posts & Content](03-posts-and-content.md) | Post creation, listing, interactions (like/dislike/view/save/share/report), tags, file uploads |
| 04 | [Comments](04-comments.md) | Comment creation, replies, likes, reports, nesting rules |
| 05 | [Wallet & Transfers](05-wallet-and-transfers.md) | Token balance, P2P transfers, fee structure, transaction history |
| 06 | [Tokenomics, Gems & Minting](06-tokenomics-gems-and-minting.md) | Action prices, daily free actions, gems system, token minting, bridge schema |
| 07 | [Advertisements](07-advertisements.md) | Basic and pinned ads, ad history, ad listing |
| 08 | [Shop](08-shop.md) | Shop purchases, order details, delivery |
| 09 | [Moderation](09-moderation.md) | Moderation dashboard, ticket actions, content visibility states |
| 10 | [Admin](10-admin.md) | Admin user search, leaderboard, friendship graph, admin extensions |

---

## Quick Reference

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/graphql` | Main GraphQL endpoint |
| `GET` | `/health` | Health check |
| `POST` | `/upload-post` | Multipart file upload (see [Posts & Content](03-posts-and-content.md#file-upload-endpoint)) |

### Schema Files

| Schema | Role | File |
|--------|------|------|
| Guest | No auth | `src/Graphql/schema/schemaguest.graphql` |
| Authenticated | User (0) | `src/Graphql/schema/schema.graphql` |
| Admin | Admin (16) | `src/Graphql/schema/admin_schema.graphql` |
| Moderator | Moderator (256) | `src/Graphql/schema/moderator_schema.graphql` |
| Bridge | Web3 Bridge (8) | `src/Graphql/schema/bridge_schema.graphql` |
| Shared types | All | `src/Graphql/schema/types/*.graphql` |

### Authentication

All authenticated requests require:
```http
Authorization: Bearer <accessToken>
Content-Type: application/json
```
