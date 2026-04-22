# New Post Implementation Plan

**Feature:** New Post  
**Priority:** #6 (after View Post & Profile)  
**Status:** 🚧 In Progress  
**Created:** 2026-04-12  
**Updated:** 2026-04-22

---

## Overview

Implement the post creation page for the Leptos frontend. This is a complex feature supporting four content types (text, image, audio, video), each with specialized editing capabilities including image cropping, video trimming, voice recording with waveform visualization, and tag management.

### Goals

1. Full parity with legacy `newpost.php` user experience
2. Support all four content types: text, image, audio, video
3. Multi-file upload with image slider
4. Image cropping with aspect ratio selection (1:1, 4:5)
5. Video trimming with timeline scrubbing
6. Voice recording with real-time waveform visualization
7. Audio cover image upload
8. Tag system with search, suggestions, and history
9. Live post preview (full and collapsed card views)
10. Multipart file upload with eligibility token
11. Token cost display with free daily action support

---

## Scope

### In Scope

- [x] New post page (`/new` or `/create` route)
- [x] Content type selector (text, image, audio, video tabs)
- [x] **Text Post:**
  - [x] Title input (required, 1-63 chars)
  - [x] Description textarea (1-500 chars)
  - [x] Character count indicators
- [ ] **Image Post:**
  - [x] Drag & drop / file picker for images
  - [x] Multi-image upload (up to 5 images)
  - [x] Image slider navigation
  - [x] Image cropping modal
  - [x] Aspect ratio toggle (1:1 square, 4:5 vertical)
  - [x] Cropped image preview
  - [x] Remove individual images
- [ ] **Audio Post:**
  - [x] Audio file upload (.mp3, .wav, .flac, .aac, .m4a)
  - [x] Voice recording with microphone
  - [ ] Real-time waveform visualization _(static SVG placeholder only — see [completion sprint](new-post-completion-sprint.md) Task 2)_
  - [x] Recording timer
  - [x] Playback controls (play/pause)
  - [x] Record again functionality
  - [x] Cover image upload (optional background)
- [ ] **Video Post:**
  - [x] Video file upload
  - [x] Dual video support (up to 2 videos)
  - [x] Video trimming interface
  - [ ] Timeline with thumbnail frames _(no frame extraction — see [completion sprint](new-post-completion-sprint.md) Task 4)_
  - [x] Start/end handle dragging
  - [x] Minimum trim duration (3 seconds) _(validation in place)_
  - [ ] FFmpeg WASM encoding _(not integrated; deferred in favour of server-side trim — see [completion sprint](new-post-completion-sprint.md) Task 5)_
  - [x] Cover image generation/upload
  - [x] Progress indicator during processing _(UI exists)_
- [ ] **Tag System:**
  - [x] Tag input with autocomplete
  - [x] Tag search via API
  - [x] Selected tags display with remove
  - [ ] Tag history (localStorage) _(not implemented — see [completion sprint](new-post-completion-sprint.md) Task 6)_
  - [x] Max 10 tags per post
  - [x] Tag validation (alphanumeric/underscores, 2-53 chars)
- [x] **Live Preview:**
  - [x] Full view preview (modal-style)
  - [x] Collapsed card preview
  - [x] Toggle between preview modes
  - [x] Back to edit functionality
- [x] **Submit Flow:**
  - [x] Pre-submission validation
  - [x] Eligibility token fetch (`postEligibility`)
  - [x] Multipart file upload (`/upload-post`)
  - [x] Post creation (`createPost` mutation)
  - [x] Success/error feedback
  - [x] Redirect to profile on success
- [x] Loading states and progress indicators
- [x] Form validation with error messages
- [x] Token cost indicator (20 tokens or free daily)
- [x] Responsive layout

### Out of Scope (Future Work)

