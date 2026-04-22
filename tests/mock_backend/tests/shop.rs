mod common;
use common::prelude::*;

#[tokio::test]
async fn test_shop_purchase_item() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 50.0, shopItemId: "test-item-001", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Musterstraße 42", city: "Berlin", zipcode: "10115", country: GERMANY }) { status ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "12201");

    // Balance should have decreased
    let res2 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let balance = decimal_val(&res2["data"]["balance"]["currentliquidity"]);
    assert!((balance - 950.0).abs() < 0.1);
}

#[tokio::test]
async fn test_shop_order_details() {
    let state = default_shared_state();
    let alice_token = login_alice(&state).await;

    // SEED_SHOP_ORDER_1 belongs to alice with SEED_SHOP_TX_1
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ shopOrderDetails(transactionId: "{SEED_SHOP_TX_1}") {{ meta {{ ResponseCode }} affectedRows {{ shopOrderId shopItemId deliveryDetails {{ name email city zipcode }} }} }} }}"#
        ),
        &alice_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "12202"
    );
    let order = &res["data"]["shopOrderDetails"]["affectedRows"][0];
    assert_eq!(order["shopItemId"], "peer-tshirt-001");
    assert_eq!(order["deliveryDetails"]["city"], "Berlin");
}

#[tokio::test]
async fn test_shop_order_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { shopOrderDetails(transactionId: "99999999-9999-4999-a999-999999999999") { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "22101"
    );
}

#[tokio::test]
async fn test_shop_purchase_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 50.0, shopItemId: "test-item", orderDetails: { name: "Test", email: "t@t.com", addressline1: "Street 123", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_shop_purchase_insufficient_balance() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 99999.0, shopItemId: "expensive-item", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "51301");
}

#[tokio::test]
async fn test_shop_purchase_invalid_name() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "X", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

#[tokio::test]
async fn test_shop_purchase_invalid_zipcode() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "123", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

#[tokio::test]
async fn test_shop_purchase_invalid_email() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "Test User", email: "notanemail", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

#[tokio::test]
async fn test_shop_order_details_buyer_can_view_own() {
    let state = default_shared_state();
    let alice_token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ shopOrderDetails(transactionId: "{SEED_SHOP_TX_1}") {{ meta {{ ResponseCode }} affectedRows {{ shopOrderId }} }} }}"#
        ),
        &alice_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "12202"
    );
    assert!(
        res["data"]["shopOrderDetails"]["affectedRows"][0]["shopOrderId"].is_string(),
        "buyer should see their own order"
    );
}

#[tokio::test]
async fn test_shop_order_details_shop_account_can_view_any() {
    let state = default_shared_state();
    let shop_token = login_shop(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ shopOrderDetails(transactionId: "{SEED_SHOP_TX_1}") {{ meta {{ ResponseCode }} affectedRows {{ shopOrderId deliveryDetails {{ name email city }} }} }} }}"#
        ),
        &shop_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "12202"
    );
    let order = &res["data"]["shopOrderDetails"]["affectedRows"][0];
    assert!(order["shopOrderId"].is_string());
    assert_eq!(order["deliveryDetails"]["city"], "Berlin");
}

#[tokio::test]
async fn test_shop_order_details_other_user_forbidden() {
    let state = default_shared_state();
    // Bob is not the buyer of SEED_SHOP_ORDER_1 (Alice is) and is not the
    // shop account; the resolver should report "Order not found".
    let bob_token = login_bob(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ shopOrderDetails(transactionId: "{SEED_SHOP_TX_1}") {{ meta {{ ResponseCode }} affectedRows {{ shopOrderId }} }} }}"#
        ),
        &bob_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "22101"
    );
    assert!(res["data"]["shopOrderDetails"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_shop_order_details_unknown_id_returns_empty_rows() {
    let state = default_shared_state();
    let shop_token = login_shop(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { shopOrderDetails(transactionId: "99999999-9999-4999-a999-999999999999") { meta { ResponseCode } affectedRows { shopOrderId } } }"#,
        &shop_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "22101"
    );
    assert!(res["data"]["shopOrderDetails"]["affectedRows"].is_null());
}
