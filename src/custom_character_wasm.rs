//! Browser/WASM bridge for custom character package files.
//!
//! The browser host reads user-selected files and passes relative paths plus
//! Uint8Array contents here. This module owns the in-WASM registry; it never
//! accesses the browser filesystem or DOM directly.

use std::cell::RefCell;

use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};

use crate::custom_character::{CustomCharacterPackage, CustomCharacterRegistry, PackageFile};

thread_local! {
    static REGISTRY: RefCell<CustomCharacterRegistry> = RefCell::new(CustomCharacterRegistry::default());
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Install a package supplied by the browser host as parallel path/byte arrays.
/// Each content item must be a Uint8Array. Paths are relative to the package root.
#[wasm_bindgen]
pub fn install_custom_character_package(paths: Array, contents: Array) -> Result<JsValue, JsValue> {
    if paths.length() != contents.length() {
        return Err(JsValue::from_str("paths and contents must have the same length"));
    }
    let mut files = Vec::with_capacity(paths.length() as usize);
    for index in 0..paths.length() {
        let path = paths.get(index).as_string()
            .ok_or_else(|| JsValue::from_str("package path must be a string"))?;
        let value = contents.get(index);
        if !value.is_instance_of::<Uint8Array>() {
            return Err(JsValue::from_str("package file content must be a Uint8Array"));
        }
        let bytes = Uint8Array::new(&value).to_vec();
        files.push(PackageFile { path, bytes });
    }

    let package = CustomCharacterPackage::from_files(files).map_err(js_error)?;
    let id = package.manifest.id.clone();
    let display_name = package.manifest.display_name.clone();
    let version = package.manifest.version.clone();

    REGISTRY.with(|registry| registry.borrow_mut().install(package).map_err(js_error))?;

    let result = Object::new();
    Reflect::set(&result, &JsValue::from_str("id"), &JsValue::from_str(&id))?;
    Reflect::set(&result, &JsValue::from_str("display_name"), &JsValue::from_str(&display_name))?;
    Reflect::set(&result, &JsValue::from_str("version"), &JsValue::from_str(&version))?;
    Ok(result.into())
}

#[wasm_bindgen]
pub fn enable_custom_character(id: &str) -> Result<(), JsValue> {
    REGISTRY.with(|registry| registry.borrow_mut().enable(id).map_err(js_error))
}

#[wasm_bindgen]
pub fn disable_custom_character(id: &str) -> Result<(), JsValue> {
    REGISTRY.with(|registry| registry.borrow_mut().disable(id).map_err(js_error))
}

#[wasm_bindgen]
pub fn custom_character_is_enabled(id: &str) -> bool {
    REGISTRY.with(|registry| registry.borrow().is_enabled(id))
}

#[wasm_bindgen]
pub fn list_custom_characters() -> Result<Array, JsValue> {
    REGISTRY.with(|registry| {
        let registry = registry.borrow();
        let output = Array::new();
        for package in registry.enabled() {
            let item = Object::new();
            Reflect::set(&item, &JsValue::from_str("id"), &JsValue::from_str(&package.manifest.id))?;
            Reflect::set(&item, &JsValue::from_str("display_name"), &JsValue::from_str(&package.manifest.display_name))?;
            Reflect::set(&item, &JsValue::from_str("version"), &JsValue::from_str(&package.manifest.version))?;
            output.push(&item);
        }
        Ok(output)
    })
}

/// Copy one declared asset into a browser-owned Uint8Array for decoding/rendering.
#[wasm_bindgen]
pub fn get_custom_character_asset(id: &str, asset_id: &str) -> Option<Uint8Array> {
    REGISTRY.with(|registry| {
        let registry = registry.borrow();
        let package = registry.get(id)?;
        let bytes = package.asset_bytes(asset_id)?;
        Some(Uint8Array::from(bytes))
    })
}
