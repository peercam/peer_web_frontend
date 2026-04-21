//! File reading utilities for media upload.

/// Read a file as ArrayBuffer bytes.
#[cfg(feature = "hydrate")]
pub async fn read_file_as_bytes(file: &web_sys::File) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;

    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| "Failed to read file")?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}
