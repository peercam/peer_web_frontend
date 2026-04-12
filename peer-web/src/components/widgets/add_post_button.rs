//! Add post button widget.

use leptos::prelude::*;

/// Button to create a new post.
#[component]
pub fn AddPostButton() -> impl IntoView {
    view! {
        <div class="widget widget-margin-bottom">
            <a href="/newpost" class="widget-inner widget-type-box widget-add-post">
                <i class="peer-icon peer-icon-plus"/>
                <span>"New Post"</span>
            </a>
        </div>
    }
}