- Post editing (future feature)
- Scheduled posts
- Draft saving
- Post templates
- Multiple video trimming
- Advanced audio editing
- GIF creation
- Poll posts
- Location tagging

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `newpost.php` | Page template with layout structure |
| `js/add_post.js` | Post creation logic, form submission, media handling |
| `js/crop.js` | Image cropping canvas operations |
| `js/voiceRecorderApi.js` | Voice recording, waveform visualization, playback |
| `js/audio.js` | Audio player initialization |
| `js/ffmpeg/` | FFmpeg WASM for video encoding/trimming |
| `css/add-post.css` | Post creation form styles |
| `css/crop.css` | Cropping modal styles |
| `css/preview.css` | Preview section styles |
| `template-parts/content-parts/add-post.php` | Form elements template |
| `template-parts/content-parts/preview.php` | Preview section template |
| `template-parts/sidebars/widget-create-post-filter.php` | Content type tabs |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: Logo + "Text Post" / "Image Post" etc.            │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (16rem)     │                         │  (17rem)          │
│              │  ┌───────────────────┐  │                   │
│  Content     │  │ MEDIA UPLOAD AREA │  │  - Profile widget │
│  Type Tabs:  │  │ (varies by type)  │  │  - Main menu      │
│  - Text      │  │                   │  │  - New post btn   │
│  - Image     │  └───────────────────┘  │  - Version        │
│  - Audio     │                         │                   │
│  - Video     │  ┌───────────────────┐  │                   │
│              │  │ TITLE INPUT       │  │                   │
│              │  └───────────────────┘  │                   │
│              │                         │                   │
│              │  ┌───────────────────┐  │                   │
│              │  │ DESCRIPTION       │  │                   │
│              │  │ (textarea)        │  │                   │
│              │  └───────────────────┘  │                   │
│              │                         │                   │
│              │  ┌───────────────────┐  │                   │
│              │  │ TAGS SECTION      │  │                   │
│              │  │ Input + Selected  │  │                   │
│              │  └───────────────────┘  │                   │
│              │                         │                   │
│              │  [Preview] [Submit]     │                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Content Type Specific Areas

#### Image Upload Area
```
┌────────────────────────────────────────────────────┐
│  IMAGE PREVIEW SLIDER                              │
│  ┌────┬────┬────┬────┬────┐                        │
│  │ ← │ IMG │ IMG │ IMG │ → │  Navigation arrows    │
│  └────┴────┴────┴────┴────┘                        │
│                                                    │
│  [+] Upload more images                            │
│                                                    │
│  ○ 1:1 Square    ○ 4:5 Vertical  (aspect ratio)   │
└────────────────────────────────────────────────────┘
```

#### Audio Upload Area
```
┌────────────────────────────────────────────────────┐
│  ┌─────────────────┬──────────────────────────┐    │
│  │ [+] Upload      │   VOICE RECORDING        │    │
│  │ .mp3/.wav/...   │                          │    │
│  │                 │   ╭──╮ Waveform SVG      │    │
│  │                 │   │○○│   00:00            │    │
│  │                 │   ╰──╯ Start recording   │    │
│  └─────────────────┴──────────────────────────┘    │
│                                                    │
│  ┌────────────────────────────────────────────┐    │
│  │ [+] Choose background image (cover)        │    │
│  └────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────┘
```

#### Video Upload Area
```
┌────────────────────────────────────────────────────┐
│  VIDEO PREVIEW                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │                 ▶                           │   │
│  │             VIDEO FRAME                      │  │
│  └─────────────────────────────────────────────┘   │
│                                                    │
│  TIMELINE (trimming)                               │
│  ┌─────────────────────────────────────────────┐   │
│  │ ║ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ ║ │   │
│  └─────────────────────────────────────────────┘   │
│    ↑ start handle              end handle ↑       │
│                                                    │
│  [Trim Video] [Cancel]                             │
└────────────────────────────────────────────────────┘
```

### Preview Modes

```
FULL VIEW PREVIEW                    COLLAPSED CARD PREVIEW
┌────────────────────────────┐       ┌───────────────┐
│ ┌──────────┬─────────────┐ │       │ ┌───────────┐ │
│ │          │ User header │ │       │ │ MEDIA     │ │
│ │  MEDIA   │ Title       │ │       │ │ THUMBNAIL │ │
│ │  GALLERY │ Description │ │       │ └───────────┘ │
│ │          │ #tags       │ │       │ Title         │
│ │          │ Comments... │ │       │ Description   │
│ └──────────┴─────────────┘ │       │ #tags         │
└────────────────────────────┘       └───────────────┘
```

### Key Features

1. **Content Type Tabs**
   - Sidebar tabs for switching between post types
   - Form fields dynamically update based on type
   - Active tab styling

