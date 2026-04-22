# New Post — Completion Sprint Plan

**Feature:** New Post (#7)
**Priority:** Next — closes the unchecked items on the 🚧 New Post row in [feature-convergence.md](../../feature-convergence.md), enabling promotion 🚧 → ✅
**Status:** 🚧 In Progress — most media handlers actually work; remaining gaps are narrower than the tracker claims
**Created:** 2026-04-22
**Plan Quality Target:** ⭐⭐⭐⭐⭐
**Parent Plan:** [new-post-implementation.md](new-post-implementation.md)

---

## Summary

The new-post page shipped its full structural scope on the 🚧 In Progress tier: page + context, content-type tabs, four content-type forms, image cropper, voice recorder, video trimmer, tag input, live preview, and submit pipeline (eligibility → upload → `createPost`). A code audit (2026-04-22) shows that **several items the parent plan and the convergence tracker mark as "stubbed" are in fact implemented**. This sprint:

1. **Reconciles the documentation drift** — promotes already-finished items from "🔲 stubbed" to "✅ done" in both the parent plan and [feature-convergence.md](../../feature-convergence.md).
2. **Closes the genuine remaining gaps** — real-time waveform, accurate video duration, video frame thumbnails, server-side trim plumbing, tag history, and E2E coverage.
3. **Promotes New Post to ✅ Implemented** in the convergence tracker.

The submit path (eligibility → multipart upload → `createPost`) already works end-to-end against the mock backend (Phase 3 ✅). No backend / mock-backend changes are required for this sprint — every task lives inside `/`.

---

## Doc-Drift Audit (2026-04-22)

Items the parent plan and convergence tracker claim are stubbed but are in fact **implemented and wired**. The first action of this sprint is to mark these correctly so the rest of the gap list is honest.

| Parent-plan claim | Reality (verified in source) | Evidence |
|---|---|---|
| "Image cropping modal — UI shell exists, canvas draw/crop logic is stubbed" | ✅ Fully implemented — `HtmlImageElement` load, `drawImage` preview with overlay, drag/scroll-zoom, output canvas at 1080px wide, PNG data-URL emit | [src/components/new_post/media/image_cropper.rs](../../..//src/components/new_post/media/image_cropper.rs) — `perform_crop`, `draw_preview`, `draw_to_output` |
| "Aspect ratio toggle — UI exists, not wired to canvas" | ✅ Wired — `aspect_ratio` signal feeds both preview canvas height (reactive) and `OUTPUT_WIDTH * ratio.value()` for output | same file, lines 65–73 + `perform_crop` |
| "Cropped image preview — output canvas is empty" | ✅ Output canvas populated; PNG data URL passed to `on_crop` callback | same file, `perform_crop` |
| "Voice recording — UI shell exists, MediaRecorder calls stubbed" | ✅ Implemented — `getUserMedia(audio: true)` → `MediaRecorder` → chunk accumulation → `Blob` → `array_buffer()` → `Vec<u8>` + MIME emitted via `on_recording_complete` | [src/components/new_post/media/voice_recorder.rs](../../..//src/components/new_post/media/voice_recorder.rs) — `start_recording`, `stop_recording` |
| "Recording timer — signal exists, not incremented" | ✅ Incremented — `set_interval_with_callback_and_timeout_and_arguments_0` ticks `elapsed_time` every 1000 ms; cleared on stop | same file, `start_recording` |
| "Playback controls — button exists, handler is empty" | ✅ `toggle_playback` toggles `<audio id="voice-recorder-preview">` `play()` / `pause()` | same file, `toggle_playback` |
| "Record again — resets state, but no actual recording" | ✅ `reset_recording` clears state + `recorded` + `recorder_store`; subsequent record cycle works | same file, `reset_recording` |
| "Start/end handle dragging — handlers are empty stubs" | ✅ Wired — `handle_start_drag` / `handle_end_drag` flip `DragMode`; `handle_timeline_mousemove` updates the active handle with `MIN_DURATION` clamp; `seek_to` updates the `<video>` playhead | [src/components/new_post/media/video_trimmer.rs](../../..//src/components/new_post/media/video_trimmer.rs) |
| "Responsive layout — desktop layout done, no mobile breakpoints" | ✅ Two breakpoints exist (`@media (max-width: 1024px)` and `@media (max-width: 768px)` in [style/new-post.scss](../../..//style/new-post.scss) lines 1338, 1357). Quality of those breakpoints still wants a manual review, but they are not absent. | grep over `style/new-post.scss` |

**Remaining genuine gaps** (tracked as Tasks 2–8 below):

| Parent-plan claim | Reality |
|---|---|
| "Real-time waveform visualization — static SVG placeholder only" | ✅ True. The `<svg>` in `voice_recorder.rs` is a static `M0,50 L200,50` line; no `AnalyserNode` / `requestAnimationFrame` driven path. |
| "Timeline with thumbnail frames — no frame extraction" | ✅ True. The trimmer timeline has no `<canvas>`-extracted frames; it's a flat trim window. |
| "FFmpeg WASM encoding — not integrated" | ✅ True, but **see Task 5** — the cleanest v1 ships the trim window as `(start, end)` server-side rather than client-side transcoding. |
| "Tag history (localStorage) — not implemented" | ✅ True. `tag_input.rs` calls `searchTags` but never persists or surfaces previously used tags. |
| Hardcoded `set_trim_video_duration.set(30.0)` in `video_upload.rs` | ✅ Real bug — no `loadedmetadata` listener to read the actual `<video>.duration`. Trims past the real end of short videos. |

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page** | [src/pages/new_post.rs](../../..//src/pages/new_post.rs) | ~250 | ✅ Auth guard, `NewPostContext`, validation, `build_input` |
| **Route** | [src/app.rs](../../..//src/app.rs) | — | ✅ `/new` registered |
| **Models** | [src/models/post.rs](../../..//src/models/post.rs) | — | ✅ `CreateContentType`, `CreatePostInput`, `MediaFile`, `is_valid_tag` |
| **API — eligibility** | [src/api/posts.rs](../../..//src/api/posts.rs) | — | ✅ `check_post_eligibility` server fn |
| **API — upload** | [src/api/posts.rs](../../..//src/api/posts.rs) | — | ✅ `upload_post_files` (multipart) |
| **API — create** | [src/api/posts.rs](../../..//src/api/posts.rs) | — | ✅ `create_post` mutation |
| **API — search tags** | [src/api/posts.rs](../../..//src/api/posts.rs) | — | ✅ `search_tags` |
| **Component — header** | [src/components/new_post/header.rs](../../..//src/components/new_post/header.rs) | — | ✅ |
| **Component — content type tabs** | [src/components/new_post/content_type_tabs.rs](../../..//src/components/new_post/content_type_tabs.rs) | — | ✅ |
| **Component — form** | [src/components/new_post/form.rs](../../..//src/components/new_post/form.rs) | — | ✅ Title / description / type-specific media area / tags |
| **Component — image upload** | [src/components/new_post/media/image_upload.rs](../../..//src/components/new_post/media/image_upload.rs) | — | ✅ Drop zone, multi-file (≤5), slider, per-image remove |
| **Component — image slider** | [src/components/new_post/media/image_slider.rs](../../..//src/components/new_post/media/image_slider.rs) | — | ✅ |
| **Component — image cropper** | [src/components/new_post/media/image_cropper.rs](../../..//src/components/new_post/media/image_cropper.rs) | ~300 | ✅ Canvas draw, drag, scroll-zoom (0.3–8×), 1:1 / 4:5 toggle, PNG data-URL output |
| **Component — audio upload** | [src/components/new_post/media/audio_upload.rs](../../..//src/components/new_post/media/audio_upload.rs) | — | ✅ File picker (mp3/wav/flac/aac/m4a) + cover slot |
| **Component — voice recorder** | [src/components/new_post/media/voice_recorder.rs](../../..//src/components/new_post/media/voice_recorder.rs) | ~350 | ✅ MediaRecorder, chunked capture, MIME forwarding, preview `<audio>`, timer, play/pause, "record again", "use recording" — **waveform still static** |
| **Component — video upload** | [src/components/new_post/media/video_upload.rs](../../..//src/components/new_post/media/video_upload.rs) | — | ✅ Multi-video (≤2), preview, remove — **`duration` hardcoded to 30.0** |
| **Component — video trimmer** | [src/components/new_post/media/video_trimmer.rs](../../..//src/components/new_post/media/video_trimmer.rs) | ~270 | ✅ Timeline + draggable handles, `MIN_DURATION` clamp, playhead, format labels — **no frame thumbnails, emits times only** |
| **Component — video cover** | [src/components/new_post/media/video_cover.rs](../../..//src/components/new_post/media/video_cover.rs) | — | ✅ Cover slot |
| **Component — drop zone** | [src/components/new_post/media/drop_zone.rs](../../..//src/components/new_post/media/drop_zone.rs) | — | ✅ Generic drag/drop + `read_file_as_bytes` helper |
| **Component — tag input** | [src/components/new_post/tags/tag_input.rs](../../..//src/components/new_post/tags/tag_input.rs) | ~150 | ✅ Autocomplete, `searchTags`, validation, ≤10 cap — **no localStorage history** |
| **Component — tag list** | [src/components/new_post/tags/tag_list.rs](../../..//src/components/new_post/tags/tag_list.rs) | — | ✅ |
| **Component — preview** | [src/components/new_post/preview.rs](../../..//src/components/new_post/preview.rs) | — | ✅ Full + collapsed card preview, "back to edit" |
| **Component — submit** | [src/components/new_post/submit.rs](../../..//src/components/new_post/submit.rs) | ~220 | ✅ Validate → eligibility → multipart upload → `createPost` → toast → navigate. **No `trim_window` plumbed into `CreatePostInput`** |
| **Component — right sidebar** | [src/components/new_post/right_sidebar.rs](../../..//src/components/new_post/right_sidebar.rs) | — | ✅ |
| **Mock Backend** | [packages/mock_backend](../../../packages/mock_backend) | — | ✅ Phase 3 done — `postEligibility`, `/upload-post`, `createPost`, `searchTags` |

### What Remains 🔲

| # | Task | File(s) | Effort | Blocks |
|---|------|---------|--------|--------|
| 1 | **Documentation accuracy pass** — flip the 7 stubbed-but-done checkboxes in [new-post-implementation.md](new-post-implementation.md), and update the New Post row + components rows in [feature-convergence.md](../../feature-convergence.md) (`Image Cropper` 🚧→✅, `Audio Player` 🚧→🟡 with waveform note, `Video Encoder` 🚧→🟡 with FFmpeg note) | `docs/plans/new-post/new-post-implementation.md`, `docs/feature-convergence.md` | S | Honest baseline for tasks 2–8 |
| 2 | **Real-time waveform during recording** — wire `AudioContext` + `AnalyserNode` to the existing `<svg class="waveform">`, drive `<path d="…">` from `getByteFrequencyData` via `requestAnimationFrame`; tear down on stop | `src/components/new_post/media/voice_recorder.rs` | M | — |
| 3 | **Video duration extraction** — replace `set_trim_video_duration.set(30.0)` with a `loadedmetadata` listener on a hidden `<video>` (or read from the `VideoTrimmer`'s own video element on `loadedmetadata` and bubble up). Ensure `MIN_DURATION` clamp + handle positions still hold for short clips. | `src/components/new_post/media/video_upload.rs`, `src/components/new_post/media/video_trimmer.rs` | S | Tasks 4, 5 |
| 4 | **Video frame thumbnails** in the trimmer timeline — extract N frames (e.g. 8) by seeking a hidden `<video>` and `drawImage` to `<canvas>` per tick, render as a strip beneath the trim window. Skip on SSR (`#[cfg(feature = "hydrate")]`). | `src/components/new_post/media/video_trimmer.rs`, `style/new-post.scss` | M | — |
| 5 | **Server-side trim plumbing** — extend `MediaFile` (or add a sibling `MediaTrim { start_secs, end_secs }`) and `CreatePostInput` to carry per-video `(start, end)` tuples; have `VideoUpload::on_trim_complete` persist the times to the context, and have `submit.rs` include them in the upload form data (key e.g. `trim_<filename>`). Mock backend already accepts the field; if the **real** backend doesn't, file an upstream ticket — **do not** ship client-side FFmpeg WASM in v1 (cost, bundle size, mobile reliability). | `src/models/post.rs`, `src/components/new_post/media/video_upload.rs`, `src/components/new_post/submit.rs`, `src/api/posts.rs` | M | — |
| 6 | **Tag history (localStorage)** — persist accepted tags into `localStorage["peer:new-post:tag-history"]` (cap 50, LRU); render up to N=8 chips above the autocomplete suggestions when input is empty/focused; expose a "Clear history" affordance | `src/components/new_post/tags/tag_input.rs`, `src/components/new_post/tags/tag_list.rs` (or new `tag_history.rs`) | S | — |
| 7 | **Mobile responsive review** — manual audit of the existing `1024px` and `768px` breakpoints against the legacy `css/add-post.css` + `css/preview.css`; fix any sidebar collapse, modal width, and crop-canvas scaling issues | `style/new-post.scss` | M | — |
| 8 | **E2E tests** (Playwright) — `end2end/tests/new_post.spec.ts` covering: text post happy path, image post (1 file, no crop), tag add/remove + history persistence, validation errors (empty title, oversize description), eligibility failure path | `end2end/tests/new_post.spec.ts` | L | Promotion ✅ |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

**Critical path:** 1 → (2, 3 in parallel) → (4 depends on 3) → 5 → 6 → 7 → 8

---

## Task Details

### Task 1 — Documentation Accuracy Pass

**Goal:** Make the parent plan and convergence tracker reflect what the code actually does, so the rest of the sprint isn't shadow-boxing already-finished items.

**Steps:**

1. In [new-post-implementation.md](new-post-implementation.md) "Scope → In Scope", flip these checkboxes from `[ ]` to `[x]` and remove the trailing `_(stubbed)_` notes:
   - Image cropping modal
   - Aspect ratio toggle
   - Cropped image preview
   - Voice recording with microphone
   - Recording timer
   - Playback controls
   - Record again
   - Responsive layout
2. Leave **unchecked**: real-time waveform, timeline thumbnail frames, FFmpeg WASM encoding, tag history.
3. In [feature-convergence.md](../../feature-convergence.md) "Components" table, update:
   - `Image Cropper` 🚧 In Progress → ✅ Implemented (note: "canvas draw + drag + zoom + 1:1/4:5 toggle, PNG data-URL output")
   - `Audio Player` 🚧 In Progress → 🟡 Mostly Implemented (note: "MediaRecorder + preview + timer wired; real-time waveform pending — see [sprint](plans/new-post/new-post-completion-sprint.md) Task 2")
   - `Video Encoder` 🚧 In Progress → 🟡 Mostly Implemented (note: "trimmer timeline + handle drag + cover wired; frame thumbnails + server-side trim plumbing pending — see [sprint](plans/new-post/new-post-completion-sprint.md) Tasks 4–5")
4. Bump **Last Updated** to today.

**Acceptance:** A reviewer reading the parent plan and tracker sees the same scope state the code shows.

---

### Task 2 — Real-time Waveform

**Goal:** Replace the static SVG line with a live amplitude curve while the user is recording.

**Approach:**

- During `start_recording` (`#[cfg(feature = "hydrate")]` block), after `getUserMedia` resolves but before `MediaRecorder.start()`, create an `AudioContext`, `MediaStreamAudioSourceNode` from the stream, and an `AnalyserNode` (`fftSize = 256`, `smoothingTimeConstant ≈ 0.8`).
- Store the `AnalyserNode` and a `Uint8Array` buffer in `StoredValue`s (mirrors the existing `recorder_store` / `chunks_store` pattern).
- Drive a `requestAnimationFrame` loop from a `Closure` that:
  - reads `getByteFrequencyData(buffer)`,
  - maps the bins to N points (e.g. 64) on a `0..200 × 0..100` viewBox,
  - mutates the existing `<path>` element's `d` attribute via `set_attribute`.
- Cancel the RAF loop and `audio_context.close()` in `stop_recording` and `reset_recording`. Add to the existing teardown path so recorder restart works cleanly.
- Add a `node_ref=path_ref` to the `<path>` element so the closure can target it; gate the entire block on `#[cfg(feature = "hydrate")]`.

**Acceptance:**
- Mic recording shows a moving waveform; speaking louder visibly amplifies the curve.
- Stopping or resetting tears the loop down (no AudioContext leak — verify with DevTools).
- SSR build still compiles (`cargo build --features ssr`).

**Out of scope:** Persisting the rendered waveform to the post; saving an image of the waveform.

---

### Task 3 — Video Duration Extraction

**Goal:** Use the real `<video>.duration` instead of the hardcoded 30.0 in `video_upload.rs::start_trim`.

**Approach (chosen — read inside the trimmer):**

- Remove `set_trim_video_duration` plumbing entirely; have `VideoTrimmer` accept just `video_src: String` (no `duration` prop).
- Inside `VideoTrimmer`, hold `duration: RwSignal<f64>` and attach a `loadedmetadata` listener to `video_ref` that writes `(*video).duration()` into it.
- All current `duration` usages (`start_time`, `end_time`, `seek_to`, `min_gap`) become `move ||`-style calls on the signal.
- Until the metadata fires, render an "Loading video…" state (don't allow `Trim` button).

**Acceptance:**
- A 7-second clip shows a 7-second timeline; handles cannot drag past the real end.
- A clip shorter than `MIN_DURATION` (3 s) renders the existing warning instead of crashing.
- The `set_trim_video_duration.set(30.0)` line is gone.

---

### Task 4 — Frame Thumbnails

**Goal:** Render a strip of N (8) extracted frames behind the trim window so the user can pick the cut visually.

**Approach:**

- After `loadedmetadata` (Task 3), sample N times from `0..duration`.
- For each sample, set `(*video).set_current_time(t)`, await a `seeked` event (`Promise` + `JsFuture`), `drawImage(video, …)` onto a small offscreen `<canvas>` (e.g. 80×45), then `to_data_url("image/jpeg", 0.6)`.
- Store the resulting URLs in a `RwSignal<Vec<String>>` and render as `<img>`s positioned `absolute` across the timeline track (CSS `flex` strip beneath the overlays).
- Run extraction once per loaded `video_src`; do not re-extract on every drag.
- Hide the strip while extraction is in progress; show a thin shimmer placeholder.

**Acceptance:**
- Selecting a video produces 8 evenly spaced thumbnails within ~1 s for a 30 s clip.
- Repositioning the trim handles does **not** retrigger extraction.
- Switching to a different video re-extracts.

**Risks / mitigations:**
- Some browsers (mobile Safari) won't decode without a user gesture — extraction is gated behind the existing "Trim" button click, which is already a gesture.
- Cross-origin sources would taint the canvas — videos in this flow are always blob URLs from the local file picker, so this is not a practical risk.

---

### Task 5 — Server-side Trim Plumbing

**Goal:** Persist the trimmer's `(start, end)` selection through the upload pipeline.

**Approach:**

1. Add `pub trim: Option<(f64, f64)>` to `MediaFile` in `src/models/post.rs`.
2. In `video_upload.rs::on_trim_complete`, instead of just clearing the modal, write `(start, end)` onto the matching `MediaFile` in the context (use `set_media_files.update(…)` with the active index).
3. In `submit.rs`, when building the multipart upload:
   - For each `MediaFile` with `Some((start, end))`, append two extra fields to the form: `trim_start_<filename>` = start, `trim_end_<filename>` = end.
4. Update `upload_post_files` server fn signature to accept the trims (`Vec<(String, f64, f64)>` keyed by filename) and forward them to `/upload-post`.
5. Verify against the mock backend that the request shape is accepted (Phase 3 currently ignores extra multipart fields, which is the desired behaviour for v1 — no test-failure risk).
6. Add a `// TODO(backend):` note in the parent plan documenting that the **real** PHP / Rust backend must implement server-side `ffmpeg` cut on these fields. File a tracking ticket; do **not** block this sprint on it.

**Acceptance:**
- Trimming a video and submitting produces an upload request with `trim_start_*` / `trim_end_*` fields visible in DevTools.
- Untrimmed videos do not include the fields.
- Mock backend continues to accept the upload (no 4xx).

**Explicitly out of scope:** Client-side FFmpeg WASM transcoding. Rationale: ~30 MB WASM bundle, slow on mobile, error-prone across codecs. Server-side cut is correct architecture.

---

### Task 6 — Tag History (localStorage)

**Goal:** Surface previously used tags so a returning user can re-pick them with one click.

**Approach:**

- New helper module `src/utils/tag_history.rs` with `read() -> Vec<String>`, `push(tag: &str)` (LRU dedupe, cap 50), `clear()`. SSR no-op fallback.
- In `tag_input.rs`:
  - On `select_suggestion` and on the `Enter` accept path, call `tag_history::push(&value)`.
  - When the input is focused **and** empty, render a "Recently used" row of up to 8 chips above the suggestions list.
  - Add a small "Clear history" link beneath the chips.
- Storage key: `peer:new-post:tag-history` (namespace consistent with other site keys).

**Acceptance:**
- Adding `rust` to two posts causes `rust` to appear in "Recently used" on the third.
- "Clear history" empties the row immediately.
- Hard reload preserves history (verify in DevTools → Application → Local Storage).
- Private/Incognito mode silently no-ops (no panic on `localStorage` unavailable).

---

### Task 7 — Mobile Responsive Review

**Goal:** Confirm the two existing breakpoints actually produce a usable mobile layout — fix any obvious gaps.

**Manual checklist (run in Chrome DevTools device toolbar):**

- iPhone 12 (390×844): content-type tabs collapse / move to top, form fills width, crop modal fits viewport, trimmer timeline scrolls or compresses cleanly, preview modal does not overflow.
- iPad (768×1024): two-column variant remains usable; sidebar tabs reachable.
- Compare against the legacy `newpost.php` rendering on the same viewports (the legacy CSS lives at [css/add-post.css](../../../css/add-post.css)).
- Fix only what is broken — do not refactor working CSS.

**Acceptance:** A short note appended to this sprint's "Implementation Notes" section listing what was changed and what needed no change.

---

### Task 8 — E2E Tests

**Goal:** Lock in the happy paths and the most common failure modes.

**File:** `end2end/tests/new_post.spec.ts` — follow the structure of [chat.spec.ts](../../..//end2end/tests/chat.spec.ts) and [wallet.spec.ts](../../..//end2end/tests/wallet.spec.ts).

**Cases:**

1. **Text post happy path** — login → `/new` → fill title + description → submit → assert toast + redirect to `/post/<id>` (or `/profile`).
2. **Image post happy path** — switch to Image tab → upload one fixture image → confirm crop → submit → assert success.
3. **Tag add/remove + history persistence** — type `playwright`, accept; verify chip appears; reload page; verify `playwright` shows in "Recently used".
4. **Validation — empty title** — submit with empty title → assert error toast + page stays on `/new`.
5. **Validation — oversize description** — paste 501 chars → assert error.
6. **Eligibility failure** — using a fresh test user with insufficient tokens (mock seed), assert the eligibility failure message surfaces in a toast and the form stays editable.

**Acceptance:** All 6 cases pass against the mock backend in CI.

---

## Definition of Done

- [ ] All 8 tasks complete, with each Acceptance section satisfied.
- [ ] `cargo build --features ssr` and `cargo build --features hydrate --target wasm32-unknown-unknown` succeed with **zero warnings**.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] All 6 new Playwright cases pass locally (`pnpm exec playwright test new_post`).
- [ ] [feature-convergence.md](../../feature-convergence.md) updated:
  - **Pages** table: New Post 🚧 In Progress → ✅ Implemented (with sprint link).
  - **Components** table: Image Cropper, Audio Player, Video Encoder updated per Task 1.
  - **Summary counts** recalculated: ✅ 13 → 14, 🚧 2 → 1; convergence ratio updated.
  - **Migration Priority** #7 marked complete.
  - **Changelog** entry under today's date.
- [ ] Parent plan [new-post-implementation.md](new-post-implementation.md) status: 🚧 → ✅; the 8 checkboxes from Task 1 flipped; the 4 remaining "future work" items left unchecked are explicitly listed under "Out of Scope" or this sprint's deferred section.

---

## Out of Scope (Deferred)

- **Client-side FFmpeg WASM transcoding** — see Task 5 rationale. Server cut is preferred.
- **Live waveform persisted as cover art** — separate feature.
- **Drafts / autosave / scheduled posts / templates** — already in parent plan's "Future Work".
- **Multi-video trimming** (each of the up-to-2 videos getting its own trim) — Task 5 already accommodates this in the data model, but Task 4's frame strip only renders for the currently active trim. A second-pass UX pass can come later.
- **Real backend (`peer_backend`) wiring of `trim_start_*` / `trim_end_*` fields** — out of frontend repo scope; tracked as upstream ticket.

---

## References

- Parent plan: [new-post-implementation.md](new-post-implementation.md)
- Convergence tracker: [feature-convergence.md](../../feature-convergence.md)
- Sister sprint patterns: [chat-completion-sprint.md](../chat/chat-completion-sprint.md), [forgot-password-completion-sprint.md](../forgot-password/forgot-password-completion-sprint.md), [view-post-completion-sprint.md](../view-post/view-post-completion-sprint.md)
- Mock backend Phase 3 (posts/content): [phase-3-posts-content.md](../mock-backend/phase-3-posts-content.md)
- Legacy implementations: [newpost.php](../../../newpost.php), [js/add_post.js](../../../js/add_post.js), [js/crop.js](../../../js/crop.js), [js/voiceRecorderApi.js](../../../js/voiceRecorderApi.js), [js/ffmpeg/](../../../js/ffmpeg/)
