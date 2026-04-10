# Shop API

## Overview

The Shop domain allows users to purchase physical merchandise using Peer tokens. Orders include delivery details and are processed through the token transfer system. Currently supports delivery to Germany only.

**Authentication**: Required for all operations.

---

## Queries

### `shopOrderDetails`

Retrieve details of a shop order by its transaction ID.

```graphql
query {
  shopOrderDetails(transactionId: String!): ShopOrderDetailsResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `transactionId` | `String!` | Yes | Transaction ID from the purchase |

#### Access Control
- The order owner can view their own order details
- The `PEER_SHOP` account (role bitmask 32) can view all orders

#### Response: `ShopOrderDetailsResponse`

```graphql
type ShopOrderDetailsResponse {
  meta: DefaultResponse!
  affectedRows: ShopOrderDetails!
}

type ShopOrderDetails {
  shopOrderId: String!
  shopItemId: String!
  shopItemSpecs: ShopItemSpecs
  deliveryDetails: ShopOrderDeliveryDetails!
  createdat: Date!
}

type ShopItemSpecs {
  size: String!
}

type ShopOrderDeliveryDetails {
  name: String!
  email: String!
  addressline1: String!
  addressline2: String
  city: String!
  zipcode: String!
  country: String!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12202` | Order details retrieved |
| `22101` | Order not found |
| `30101` | Missing transaction ID |
| `60501` | Not authenticated or not authorized |

---

## Mutations

### `performShopOrder`

Purchase a shop item using tokens.

```graphql
mutation {
  performShopOrder(
    tokenAmount: Decimal!
    shopItemId: String!
    orderDetails: ShopOrderDetailsInput!
  ): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tokenAmount` | `Decimal!` | Yes | Token amount for the purchase |
| `shopItemId` | `String!` | Yes | Identifier of the shop item |
| `orderDetails` | `ShopOrderDetailsInput!` | Yes | Delivery and item details |

#### Input: `ShopOrderDetailsInput`

```graphql
input ShopOrderDetailsInput {
  name: String!                                    # 2–100 characters
  email: String!                                   # Valid email
  addressline1: String!                            # 6–100 characters
  addressline2: String                             # Optional
  city: String!                                    # 2–100 characters
  zipcode: String!                                 # Exactly 5 characters
  country: ShopSupportedDeliveryCountry!           # Currently: GERMANY only
  shopItemSpecs: ShopItemSpecsInput                # Optional item specifications
}

input ShopItemSpecsInput {
  size: String                                     # 1–100 characters
}

enum ShopSupportedDeliveryCountry {
  GERMANY
}
```

#### Validation Rules

| Field | Constraint |
|-------|-----------|
| `name` | 2–100 characters |
| `email` | Valid email format |
| `addressline1` | 6–100 characters |
| `city` | 2–100 characters |
| `zipcode` | Exactly 5 characters |
| `country` | Must be `GERMANY` |
| `size` (if provided) | 1–100 characters |

#### Business Rules
- User must not be a system account
- User must have sufficient token balance to cover the amount
- Tokens are transferred to the `PEER_SHOP` account
- A `ShopOrder` record is created with all delivery details
- Transaction is wrapped in a database transaction for rollback safety

#### Side Effects
- Deducts `tokenAmount` from user's wallet
- Credits `tokenAmount` to the Peer Shop account
- Creates shop order record linked to the transaction
- Transaction category: `SHOP_PURCHASE`

#### Response Codes

| Code | Description |
|------|-------------|
| `12201` | Order placed successfully |
| `30101` | Missing required fields |
| `31202` | Insufficient balance |
| `31205` | System user cannot place orders |
| `41223` | Order creation failed |
| `51301` | Insufficient balance |
| `60501` | Not authenticated |
