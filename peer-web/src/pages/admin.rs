//! Admin page — content moderation dashboard.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::Redirect;

use crate::components::admin::admin_header::AdminHeader;
use crate::components::admin::moderation_list::ModerationList;
use crate::components::auth_guard::AuthGuard;

/// Admin page component — role-gated content moderation dashboard.
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Title text="Admin - Peer Network"/>
        <AuthGuard>
            <RoleGuard>
                <div id="admin-page" class="site_layout admin-layout">
                    <AdminHeader/>
                    <main class="site-main site-main-admin">
                        <h1 class="page-title xxl_font_size">"Content moderation"</h1>
                        <ModerationList/>
                    </main>
                </div>
            </RoleGuard>
        </AuthGuard>
    }
}

/// Role guard that checks for moderator/admin permissions.
///
/// Returns a tri-state: authorized, access denied (redirect), or error.
#[component]
fn RoleGuard(children: ChildrenFn) -> impl IntoView {
    let role_check = Resource::new(
        || (),
        |_| async move { check_moderator_role().await },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="admin-loading">
                <p>"Checking permissions..."</p>
            </div>
        }>
            {move || {
                role_check.get().map(|result| match result {
                    Ok(true) => (children)().into_any(),
                    Ok(false) => view! {
                        <Redirect path="/dashboard"/>
                    }.into_any(),
                    Err(_) => view! {
                        <div class="admin-loading">
                            <p class="error-text">"Failed to verify permissions. Please try again."</p>
                            <a href="/admin" class="button btn-blue">"Retry"</a>
                        </div>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}

/// Server function to check if the current user has moderator permissions.
///
/// Returns `Ok(true)` if authorized, `Ok(false)` if the user lacks
/// moderator/admin roles, `Err` for server/network failures.
#[server(CheckModeratorRole, "/api")]
async fn check_moderator_role() -> Result<bool, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, ModerationStatsData, MODERATION_STATS_QUERY};

    let token = match get_access_token_from_cookies().await {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };

    match query::<_, ModerationStatsData>(
        MODERATION_STATS_QUERY,
        serde_json::json!({}),
        Some(&token),
    )
    .await
    {
        Ok(_) => Ok(true),
        Err(e) => {
            let msg = e.to_string();
            // GraphQL permission/auth errors → not authorized
            if msg.contains("not authorized")
                || msg.contains("Not authorized")
                || msg.contains("62101")
                || msg.contains("60501")
            {
                Ok(false)
            } else {
                // Network/server errors → propagate so UI can show retry
                Err(ServerFnError::new(format!("Permission check failed: {}", msg)))
            }
        }
    }
}
