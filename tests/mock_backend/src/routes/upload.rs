use axum::{
    Json,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<UploadAffectedRows>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadAffectedRows {
    #[serde(rename = "uploadedFiles")]
    pub uploaded_files: String,
}

/// POST /upload-post handler.
pub async fn upload_post_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut body: Multipart,
) -> (StatusCode, Json<UploadResponse>) {
    // 1. Auth check
    let viewer_id = {
        let token = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        match token {
            Some(t) => {
                let mock_state = state.mock_state.read().await;
                match mock_state.access_tokens.get(t) {
                    Some(uid) => *uid,
                    None => {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(UploadResponse {
                                status: "error".into(),
                                response_code: "60501".into(),
                                affected_rows: None,
                            }),
                        );
                    }
                }
            }
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(UploadResponse {
                        status: "error".into(),
                        response_code: "60501".into(),
                        affected_rows: None,
                    }),
                );
            }
        }
    };

    // 2. Parse multipart — extract eligibilityToken and file fields
    let mut eligibility_token = None;
    let mut file_count = 0u32;

    while let Ok(Some(field)) = body.next_field().await {
        let name = field.name().map(|n| n.to_string());
        match name.as_deref() {
            Some("eligibilityToken") => {
                eligibility_token = field.text().await.ok();
            }
            Some("file") => {
                let _ = field.bytes().await;
                file_count += 1;
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    // 3. Validate eligibility token
    let token_str = match eligibility_token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    status: "error".into(),
                    response_code: "40902".into(),
                    affected_rows: None,
                }),
            );
        }
    };

    let mut mock_state = state.mock_state.write().await;

    let token_status = mock_state.eligibility_token_status.get(&token_str).cloned();
    match token_status.as_deref() {
        Some("ISSUED") => { /* valid */ }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    status: "error".into(),
                    response_code: "40902".into(),
                    affected_rows: None,
                }),
            );
        }
    }

    // Verify token belongs to this user
    if mock_state.eligibility_tokens.get(&token_str) != Some(&viewer_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                status: "error".into(),
                response_code: "40902".into(),
                affected_rows: None,
            }),
        );
    }

    // 4. Generate mock filenames
    let filenames: Vec<String> = (0..file_count.max(1))
        .map(|_| format!("mock-{}.jpg", Uuid::new_v4()))
        .collect();

    // 5. Update state
    mock_state
        .eligibility_token_status
        .insert(token_str.clone(), "FILE_UPLOADED".into());
    mock_state
        .uploaded_files
        .insert(token_str, filenames.clone());

    (
        StatusCode::OK,
        Json(UploadResponse {
            status: "success".into(),
            response_code: "11515".into(),
            affected_rows: Some(UploadAffectedRows {
                uploaded_files: filenames.join(","),
            }),
        }),
    )
}