2. **Image Cropping**
   - Canvas-based cropping with drag/zoom
   - Two aspect ratios: 1:1 (square), 4:5 (vertical/portrait)
   - Mouse drag to position image
   - Scroll to zoom (scale 0.3 - 8x)
   - "Crop" button saves to preview

3. **Multi-Image Upload**
   - Up to 5 images per post
   - Slider navigation between images
   - Individual image removal
   - "Add more" button
   - Each image gets cropped separately

4. **Voice Recording**
   - MediaRecorder API for capturing
   - Real-time waveform visualization (SVG with paths)
   - Timer display while recording
   - Playback with pause/resume
   - Record again to overwrite
   - Chrome/Safari: WebM → WAV conversion

5. **Video Trimming**
   - Timeline with draggable start/end handles
   - Minimum 3-second duration
   - FFmpeg WASM for encoding
   - Progress bar during processing
   - Cover frame extraction

6. **Tag Management**
   - Input with autocomplete dropdown
   - `searchTags` API for suggestions
   - Tag history in localStorage
   - Clear history option
   - Max 10 tags
   - Validation: `^[a-zA-Z0-9_]+$`, 2-53 chars

7. **Form Validation**
   - Title: required, 1-63 chars
   - Description: optional, max 500 chars
   - Media: at least one file required
   - Tags: valid format, max 10
   - Real-time error messages

8. **Submit Flow**
   1. Client-side validation
   2. Fetch eligibility token (`postEligibility`)
   3. Upload files to `/upload-post` with token
   4. Call `createPost` mutation with uploaded filenames
   5. Show success/error message
   6. Redirect to profile on success

---

## Backend API Reference

### `postEligibility` Query

```graphql
query PostEligibility {
  postEligibility {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    eligibilityToken
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `10901` | Eligibility token issued |
| `31512` | Rate limit exceeded (max 5 tokens/hour) |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `POST /upload-post` (REST Endpoint)

Multipart file upload for post media.

```http
POST /upload-post
Content-Type: multipart/form-data
Authorization: Bearer <accessToken>

--boundary
Content-Disposition: form-data; name="eligibilityToken"

<token>
--boundary
Content-Disposition: form-data; name="file"; filename="image0.png"
Content-Type: image/png

<binary data>
--boundary--
```

#### Response

```json
{
  "status": "success",
  "ResponseCode": "11515",
  "affectedRows": {
    "uploadedFiles": "image0.png,image1.png"
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11515` | File uploaded successfully |
| `30102` | Missing or invalid fields |
| `30261` | Invalid file |
| `40902` | Invalid eligibility token |
| `41514` | Upload failed |
| `60501` | Not authenticated |

---

### `createPost` Mutation

```graphql
mutation CreatePost($action: PostType!, $input: PostInput!) {
  createPost(action: $action, input: $input) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      id
      contenttype
      title
    }
  }
}
```

#### Input Types

```graphql
enum PostType {
  POST
}

input PostInput {
  title: String!               # 1–63 characters
  mediadescription: String     # 1–500 characters (optional)
  contenttype: ContentType!    # image, audio, video, text
  media: [String!]             # Media file paths/URLs
  cover: [String!]             # Cover image paths/URLs
  tags: [String!]              # Up to 10 tags
  uploadedFiles: String        # Comma-separated filenames from /upload-post
}

enum ContentType {
  image
  audio
  video
  text
}
```

#### Media Limits Per Content Type

| Content Type | Max Media | Max Cover |
|-------------|-----------|-----------|
| `image` | 5 | 1 |
| `audio` | 1 | 1 |
| `video` | 2 | 1 |
| `text` | 1 | 1 |

#### Token Cost

- **Price:** 20.0 tokens per post
- **Daily free:** 1 free post per day

#### Response Codes

| Code | Description |
|------|-------------|
| `11508` | Post created (paid) |
| `11513` | Post created (free daily action) |
| `30101` | Missing required fields |
| `30102` | Empty field values |
| `30206` | Invalid contenttype |
| `30210` | Invalid title length (1-63 chars) |
| `30251` | Invalid media format |
| `30262` | Invalid tags (format/count) |
| `30263` | Invalid mediadescription length |
| `30266` | Mixed media types not allowed |
| `30267` | Too many media items |
| `30268` | Too many cover items |
| `31511` | Temporary file expired |
| `40301` | Unexpected error |
| `40306` | Cover upload failed |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `searchTags` Query

```graphql
query SearchTags($tagName: String!, $offset: Int, $limit: Int) {
  searchTags(tagName: $tagName, offset: $offset, limit: $limit) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      name
    }
  }
}
```

#### Constraints

- `tagName`: 2–53 chars, alphanumeric/underscores (`^[a-zA-Z0-9_]+$`)

---

## Implementation Plan

### Phase 1: Core Infrastructure & Models

#### 1.1 Post Models (`src/models/post.rs`)

Extend existing or create new:

```rust
/// Content type for post creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Image,
    Audio,
    Video,
    Text,
}

