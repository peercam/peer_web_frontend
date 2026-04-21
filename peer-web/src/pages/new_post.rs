//! New post page.
//!
//! Allows authenticated users to create posts with different content types:
//! - Text posts
//! - Image posts (with multi-image upload and cropping)
//! - Audio posts (with file upload or voice recording)
//! - Video posts (with trimming)

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::new_post::{
    ContentTypeTabs, NewPostForm, NewPostHeader, NewPostRightSidebar, PostPreview,
};
use crate::models::post::{CreateContentType, CreatePostInput, MediaFile};

/// Provide new post context for child components.
pub fn provide_new_post_context() {
    let (content_type, set_content_type) = signal(CreateContentType::Text);
    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (tags, set_tags) = signal(Vec::<String>::new());
    let (media_files, set_media_files) = signal(Vec::<MediaFile>::new());
    let (cover_file, set_cover_file) = signal(Option::<MediaFile>::None);
    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_preview, set_show_preview) = signal(false);
    let (current_media_index, set_current_media_index) = signal(0usize);

    provide_context(NewPostContext {
        content_type,
        set_content_type,
        title,
        set_title,
        description,
        set_description,
        tags,
        set_tags,
        media_files,
        set_media_files,
        cover_file,
        set_cover_file,
        is_submitting,
        set_is_submitting,
        error,
        set_error,
        show_preview,
        set_show_preview,
        current_media_index,
        set_current_media_index,
    });
}

/// Context for new post state shared across components.
#[derive(Clone, Copy)]
pub struct NewPostContext {
    pub content_type: ReadSignal<CreateContentType>,
    pub set_content_type: WriteSignal<CreateContentType>,
    pub title: ReadSignal<String>,
    pub set_title: WriteSignal<String>,
    pub description: ReadSignal<String>,
    pub set_description: WriteSignal<String>,
    pub tags: ReadSignal<Vec<String>>,
    pub set_tags: WriteSignal<Vec<String>>,
    pub media_files: ReadSignal<Vec<MediaFile>>,
    pub set_media_files: WriteSignal<Vec<MediaFile>>,
    pub cover_file: ReadSignal<Option<MediaFile>>,
    pub set_cover_file: WriteSignal<Option<MediaFile>>,
    pub is_submitting: ReadSignal<bool>,
    pub set_is_submitting: WriteSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_error: WriteSignal<Option<String>>,
    pub show_preview: ReadSignal<bool>,
    pub set_show_preview: WriteSignal<bool>,
    pub current_media_index: ReadSignal<usize>,
    pub set_current_media_index: WriteSignal<usize>,
}

impl NewPostContext {
    /// Get the context from the reactive scope.
    pub fn use_context() -> Self {
        expect_context::<NewPostContext>()
    }

    /// Reset the form to initial state.
    pub fn reset(&self) {
        self.set_title.set(String::new());
        self.set_description.set(String::new());
        self.set_tags.set(Vec::new());
        self.set_media_files.set(Vec::new());
        self.set_cover_file.set(None);
        self.set_error.set(None);
        self.set_show_preview.set(false);
        self.set_current_media_index.set(0);
    }

    /// Add a media file.
    pub fn add_media(&self, file: MediaFile) {
        let max = self.content_type.get().max_media();
        self.set_media_files.update(|files| {
            if files.len() < max {
                files.push(file);
            }
        });
    }

    /// Remove a media file at index.
    pub fn remove_media(&self, index: usize) {
        self.set_media_files.update(|files| {
            if index < files.len() {
                files.remove(index);
            }
        });
        // Update current index if needed
        let current = self.current_media_index.get();
        let len = self.media_files.get().len();
        if current >= len && len > 0 {
            self.set_current_media_index.set(len - 1);
        }
    }

    /// Add a tag.
    pub fn add_tag(&self, tag: String) {
        let tag = tag.to_lowercase();
        self.set_tags.update(|tags| {
            if tags.len() < 10 && !tags.contains(&tag) {
                tags.push(tag);
            }
        });
    }

    /// Remove a tag.
    pub fn remove_tag(&self, tag: &str) {
        self.set_tags.update(|tags| {
            tags.retain(|t| t != tag);
        });
    }

    /// Build the CreatePostInput from current state.
    pub fn build_input(&self) -> CreatePostInput {
        CreatePostInput {
            title: self.title.get(),
            mediadescription: {
                let desc = self.description.get();
                if desc.is_empty() { None } else { Some(desc) }
            },
            contenttype: self.content_type.get(),
            media: None, // Set after upload
            cover: None, // Set after upload
            tags: {
                let tags = self.tags.get();
                if tags.is_empty() { None } else { Some(tags) }
            },
            uploaded_files: None, // Set after upload
        }
    }

    /// Validate the current form state.
    pub fn validate(&self) -> Result<(), String> {
        let title = self.title.get();
        if title.is_empty() {
            return Err("Title is required".to_string());
        }
        if title.len() > 63 {
            return Err("Title must be 63 characters or less".to_string());
        }

        let desc = self.description.get();
        if desc.len() > 500 {
            return Err("Description must be 500 characters or less".to_string());
        }

        let content_type = self.content_type.get();
        if content_type.requires_media() && self.media_files.get().is_empty() {
            return Err(format!(
                "{} requires at least one media file",
                content_type.display_name()
            ));
        }

        Ok(())
    }
}

/// New post page component.
#[component]
pub fn NewPostPage() -> impl IntoView {
    provide_new_post_context();
    let ctx = NewPostContext::use_context();

    view! {
        <Title text="Create Post - Peer Network"/>
        <AuthGuard>
            <div id="addPost" class="site_layout">
                <NewPostHeader/>
                <ContentTypeTabs/>
                <main class="site-main site-main-createpost">
                    <Show
                        when=move || !ctx.show_preview.get()
                        fallback=move || view! { <PostPreview/> }
                    >
                        <NewPostForm/>
                    </Show>
                </main>
                <NewPostRightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}

/// Mobile navigation footer.
#[component]
fn MobileFooter() -> impl IntoView {
    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="nav-item">
                    <i class="peer-icon peer-icon-home"/>
                    <span>"Home"</span>
                </a>
                <a href="/search" class="nav-item">
                    <i class="peer-icon peer-icon-search"/>
                    <span>"Search"</span>
                </a>
                <a href="/newpost" class="nav-item add-post active">
                    <i class="peer-icon peer-icon-plus"/>
                </a>
                <a href="/notifications" class="nav-item">
                    <i class="peer-icon peer-icon-bell"/>
                    <span>"Alerts"</span>
                </a>
                <a href="/profile" class="nav-item">
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}
