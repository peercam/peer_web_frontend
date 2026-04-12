//! File reading utilities for media upload.

/// Read a file as base64.
#[cfg(feature = "hydrate")]
pub async fn read_file_as_base64(file: &web_sys::File) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let reader = web_sys::FileReader::new().map_err(|_| "Failed to create FileReader")?;
    let file_clone = file.clone();
    
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader_clone = reader.clone();
        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            if let Ok(result) = reader_clone.result() {
                resolve.call1(&wasm_bindgen::JsValue::NULL, &result).ok();
            }
        }) as Box<dyn FnMut()>);
        
        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            reject.call0(&wasm_bindgen::JsValue::NULL).ok();
        }) as Box<dyn FnMut()>);
        
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        
        onload.forget();
        onerror.forget();
    });

    reader
        .read_as_data_url(&file_clone)
        .map_err(|_| "Failed to read file")?;

    let result = JsFuture::from(promise)
        .await
        .map_err(|_| "Failed to read file")?;

    result
        .as_string()
        .ok_or_else(|| "Failed to convert result to string".to_string())
}

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
