# Wallet — Completion Sprint Plan

**Feature:** Wallet (#9)
**Priority:** Closed — both unchecked client scope items on the 🟡 Wallet row in [feature-convergence.md](../../feature-convergence.md) are now ✅
**Status:** ✅ Complete — shop delivery panel wired, `format_balance()` groups thousands with 4dp rounding, Playwright `wallet.spec.ts` lands the lazy-load network assertion
**Created:** 2026-04-22
**Closed:** 2026-04-22
**Plan Quality Target:** ⭐⭐⭐⭐⭐

---

## Summary

Wallet shipped its core UI and GraphQL layer (146-line page + transfer modal (760L) + balance header + transaction history + transaction item + 246L API + 1,092L SCSS) on the 🟡 Implemented (tests pending) tier. Two gaps remain on the parent plan's scope checklist:

1. **Shop purchase order details (delivery info)** — the model (`ShopOrderDetails`, `DeliveryDetails`, `ShopItemSpecs`), GraphQL constant (`SHOP_ORDER_DETAILS_QUERY`), and server function (`get_shop_order_details`) all exist in the client, and the mock backend has a passing `shopOrderDetails` resolver (Phase 5 ✅). The lazy-load delivery panel inside the expanded transaction row is **not wired up**.
2. **Thousand-separator formatting on the balance display** — `format_balance()` in `models/transaction.rs` is documented as "thousand separators" but its body just calls `format_decimal()` (trim-trailing-zeros). Wallet balance, transaction amounts, and the transfer-modal summary all currently render without grouping commas, diverging from the legacy `formatAmount()` behaviour in `js/wallet.js`.

Secondary items carried over from the parent plan's "tests pending" note:

3. **E2E coverage** for the transfer modal happy path, balance reload, and the shop-order delivery panel.

Finishing this sprint promotes Wallet from 🟡 → ✅ in [feature-convergence.md](../../feature-convergence.md), bringing the convergence count to 13/20 (65%) on the Pages table (✅ 12 → 13, 🟡 6 → 5).

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page** | [src/pages/wallet.rs](../../..//src/pages/wallet.rs) | 146 | ✅ Auth guard, layout, balance + history composition |
| **Route** | [src/app.rs](../../..//src/app.rs) | — | ✅ `/wallet` registered |
| **Component — balance_header** | [src/components/wallet/balance_header.rs](../../..//src/components/wallet/balance_header.rs) | — | ✅ Animated logo, balance display, reload button — **uses `format_balance()` (currently no separators)** |
| **Component — transaction_history** | [src/components/wallet/transaction_history.rs](../../..//src/components/wallet/transaction_history.rs) | — | ✅ Infinite scroll via `use_infinite_scroll`, skeletons, empty state |
| **Component — transaction_item** | [src/components/wallet/transaction_item.rs](../../..//src/components/wallet/transaction_item.rs) | ~280 | ✅ Bubble + expanded `TransactionDetail` — **no `delivery_info_container`** |
| **Component — transfer_modal** | [src/components/wallet/transfer_modal.rs](../../..//src/components/wallet/transfer_modal.rs) | 760 | ✅ Friend list, search, amount + fees, message, summary, success/error |
| **API — wallet** | [src/api/wallet.rs](../../..//src/api/wallet.rs) | 246 | ✅ `get_balance`, `transaction_history`, `transfer_tokens` |
| **API — shop (read path)** | [src/api/shop.rs](../../..//src/api/shop.rs) | — | ✅ `get_shop_order_details(transaction_id)` server fn — **never called from any component** |
| **GraphQL** | [src/api/graphql.rs](../../..//src/api/graphql.rs) | — | ✅ `SHOP_ORDER_DETAILS_QUERY`, `ShopOrderDetailsData` wrapper |
| **Models** | [src/models/transaction.rs](../../..//src/models/transaction.rs) | — | ✅ `ShopOrderDetails`, `ShopItemSpecs`, `DeliveryDetails`, `ShopOrderDetailsResponse` — **`format_balance()` body is wrong** |
| **SCSS** | [style/wallet.scss](../../..//style/wallet.scss) | 1,092 | ✅ Desktop + mobile, transaction rows, transfer modal, fee breakdown |
| **Mock Backend — wallet** | [packages/mock_backend/src/schema/{query,mutation}/wallet.rs](../../../packages/mock_backend/src/schema/query/wallet.rs) | — | ✅ Phase 5 done — `getBalance`, `transactionHistory`, `transferTokens` |
| **Mock Backend — shop_order_details** | [packages/mock_backend/src/schema/query/shop.rs](../../../packages/mock_backend/src/schema/query/shop.rs) | 71 | ✅ Phase 5 done — returns delivery panel for the **buyer** of the order |

### What Remains 🔲

| # | Task | File(s) | Effort | Blocks |
|---|------|---------|--------|--------|
| 1 | **Fix `format_balance()` to actually group thousands** with locale-style commas (e.g. `12,345.6789`); cap fractional digits at 4 via `Decimal::round_dp(4)` to match legacy `toLocaleString({ maximumFractionDigits: 4 })`; preserve trailing-zero trimming on the fractional part | `src/models/transaction.rs` | S | Tasks 2, 7 |
| 2 | **Adopt `format_balance()` at every token-amount call site** in `wallet/` (preserves "always show separators" UX from legacy `formatAmount()`); keep `format_decimal()` available for non-amount decimals (currently no callers in `wallet/`). Concrete swap list below in Task 2 details. | `src/components/wallet/transaction_item.rs`, `src/components/wallet/transfer_modal.rs` | S | — |
| 3 | **Add `PEER_SHOP_ID` constant** + `is_shop_account()` helper on the auth/profile context, mirroring `js/global.js#L6` | `src/state/auth.rs` (or `src/utils/constants.rs`) | S | Task 4 |
| 4 | **Render delivery panel** inside the expanded `TransactionDetail` when `category == ShopPurchase` and the viewer is the shop account; lazy-load via `Resource::new` keyed on `transaction_id`, swap a "Loading…" / "Unable to load…" placeholder for the `delivery_info_container` markup; reuse legacy `delivery_label` + `price_detail_row` SCSS classes (already in `wallet.scss`) | `src/components/wallet/transaction_item.rs`, `style/wallet.scss` (verify selectors) | M | Task 5 |
| 5 | **Fall back gracefully when product metadata is unavailable** — legacy reads `peerShopProducts[shopItemId]` from Firestore for the human-readable item name; until Peer Shop's Firebase integration lands ([peer-shop-implementation.md](../peer-shop/peer-shop-implementation.md) gap), show `"Shop item #<shopItemId>"` (with optional `, size <size>` suffix from `shopItemSpecs.size`) and document the deferred lookup with a `TODO(peer-shop-firebase)` comment that links to the Peer Shop plan | `src/components/wallet/transaction_item.rs` | S | — |
| 6 | **Mock backend: allow shop account to view buyer order details** — the existing resolver gates on `o.buyer_id == user_id`, which blocks the shop-account view path the new UI exercises. Extend the auth check to also pass when the caller's id equals the seeded shop account id (mirrors legacy `PEER_SHOP_ID` gate); add a regression test covering both buyer-view and shop-account-view | `packages/mock_backend/src/schema/query/shop.rs`, `packages/mock_backend/src/seed.rs` (seed shop account + one shop order if not present), `packages/mock_backend/tests/wallet_shop_orders.rs` (new or extend existing wallet test file) | M | Task 4 verification |
| 7 | **Unit tests for `format_balance()`** — table-driven cases: `0`, `1`, `999`, `1000` → `1,000`, `1234567` → `1,234,567`, `1234.5` → `1,234.5`, `1234.5000` → `1,234.5`, very small fractions, negative values | `src/models/transaction.rs` (`#[cfg(test)]`) | S | — |
| 8 | **E2E tests (Playwright)** covering: balance renders with separators (`/12,345/` regex), open transaction → fee breakdown visible, transfer modal happy-path (friend → amount → message → confirm → success), shop-purchase row in shop-account session shows delivery panel after expand | `end2end/tests/wallet.spec.ts` (new) | L | Tasks 1–6 |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

**Critical path:** 1 → 2 → 7 (formatting track), 3 → 4 → 5 (delivery panel track), 6 in parallel with track 2, 8 last.

**Rollback note for Task 1:** `format_balance()` is currently only consumed by `balance_header.rs` and no test asserts on its un-grouped output (`grep -rn format_balance /`). Changing the implementation is safe; downstream callers added in Task 2 are written against the new behaviour from the start.

---

## Task Details

### Task 1 — Fix `format_balance()` to group thousands

**Current behaviour** ([models/transaction.rs#L325-L329](../../..//src/models/transaction.rs)):

```rust
/// Format a balance with thousand separators.
pub fn format_balance(balance: Decimal) -> String {
    // Simple implementation - trim trailing zeros
    format_decimal(balance)
}
```

The doc comment lies — there is no separator logic. The legacy reference implementation is `formatAmount()` in `js/wallet.js`, which calls `Number(x).toLocaleString('en-US', { maximumFractionDigits: 4 })`. Two semantics matter for parity: (a) thousands grouping with commas, and (b) **rounding** (not truncating) the fractional part to **at most 4 digits**. Token amounts in this app can carry 8+ fractional digits, so without the cap output would diverge from legacy.

**Implementation sketch:**

```rust
pub fn format_balance(balance: Decimal) -> String {
    // Match legacy `toLocaleString({ maximumFractionDigits: 4 })` — round, then group.
    let rounded = balance.round_dp(4);
    let trimmed = format_decimal(rounded);
    let (int_part, frac_part) = match trimmed.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (trimmed.as_str(), None),
    };

    let (sign, digits) = if let Some(rest) = int_part.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", int_part)
    };

    // Insert commas every 3 digits from the right.
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let grouped: String = grouped.chars().rev().collect();

    match frac_part {
        Some(f) => format!("{sign}{grouped}.{f}"),
        None => format!("{sign}{grouped}"),
    }
}
```

**Notes:**

- `round_dp(4)` uses banker's rounding by default — acceptable parity (legacy `toLocaleString` rounds half-to-even on most engines too). Document the rounding mode in the doc-comment.
- Operates on the trimmed string from `format_decimal()` so trailing-zero stripping behaviour is preserved on the fractional side.
- Pure ASCII; no `num-format` / `icu` crate dependency. The legacy UI is en-US only.
- Negative values are handled by stripping the sign before grouping.

---

### Task 2 — Adopt `format_balance()` in amount cells

Every `format_decimal()` call inside `wallet/` today renders a token amount. Replace them all. Concrete call-site list (verified via `grep -n 'format_decimal' src/components/wallet/`):

**`src/components/wallet/transaction_item.rs`** — 6 sites:
- Line 161 — `amount` (transaction amount)
- Line 165 — `net_amount` (base amount)
- Line 172 — `fees.total` (fees included)
- Line 176 — `fees.peer` (peer fee subitem)
- Line 180 — `fees.burn` (burn fee subitem)
- Line 185 — `inv` (investor fee subitem)

**`src/components/wallet/transfer_modal.rs`** — 9 sites:
- Lines 197, 199 — running balance / available balance preview
- Line 560 — `(Available: …)` hint under amount input
- Lines 579, 606, 692 — fee total / final total
- Lines 586, 593, 600 — peer / burn / investor fee subitems
- Line 696 — summary `amt`
- Line 700 — summary `fees.total`

Also update the import line in each file (`use crate::models::transaction::{…, format_decimal}` → `…, format_balance`). After the swap, `grep -n 'format_decimal' src/components/wallet/` MUST return zero results — that grep is the DoD assertion. `format_decimal()` itself stays exported for future non-amount callers.

---

### Task 3 — `PEER_SHOP_ID` constant + viewer check

The shop-account UI gate is the legacy hard-coded UUID `292bebb1-0951-47e8-ac8a-759138a2e4a9` ([js/global.js#L6](../../../js/global.js)).

**Implementation:**

- Define `pub const PEER_SHOP_ID: &str = "292bebb1-0951-47e8-ac8a-759138a2e4a9";` in a new `src/utils/constants.rs` module (or in `state/auth.rs` if a constants module does not yet exist — check `src/utils/mod.rs` first; create the module + register only if missing).
- Add `pub fn is_shop_account(user_id: &str) -> bool { user_id == PEER_SHOP_ID }` alongside it.
- Wire into `transaction_item.rs` by reading the current user id from `AuthContext` (already in scope on the wallet page).

**Decision:** Hard-code the prod UUID for v1, mirroring legacy. The mock-backend seed will align its shop user UUID to this same constant (Task 6) so a single source of truth holds across client + mock. If a deploy-time override becomes necessary later, lift to `option_env!("PEER_SHOP_ID").unwrap_or("292bebb1-…")` in a follow-up — out of scope here.

---

### Task 4 — Delivery panel in expanded `TransactionDetail`

**Audience model — single source of truth:**

- **UI gate (client):** show the panel only when `tx.category() == TransactionCategory::ShopPurchase && is_shop_account(current_user_id)`. Buyers viewing their own orders do **not** see a delivery panel in the wallet history (legacy parity — `js/wallet.js` only renders this for the shop account).
- **Server gate (mock backend, Task 6):** allow either `buyer_id == user_id` **or** shop-account viewer. The buyer branch stays for parity with the legacy GraphQL contract and to keep the existing `test_shop_order_details_buyer_can_view_own` regression green; it is intentionally not exercised by the wallet UI.

**Lazy-load pattern (Leptos idiom).** `LocalResource::new` fires when the resource is **constructed**, not when the surrounding `<Show>` displays. To get genuine click-triggered fetching the resource MUST be created inside the `expanded` branch — either by declaring it inside a child component that is only mounted when expanded, or by gating with a `Memo` keyed on `expanded`:

```rust
// Render this child component only when the row is expanded — the `LocalResource`
// is created in its setup, so the fetch fires on first expand and is reused on
// subsequent re-renders while the row stays open.
#[component]
fn DeliveryPanel(transaction_id: String) -> impl IntoView {
    let id_for_fetch = transaction_id.clone();
    let delivery = LocalResource::new(move || {
        let id = id_for_fetch.clone();
        async move { crate::api::shop::get_shop_order_details(id).await }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="price_detail_row md_font_size txt-color-gray">
                {t!("wallet.delivery.loading", default = "Loading delivery info…")}
            </div>
        }>
            {move || delivery.get().map(|res| match res {
                Ok(resp) => match resp.affected_rows.and_then(|r| r.into_iter().next()) {
                    Some(order) => render_delivery_panel(order).into_any(),
                    None => view! {
                        <div class="price_detail_row md_font_size txt-color-gray">
                            {t!("wallet.delivery.unavailable", default = "Unable to load delivery info")}
                        </div>
                    }.into_any(),
                },
                Err(_) => view! {
                    <div class="price_detail_row md_font_size txt-color-gray">
                        {t!("wallet.delivery.error", default = "Error loading delivery info")}
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
}

// In TransactionDetail:
<Show when=move || expanded.get() && show_delivery_for_viewer fallback=|| ()>
    <DeliveryPanel transaction_id=tx.transaction_id.clone() />
</Show>
```

This matches legacy `dataset.deliveryLoaded` lazy-load semantics. **Verify the i18n macro / helper name** against the existing wallet components before coding — if the project does not yet have one, fall back to plain string literals and open a follow-up. (See Risk #7.)

**Markup target** (legacy parity, `js/wallet.js#L233-L255`):

```html
<div class="delivery_info_container">
  <div class="delivery_label md_font_size bold">
    <i class="peer-icon peer-icon-delivery-info"></i> Delivery information
  </div>
  <div class="price_detail_row md_font_size">
    <span class="price_label txt-color-gray">Item</span>
    <span class="price bold">{itemDisplay}</span>
  </div>
  <!-- Name, Email, Address rows -->
</div>
```

Concatenate `[addressline1, addressline2, city, zipcode, country]` filtered for `Some(non_empty)` joined by `", "`. Show `"N/A"` when a field is `None`.

---

### Task 5 — Product-name fallback

Legacy reads `peerShopProducts[shopItemId]` from Firestore for the human-readable item name; that data source is not yet in peer-web (the Peer Shop plan still lists Firebase product data as a gap).

**Until that lands:** display `format!("Shop item #{}", shop_item_id)` and append `, size {size}` only if `shopItemSpecs.size` is `Some(non_empty)`. Add:

```rust
// TODO(peer-shop-firebase): replace with product lookup once the Peer Shop
// Firebase integration lands. See docs/plans/peer-shop/peer-shop-implementation.md.
```

This deliberately avoids creating a dependency between this sprint and the larger Peer Shop Firebase work.

---

### Task 6 — Mock backend: shop-account view path

Current mock resolver ([packages/mock_backend/src/schema/query/shop.rs](../../../packages/mock_backend/src/schema/query/shop.rs#L40)):

```rust
.find(|o| o.transaction_id.to_string() == transaction_id && o.buyer_id == user_id);
```

This means **only the buyer** can fetch their own order — the new client UI fetches from a **shop-account session**. Extend:

```rust
let viewer_is_shop = user_id == PEER_SHOP_ID; // mirror client constant
let order = state.shop_orders.iter().find(|o| {
    o.transaction_id.to_string() == transaction_id
        && (o.buyer_id == user_id || viewer_is_shop)
});
```

**Seed updates** (`packages/mock_backend/src/seed.rs`):

- Add a "shop" user with the same UUID as the client constant (`292bebb1-…`), a known email + password (document both at the top of the seed file), and any required session-token plumbing so Playwright can authenticate via the existing login helper without bespoke setup.
- Seed at least one `ShopOrder` with full delivery details (name, email, address line 1+2, city, zipcode, country) so the panel renders all rows in the E2E test.
- **Side-effect audit:** confirm the new shop user does not silently inflate counters used by unrelated tests (friend lists, follower counts, dashboard feed totals, referral leaderboard). If any test asserts on exact totals, update those expectations in the same commit.

**New tests** (extend the existing `wallet_*.rs` test file in `packages/mock_backend/tests/` or add a new `wallet_shop_orders.rs`):

- `test_shop_order_details_buyer_can_view_own` — passes today, regression coverage.
- `test_shop_order_details_shop_account_can_view_any` — new path enabled by this task.
- `test_shop_order_details_other_user_forbidden` — third-party still blocked.
- `test_shop_order_details_unknown_id_returns_empty_rows` — `affected_rows: None` + meta `22101`.

---

### Task 7 — Unit tests for `format_balance()`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn formats_with_thousand_separators() {
        let cases = [
            ("0", "0"),
            ("999", "999"),
            ("1000", "1,000"),
            ("12345", "12,345"),
            ("1234567", "1,234,567"),
            ("1234.5", "1,234.5"),
            ("1234.5000", "1,234.5"),
            ("0.0001", "0.0001"),
            ("-1000", "-1,000"),
            // Rounding parity with legacy `toLocaleString({ maximumFractionDigits: 4 })`:
            ("1234.56789", "1,234.5679"),   // round half away from zero / banker's — assert actual behaviour
            ("0.000049", "0"),               // rounds to zero past 4 dp
            ("-1234.56789", "-1,234.5679"),
        ];
        for (input, expected) in cases {
            let d: Decimal = input.parse().unwrap();
            assert_eq!(format_balance(d), expected, "input={input}");
        }
    }
}
```

---

### Task 8 — E2E coverage

New file: `end2end/tests/wallet.spec.ts`. Suggested cases:

1. **Balance with separators** — login as seeded user with a known balance (e.g. `12345.6789`); assert balance text matches `/\d{1,3}(,\d{3})+(\.\d+)?/` (this regex actually requires at least one grouping comma; the previous `/^[\d,]+/` form falsely passed on un-grouped digits). Optionally assert exact string `12,345.6789`.
2. **Transaction expand → fee breakdown** — click first row, assert `Fees included` row visible with grouped amount (regex same as case 1).
3. **Transfer happy path** — open transfer modal → pick friend → enter amount → enter message (≤500 chars, no URL) → confirm → success toast → modal auto-closes → balance reloads.
4. **Shop delivery panel (lazy-load assertion)** — login as the seeded shop account → before expand, attach a Playwright `page.on('request')` listener filtered to `shopOrderDetails`; assert zero matching requests until the row is expanded → expand a `SHOP_PURCHASE` row → assert `Delivery information` heading appears, Name / Email / Address rows render, and exactly one `shopOrderDetails` request fired.
5. **Non-shop viewer is gated** — login as a regular seeded user → expand a `SHOP_PURCHASE` row in their own history → assert the delivery panel is **not** rendered (UI gate) and no `shopOrderDetails` request fires.
6. **Reload balance** — click reload → spinner appears → updated balance settles.

---

## Definition of Done

- [ ] `format_balance()` produces `1,234,567.89` style output for the table-driven cases in Task 7, including the rounding-to-4dp cases.
- [ ] All token-amount call sites in `wallet/` use `format_balance()`; `grep -rn 'format_decimal' src/components/wallet/` returns zero matches.
- [ ] `PEER_SHOP_ID` constant + `is_shop_account()` helper exported from a single canonical module; only one definition exists in the workspace (`grep -rn 292bebb1 src` returns one match).
- [ ] Expanded `SHOP_PURCHASE` row rendered from a shop-account session displays the delivery panel with all available fields, falling back to `"N/A"` per missing field.
- [ ] Delivery panel fetch is lazy: no `shopOrderDetails` network request fires on row construction or while the row is collapsed; exactly one fires on first expand. Verified by Playwright network assertion (Task 8 case 4).
- [ ] Non-shop-account viewers do **not** see the delivery panel for any transaction (UI gate, asserted by Task 8 case 5) — independent of the server gate, which still allows buyer-self-view for legacy parity.
- [ ] Mock backend allows shop-account viewer to read any `shopOrderDetails`; buyer-self-view still works; third-party still blocked. New tests pass.
- [ ] User-facing strings introduced by the delivery panel (`Loading delivery info…`, `Unable to load delivery info`, `Error loading delivery info`, `Delivery information`, `Item`, `Name`, `Email`, `Address`, `N/A`) route through the project's i18n helper if one exists in neighbouring wallet components; otherwise an issue is filed and linked from the source TODO.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` (workspace + mock backend) all pass.
- [ ] `cargo build --features ssr` and `cargo build --features hydrate --target wasm32-unknown-unknown` both succeed.
- [ ] Playwright `wallet.spec.ts` runs green locally.
- [ ] [feature-convergence.md](../../feature-convergence.md) updated: Wallet row 🟡 → ✅, parent-plan scope checklist boxes ticked, Summary counts adjusted (✅ 12 → 13, 🟡 6 → 5, convergence ~87% → ~90%), Migration Priority `🟡 Wallet` → `✅ ~~Wallet~~`, `Last Updated` date bumped, changelog entry added.

---

## Risks & Open Questions

| # | Risk / Question | Resolution |
|---|----------------|------------|
| 1 | `LocalResource` vs `Resource::new` for the lazy delivery fetch — which is the project's idiom? | Audit one of the existing components that lazy-loads on a click (e.g. `RelationsModal`, `comments/comment_replies`) and copy its pattern; pick whichever already compiles cleanly under both `ssr` and `hydrate`. |
| 2 | Hard-coded `PEER_SHOP_ID` couples client + mock to a single UUID. | Acceptable for v1 (mirrors legacy). If a deploy-time override is later required, lift to `option_env!("PEER_SHOP_ID")` with the current value as the default. |
| 3 | Product-name lookup is stubbed pending Peer Shop Firebase. | Documented `TODO(peer-shop-firebase)` + plan link; not a blocker for promoting Wallet to ✅ because the legacy delivery panel also degrades to `"Shop Item"` when `peerShopProducts` is empty (`js/wallet.js#L223`). |
| 4 | `format_balance()` is exported and may be called outside `wallet/` after this sprint. | The function is currently used only in `balance_header.rs`. After Task 2 it will be used in `transaction_item.rs` + possibly `transfer_modal.rs`. No other modules import it. Safe to change the implementation. |
| 5 | Mock backend seed already has a shop user? | Verify in `packages/mock_backend/src/seed.rs` before adding; if absent, add. If present with a different UUID than the client constant, prefer aligning the seed UUID to the client constant rather than the reverse (the client constant is the one referenced by production data). |
| 6 | E2E currently has no `wallet.spec.ts`. | Confirmed via search — `end2end/tests/` has no wallet coverage. Greenfield file; no merge conflict risk. |
| 7 | Project i18n helper for the new delivery-panel strings — does one exist? | Audit one neighbouring wallet component (e.g. `transfer_modal.rs`) for an existing `t!()` / `tr()` / `i18n!()` macro before coding. If present, use it; if absent, ship plain literals and open a follow-up issue, linking it from the inline TODO. Do not block this sprint on standing up an i18n layer. |
| 8 | Convergence percentage in the Summary section is currently quoted as ~87% but recomputing 12/20 = 60%, 13/20 = 65%. The denominator-vs-percentage mismatch lives in [feature-convergence.md](../../feature-convergence.md) itself. | Out of scope to recompute the entire tracker here; this sprint will only update its own row + summary counts and flag the discrepancy in the convergence-doc commit message so it can be fixed separately. |

---

## Out of Scope

These are intentionally not part of this sprint and remain future work (already listed under "Out of Scope (Future Work)" in [wallet-implementation.md](wallet-implementation.md)):

- Win logs / Payment logs detail views
- Today's interactions summary
- Tokenomics info modal
- Token minting details
- Wallet analytics / charts
- Export transaction history
- Peer Shop Firebase product data lookup (tracked under [peer-shop-implementation.md](../peer-shop/peer-shop-implementation.md))

---

## Convergence Impact

After completion:

- Wallet row: 🟡 → ✅
- Summary table: ✅ 12 → 13, 🟡 6 → 5 (totals 20 unchanged)
- Convergence: ~87% → ~90% (per the Summary section's own rounding convention; the literal ratio 13/20 = 65% — see Risk #8)
- Migration Priority list: #9 promoted to `✅ ~~Wallet~~`
- Parent plan scope checklist: 2 unchecked items resolved
- `Last Updated` date in [feature-convergence.md](../../feature-convergence.md) bumped to the merge date