/// Input for creating a new post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostInput {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediadescription: Option<String>,
    pub contenttype: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_files: Option<String>,
}

/// Post eligibility response.
#[derive(Debug, Clone, Deserialize)]
pub struct PostEligibilityResponse {
    pub meta: MetaResponse,
    pub eligibility_token: Option<String>,
}

/// Tag from search results.
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub name: String,
}

/// Tag search response.
#[derive(Debug, Clone, Deserialize)]
pub struct TagSearchResponse {
    pub meta: MetaResponse,
    pub counter: i32,
    pub affected_rows: Option<Vec<Tag>>,
}
```

#### 1.2 API Functions (`src/api/posts.rs`)

```rust
/// Check post eligibility and get upload token.
#[server(CheckPostEligibility)]
pub async fn check_post_eligibility() -> Result<PostEligibilityResponse, ServerFnError> {
    // GraphQL query to postEligibility
}

/// Upload files for post (multipart).
#[server(UploadPostFiles)]
pub async fn upload_post_files(
    eligibility_token: String,
    files: Vec<FileData>,
) -> Result<UploadResponse, ServerFnError> {
    // REST POST to /upload-post
}

/// Create a new post.
#[server(CreatePost)]
pub async fn create_post(input: CreatePostInput) -> Result<PostResponse, ServerFnError> {
    // GraphQL mutation to createPost
}

