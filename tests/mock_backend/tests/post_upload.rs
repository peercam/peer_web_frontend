mod common;
use common::prelude::*;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_upload_without_auth() {
    let app = app();

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\nsome-token\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_upload_invalid_token() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let app = app_with_state(state.clone());

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\ninvalid-token\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header("Authorization", format!("Bearer {token}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["ResponseCode"], "40902");
}

#[tokio::test]
async fn test_upload_post_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get eligibility token
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            postEligibility {
                eligibilityToken
            }
        }
    "#,
        &token,
    )
    .await;
    let elig_token = res["data"]["postEligibility"]["eligibilityToken"]
        .as_str()
        .unwrap()
        .to_string();

    let app = app_with_state(state.clone());

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\n{elig_token}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header("Authorization", format!("Bearer {token}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["ResponseCode"], "11515");
    assert!(json["affectedRows"]["uploadedFiles"].is_string());
}
