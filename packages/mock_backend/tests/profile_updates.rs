mod common;
use common::prelude::*;

#[tokio::test]
async fn test_update_profile_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateProfileImage(img: "data:image/png;base64,iVBOR...") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateProfileImage"]["ResponseCode"], "11004");

    let res = graphql_with_auth(
        &state,
        r#"
        query { getProfile { affectedRows { img } } }
    "#,
        &token,
    )
    .await;
    assert_eq!(
        res["data"]["getProfile"]["affectedRows"]["img"],
        "data:image/png;base64,iVBOR..."
    );
}

#[tokio::test]
async fn test_update_bio() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateBio(biography: "data:text/plain;base64,SGVsbG8gV29ybGQ=") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateBio"]["ResponseCode"], "11003");
}

#[tokio::test]
async fn test_update_username_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "newName123", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "11007");
}

#[tokio::test]
async fn test_update_username_invalid_format() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "ab", password: "TestPass123") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "30202");
}

#[tokio::test]
async fn test_update_username_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "validName", password: "WrongPass") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_update_email_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateEmail(email: "newemail@peer.com", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "11006");
}

#[tokio::test]
async fn test_update_email_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateEmail(email: "new@peer.com", password: "WrongPass") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_update_preferences() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUserPreferences(userPreferences: {
                contentFilteringSeverityLevel: MYGRANDMAHATES
            }) {
                status ResponseCode
                affectedRows { contentFilteringSeverityLevel }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["updateUserPreferences"];
    assert_eq!(data["ResponseCode"], "11014");
    assert_eq!(
        data["affectedRows"]["contentFilteringSeverityLevel"],
        "MYGRANDMAHATES"
    );
}