/// Search tags by name.
#[server(SearchTags)]
pub async fn search_tags(
    tag_name: String,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<TagSearchResponse, ServerFnError> {
    // GraphQL query to searchTags
}
```

---

### Phase 2: Page & Form Structure

#### 2.1 New Post Page (`src/pages/new_post.rs`)

```rust
#[component]
pub fn NewPostPage() -> impl IntoView {
    // State
    let (content_type, set_content_type) = signal(ContentType::Text);
    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (tags, set_tags) = signal(Vec::<String>::new());
    let (media_files, set_media_files) = signal(Vec::<MediaFile>::new());
    let (cover_file, set_cover_file) = signal(Option::<MediaFile>::None);
    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_preview, set_show_preview) = signal(false);

    view! {
        <RequireAuth>
            <div class="site_layout" id="addPost">
                <NewPostHeader content_type />
                <NewPostSidebar content_type set_content_type />
                <main class="site-main site-main-createpost">
                    <Show when=move || !show_preview.get()>
                        <NewPostForm
                            content_type
                            title set_title
                            description set_description
                            tags set_tags
                            media_files set_media_files
                            cover_file set_cover_file
                        />
                    </Show>
                    <Show when=move || show_preview.get()>
                        <PostPreview
                            content_type
                            title
                            description
                            tags
                            media_files
                            cover_file
                            on_back=move |_| set_show_preview.set(false)
                        />
                    </Show>
                </main>
                <NewPostRightSidebar />
                <Footer />
            </div>
        </RequireAuth>
    }
}
```

#### 2.2 Content Type Components

```
src/components/new_post/
├── mod.rs
├── content_type_tabs.rs      # Sidebar tabs
├── form/
│   ├── mod.rs
│   ├── text_form.rs          # Text post form
│   ├── image_form.rs         # Image upload + slider
│   ├── audio_form.rs         # Audio upload + recording
│   └── video_form.rs         # Video upload + trimming
├── media/
│   ├── mod.rs
│   ├── image_cropper.rs      # Canvas-based cropping
│   ├── image_slider.rs       # Multi-image navigation
│   ├── voice_recorder.rs     # Recording + visualization
│   ├── video_trimmer.rs      # Timeline trimming
│   └── drop_zone.rs          # Drag & drop file area
├── tags/
│   ├── mod.rs
│   ├── tag_input.rs          # Autocomplete input
│   ├── tag_list.rs           # Selected tags display
│   └── tag_suggestions.rs    # Dropdown suggestions
├── preview/
│   ├── mod.rs
│   ├── full_preview.rs       # Full post preview
│   └── card_preview.rs       # Collapsed card preview
└── submit_form.rs            # Form submission logic
```

---

### Phase 3: Media Components

#### 3.1 Image Cropper (`image_cropper.rs`)

```rust
#[component]
pub fn ImageCropper(
    image_src: Signal<String>,
    aspect_ratio: Signal<f64>,    // 1.0 for 1:1, 0.8 for 4:5
    on_crop: Callback<String>,    // Returns cropped base64
    on_cancel: Callback<()>,
) -> impl IntoView {
    // State for position, scale, dragging
    let (position, set_position) = signal((0.0, 0.0));
    let (scale, set_scale) = signal(1.0);
    let (is_dragging, set_dragging) = signal(false);
    
    // Canvas refs
    let crop_canvas = NodeRef::<Canvas>::new();
    let cropped_canvas = NodeRef::<Canvas>::new();

    // Draw function
    let draw = move || {
        // Draw image on canvas with dark overlay outside crop area
    };

    // Crop function
    let perform_crop = move |_| {
        // Extract cropped region to cropped_canvas
        // Convert to base64 and call on_crop
    };

    view! {
        <dialog class="crop-modal" open>
            <div class="crop-container">
                <canvas
                    node_ref=crop_canvas
                    on:mousedown=handle_mouse_down
                    on:mousemove=handle_mouse_move
                    on:mouseup=handle_mouse_up
                    on:wheel=handle_wheel
                />
                <canvas node_ref=cropped_canvas class="hidden" />
            </div>
            <div class="crop-controls">
                <AspectRatioToggle aspect_ratio />
                <button on:click=perform_crop>"Crop"</button>
                <button on:click=move |_| on_cancel.run(())>"Cancel"</button>
            </div>
        </dialog>
    }
}
```

#### 3.2 Voice Recorder (`voice_recorder.rs`)

```rust
#[component]
pub fn VoiceRecorder(
    on_recording_complete: Callback<AudioBlob>,
) -> impl IntoView {
    let (state, set_state) = signal(RecorderState::Initial);
    let (elapsed_time, set_elapsed_time) = signal(0u32);
    let (audio_url, set_audio_url) = signal(Option::<String>::None);
    
    // Waveform visualization
    let visualizer_paths = NodeRef::<Svg>::new();

    // Start recording
    let start_recording = move |_| {
        // Request microphone access
        // Create MediaRecorder
        // Start waveform animation
        set_state.set(RecorderState::Recording);
    };

    // Stop recording
    let stop_recording = move |_| {
        // Stop MediaRecorder
        // Convert to WAV if needed (Chrome/Safari)
        // Create blob URL
        set_state.set(RecorderState::Preview);
    };

    // Playback control
    let toggle_playback = move |_| {
        // Play/pause recorded audio
    };

    view! {
        <div class="voice-recorder">
            <WaveformVisualizer node_ref=visualizer_paths />
            <div class="mic-button" on:click=handle_mic_click>
                <MicIcon state />
            </div>
            <span class="recording-timer">
                {move || format_time(elapsed_time.get())}
            </span>
            <Show when=move || state.get() == RecorderState::Preview>
                <button class="record-again" on:click=reset_recording>
                    "Record again"
                </button>
            </Show>
        </div>
    }
}
```

#### 3.3 Video Trimmer (`video_trimmer.rs`)

```rust
#[component]
pub fn VideoTrimmer(
    video_src: Signal<String>,
    on_trim_complete: Callback<VideoBlob>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    const MIN_DURATION: f64 = 3.0;
    
    let (start_percent, set_start) = signal(0.0);
    let (end_percent, set_end) = signal(1.0);
    let (is_processing, set_processing) = signal(false);
    let (progress, set_progress) = signal(0);
    
    let video_ref = NodeRef::<Video>::new();

    // Handle trim
    let perform_trim = move |_| async move {
        set_processing.set(true);
        
        // Load FFmpeg WASM
        // Execute trim command
        // Report progress
        // Return trimmed video blob
        
        set_processing.set(false);
    };

    view! {
        <div class="video-trimmer">
            <video node_ref=video_ref src=video_src controls />
            
            <div class="timeline">
                <div class="overlay-left" style=move || format!("width: {}%", start_percent.get() * 100.0) />
                <div class="trim-window" />
                <div class="overlay-right" style=move || format!("width: {}%", (1.0 - end_percent.get()) * 100.0) />
                
                <div class="handle-left" on:mousedown=start_drag_left />
                <div class="handle-right" on:mousedown=start_drag_right />
            </div>
            
            <div class="trim-controls">
                <button on:click=perform_trim disabled=is_processing>
                    {move || if is_processing.get() { "Processing..." } else { "Trim Video" }}
                </button>
                <button on:click=move |_| on_cancel.run(())>"Cancel"</button>
            </div>
            
            <Show when=is_processing>
                <progress value=progress max=100 />
                <span>{move || format!("{}%", progress.get())}</span>
            </Show>
        </div>
    }
}
```

---

### Phase 4: Tag System

#### 4.1 Tag Input (`tag_input.rs`)

```rust
#[component]
pub fn TagInput(
    tags: RwSignal<Vec<String>>,
    max_tags: usize,
) -> impl IntoView {
    let (input_value, set_input) = signal(String::new());
    let (suggestions, set_suggestions) = signal(Vec::<String>::new());
    let (show_suggestions, set_show) = signal(false);
    
    // Debounced search
    let search_tags = create_action(|query: &String| {
        let q = query.clone();
        async move {
            search_tags_api(q, None, Some(10)).await
        }
    });

    // Tag history from localStorage
    let tag_history = use_local_storage::<Vec<String>>("tagHistory");

    let add_tag = move |tag: String| {
        let tag = tag.to_lowercase();
        if !tags.get().contains(&tag) && tags.get().len() < max_tags {
            tags.update(|t| t.push(tag.clone()));
            // Update history
            tag_history.update(|h| {
                if !h.contains(&tag) {
                    h.insert(0, tag);
                    h.truncate(20);
                }
            });
        }
        set_input.set(String::new());
        set_show.set(false);
    };

    view! {
        <div class="tag-input-container">
            <input
                type="text"
                placeholder="Add tags..."
                prop:value=input_value
                on:input=handle_input
                on:keydown=handle_keydown
                on:focus=move |_| set_show.set(true)
            />
            <Show when=show_suggestions>
                <TagSuggestions
                    suggestions
                    history=tag_history
                    on_select=add_tag
                />
            </Show>
        </div>
        <TagList tags on_remove=move |tag| tags.update(|t| t.retain(|x| x != &tag)) />
        <span class="tag-count">{move || format!("{}/{}", tags.get().len(), max_tags)}</span>
    }
}
```

---

### Phase 5: Form Submission

#### 5.1 Submit Logic (`submit_form.rs`)

```rust
pub async fn submit_post(
    content_type: ContentType,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    media_files: Vec<MediaFile>,
    cover_file: Option<MediaFile>,
) -> Result<PostResponse, SubmitError> {
    // 1. Validate inputs
    validate_post_input(&title, &description, content_type, &media_files, &tags)?;

    // 2. Get eligibility token
    let eligibility = check_post_eligibility().await?;
    let token = eligibility.eligibility_token
        .ok_or(SubmitError::NoEligibility)?;

    // 3. Upload files
    let upload_result = upload_post_files(token, media_files).await?;
    let uploaded_files = upload_result.uploaded_files;

    // 4. Create post
    let input = CreatePostInput {
        title,
        mediadescription: description,
        contenttype: content_type,
        media: None,  // Using uploadedFiles instead
        cover: cover_file.map(|c| vec![c.to_base64()]),
        tags: Some(tags),
        uploaded_files: Some(uploaded_files),
    };

    let response = create_post(input).await?;
    
    Ok(response)
}
```

---

### Phase 6: Styling

#### 6.1 SCSS Structure

```
src/style/
├── pages/
│   └── _new_post.scss
├── components/
│   ├── _image_cropper.scss
│   ├── _voice_recorder.scss
│   ├── _video_trimmer.scss
│   ├── _tag_input.scss
│   └── _post_preview.scss
```

Port styles from:
- `css/add-post.css`
- `css/crop.css`
- `css/preview.css`

---

## Testing Plan

### Unit Tests

- [ ] `CreatePostInput` validation
- [ ] Title length constraints (1-63 chars)
- [ ] Description length constraints (0-500 chars)
- [ ] Tag format validation
- [ ] Content type enum serialization

### Component Tests

- [ ] Content type tab switching
- [ ] Form field state management
- [ ] Tag add/remove functionality
- [ ] Image slider navigation
- [ ] Preview mode toggle

### Integration Tests

- [ ] Eligibility token flow
- [ ] File upload with token
- [ ] Post creation with uploaded files
- [ ] Error handling (insufficient balance, rate limit)
- [ ] Redirect on success

### E2E Tests (Playwright)

- [ ] Text post creation flow
- [ ] Image upload and crop flow
- [ ] Audio recording and submission
- [ ] Video upload and trim flow
- [ ] Tag autocomplete interaction
- [ ] Preview mode verification
- [ ] Form validation errors
- [ ] Submit success/failure feedback

---

## Migration Checklist

- [x] Create models for post creation types
- [x] Implement `postEligibility` API call
- [x] Implement `/upload-post` multipart endpoint
- [x] Implement `createPost` mutation
- [x] Implement `searchTags` query
- [x] Build new post page layout
- [x] Build content type tabs sidebar
- [x] Build text post form
- [x] Build image upload + slider
- [ ] Build image cropper modal (canvas) _(UI shell only, draw/crop logic stubbed)_
- [x] Build audio upload dropzone
- [ ] Build voice recorder with visualization _(UI shell only, MediaRecorder stubbed)_
- [ ] Integrate WAV conversion for Chrome/Safari
- [x] Build video upload section
- [ ] Build video trimmer with timeline _(UI shell only, drag handlers empty)_
- [ ] Integrate FFmpeg WASM for video encoding
- [x] Build tag input with autocomplete
- [x] Build tag suggestions dropdown
- [x] Build tag list with remove
- [ ] Implement tag history (localStorage)
- [x] Build full view preview
- [x] Build collapsed card preview
- [x] Implement form validation
- [x] Implement submit flow
- [x] Add loading states and progress
- [x] Add error feedback
- [x] Add success redirect
- [x] Port styles from legacy CSS
- [ ] Test all content types
- [ ] Add E2E tests
- [x] Update feature-convergence.md

---

## Dependencies

- **FFmpeg WASM**: For video trimming/encoding (`@ffmpeg/ffmpeg`)
- **Web Audio API**: For voice recording waveform
- **MediaRecorder API**: For voice capture
- **Canvas API**: For image cropping
- **FileReader API**: For file to base64 conversion
- **localStorage**: For tag history persistence

---

## Notes

### FFmpeg WASM Integration

The legacy implementation uses `js/ffmpeg/ffmpeg/package/dist/umd/ffmpeg.js`. In Leptos, consider:
1. Loading FFmpeg WASM via `wasm-bindgen` interop
2. Using Rust-native video processing if available
3. Web Worker for non-blocking encoding

### Voice Recording Browser Compatibility

- Chrome/Edge: Records as WebM, needs conversion to WAV
- Safari: May need different MIME type handling
- Firefox: Generally works with WebM

### Cover Image Handling

- For audio posts: Background cover is optional
- For video posts: First frame can be extracted as cover
- Cover is sent as base64 while media uses multipart upload

### Implementation Status (2026-04-14)

**~70% structurally complete.** All scaffolding, state management, API calls, routing, models, and styles are in place. Compiler warnings are all unused-variable/import issues from stub implementations.

**Remaining work (browser-side media processing):**
1. **Image Cropper** — Canvas draw logic, image rendering with dark overlay, crop region extraction (currently outputs empty canvas)
2. **Voice Recorder** — MediaRecorder API integration, real-time waveform animation, timer increment, WAV conversion for Chrome/Safari
3. **Video Trimmer** — Timeline handle drag logic, FFmpeg WASM loading/encoding, thumbnail frame generation
4. **Tag History** — localStorage persistence for recently used tags
5. **Drag & Drop** — `dragover`/`drop` event handlers on drop zones (currently click-only)
6. **Responsive Layout** — Mobile breakpoints and tablet adaptations
