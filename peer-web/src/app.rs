use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path, StaticSegment,
};

use crate::components::toast::ToastProvider;
use crate::hooks::use_proactive_refresh;
use crate::pages::{ChatPage, DashboardPage, ForgotPasswordPage, InvitePage, LoginPage, MyProfilePage, NewPostPage, ReferralBoardPage, RegisterPage, SettingsPage, ViewPostPage, ViewProfilePage, WalletPage};
use crate::state::auth::provide_auth_context;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
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

    // Set up proactive token refresh
    use_proactive_refresh();

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
                        <Route path=StaticSegment("dashboard") view=DashboardPage/>
                        <Route path=StaticSegment("profile") view=MyProfilePage/>
                        <Route path=StaticSegment("settings") view=SettingsPage/>
                        <Route path=StaticSegment("chat") view=ChatPage/>
                        <Route path=StaticSegment("wallet") view=WalletPage/>
                        <Route path=StaticSegment("invite") view=InvitePage/>
                        <Route path=StaticSegment("referral") view=ReferralBoardPage/>
                        <Route path=StaticSegment("newpost") view=NewPostPage/>
                        <Route path=StaticSegment("new") view=NewPostPage/>
                        <Route path=StaticSegment("create") view=NewPostPage/>
                        <Route path=path!("/profile/:slug") view=ViewProfilePage/>
                        <Route path=path!("/u/:slug") view=ViewProfilePage/>
                        <Route path=path!("/post/:id") view=ViewPostPage/>
                        <Route path=path!("/p/:id") view=ViewPostPage/>
                    </Routes>
                </main>
            </Router>
        </ToastProvider>
    }
}

/// Minimal home page (placeholder).
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Title text="Welcome to Peer"/>
        <h1>"Welcome to Peer"</h1>
        <a href="/register">"Create an Account"</a>
    }
}
