use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Redirect, Route, Router, Routes},
    hooks::{use_navigate, use_query_map},
    path,
};

use crate::components::pwa::InstallPrompt;
use crate::components::toast::ToastProvider;
use crate::hooks::use_proactive_refresh;
use crate::pages::{
    AdminPage, ChatPage, DashboardPage, ForgotPasswordPage, InvitePage, LoginPage, MyAdsPage,
    MyProfilePage, NewPostPage, PeerShopPage, ReferralBoardPage, RegisterPage, SettingsPage,
    VersionHistoryPage, ViewPostPage, ViewProfilePage, WalletPage,
};
use crate::state::auth::{encode_redirect, provide_auth_context, use_auth};
use crate::utils::pwa::{
    provide_install_prompt_context, provide_sw_update_context, register_service_worker,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>

                // PWA / Web App Manifest
                <link rel="manifest" href="/manifest.webmanifest"/>
                <meta name="theme-color" content="#00beff"/>
                <meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)"/>
                <meta name="application-name" content="Peer"/>

                // Apple-specific
                <meta name="mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <meta name="apple-mobile-web-app-title" content="Peer"/>
                <link rel="apple-touch-icon" href="/img/pwa/apple-touch-icon-180.png"/>
                <link rel="icon" type="image/png" sizes="192x192" href="/img/pwa/icon-192.png"/>
                <link rel="icon" type="image/png" sizes="512x512" href="/img/pwa/icon-512.png"/>

                // Apple splash (iOS home-screen launch images)
                <link
                    rel="apple-touch-startup-image"
                    href="/img/pwa/splash/apple-splash-2048-2732.png"
                    media="(device-width: 1024px) and (device-height: 1366px) and (-webkit-device-pixel-ratio: 2)"
                />
                <link
                    rel="apple-touch-startup-image"
                    href="/img/pwa/splash/apple-splash-1290-2796.png"
                    media="(device-width: 430px) and (device-height: 932px) and (-webkit-device-pixel-ratio: 3)"
                />
                <link
                    rel="apple-touch-startup-image"
                    href="/img/pwa/splash/apple-splash-1170-2532.png"
                    media="(device-width: 390px) and (device-height: 844px) and (-webkit-device-pixel-ratio: 3)"
                />

                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();
    provide_install_prompt_context();
    provide_sw_update_context();

    // Set up proactive token refresh
    use_proactive_refresh();

    // Register the service worker on hydrate (no-op on SSR).
    register_service_worker();

    view! {
        <Stylesheet id="leptos" href="/pkg/peer-web.css"/>

        <ToastProvider>
            <Router>
                <main>
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("login") view=LoginPage/>
                        <Route path=StaticSegment("register") view=RegisterPage/>
                        <Route path=StaticSegment("forgotpassword") view=ForgotPasswordPage/>
                        <Route path=StaticSegment("admin") view=AdminPage/>
                        <Route path=StaticSegment("dashboard") view=DashboardPage/>
                        <Route path=StaticSegment("profile") view=MyProfilePage/>
                        <Route path=StaticSegment("settings") view=SettingsPage/>
                        <Route path=StaticSegment("chat") view=ChatPage/>
                        <Route path=StaticSegment("wallet") view=WalletPage/>
                        <Route path=StaticSegment("my-ads") view=MyAdsPage/>
                        <Route path=StaticSegment("invite") view=InvitePage/>
                        <Route path=StaticSegment("referral") view=ReferralBoardPage/>
                        <Route path=StaticSegment("newpost") view=NewPostPage/>
                        <Route path=StaticSegment("new") view=NewPostPage/>
                        <Route path=StaticSegment("create") view=NewPostPage/>
                        <Route path=StaticSegment("shop") view=PeerShopPage/>
                        <Route path=StaticSegment("version-history") view=VersionHistoryPage/>
                        <Route path=StaticSegment("edit-profile") view=EditProfileRedirect/>
                        <Route path=StaticSegment("edit_profile") view=EditProfileRedirect/>
                        <Route path=path!("/profile/:slug") view=ViewProfilePage/>
                        <Route path=path!("/u/:slug") view=ViewProfilePage/>
                        <Route path=path!("/post/:id") view=ViewPostPage/>
                        <Route path=path!("/p/:id") view=ViewPostPage/>
                    </Routes>
                </main>
                <InstallPrompt/>
            </Router>
        </ToastProvider>
    }
}

/// Auth-aware landing route.
///
/// Mirrors the legacy `index.php` 302 to `dashboard.php`: visiting `/` ends up
/// at `/dashboard` (signed in) or `/login?message=mustLogin` (signed out).
/// Any inbound `?redirect=…` is preserved through to `/login` (re-encoded via
/// the shared [`encode_redirect`] helper); other inbound query keys, including
/// `?message=…`, are dropped to keep behaviour consistent with `AuthGuard`.
#[component]
fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let query = use_query_map();

    let guest_redirect_url = Memo::new(move |_| {
        let redirect = query.with(|q| q.get("redirect"));
        match redirect {
            Some(target) if !target.is_empty() => format!(
                "/login?message=mustLogin&redirect={}",
                encode_redirect(&target)
            ),
            _ => "/login?message=mustLogin".to_string(),
        }
    });

    view! {
        <Title text="Peer Network"/>
        <Show
            when=move || auth.is_session_checked.get()
            fallback=|| view! { <div class="auth-guard-loading" role="status" aria-busy="true"></div> }
        >
            <Show
                when=move || auth.is_authenticated.get()
                fallback=move || view! { <Redirect path=guest_redirect_url.get()/> }
            >
                <Redirect path="/dashboard"/>
            </Show>
        </Show>
    }
}

/// Redirect legacy edit-profile URLs to the settings page.
#[component]
#[allow(clippy::unused_unit)]
fn EditProfileRedirect() -> impl IntoView {
    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate("/settings", Default::default());
    });

    view! {}
}
