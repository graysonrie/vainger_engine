use bevy::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Default)]
pub(super) struct CanvasColorSpaceState {
    frames: u8,
    done: bool,
}

/// Stops the browser from remapping the WebGL canvas on wide-gamut displays.
///
/// The game writes sRGB bytes. Chrome then converts that canvas into the
/// monitor gamut, so a color such as 205, 15, 75 shows up near 191, 47, 75.
/// Tagging the drawing buffer as Display P3 leaves those bytes alone on a P3
/// panel, which is how the unmanaged desktop window already behaves.
///
/// This waits until wgpu has created the context. Calling `getContext` first
/// takes the canvas and the renderer never starts.
pub(super) fn tag_canvas_color_space(mut state: Local<CanvasColorSpaceState>) {
    if state.done {
        return;
    }
    state.frames = state.frames.saturating_add(1);
    if state.frames < 8 {
        return;
    }

    match try_tag() {
        Ok(TagResult::NotWideGamut) => {
            info!("Display is not wide-gamut; leaving the WebGL canvas in sRGB");
            state.done = true;
        }
        Ok(TagResult::Tagged) => {
            info!("WebGL drawing buffer tagged display-p3");
            state.done = true;
        }
        Err(reason) => {
            if state.frames >= 120 {
                warn!("Left the WebGL canvas in sRGB: {reason}");
                state.done = true;
            }
        }
    }
}

enum TagResult {
    NotWideGamut,
    Tagged,
}

fn try_tag() -> Result<TagResult, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    if !is_wide_gamut(&window)? {
        return Ok(TagResult::NotWideGamut);
    }

    let document = window.document().ok_or_else(|| "no document".to_string())?;
    let element = document
        .query_selector("canvas")
        .map_err(|err| format!("canvas query failed: {err:?}"))?
        .ok_or_else(|| "no canvas".to_string())?;
    let canvas: web_sys::HtmlCanvasElement = element
        .dyn_into()
        .map_err(|_| "element is not a canvas".to_string())?;

    // Same context id wgpu already created. A different id, or this call
    // before the first present, would make getContext return null later.
    let context = canvas
        .get_context("webgl2")
        .map_err(|err| format!("getContext failed: {err:?}"))?
        .ok_or_else(|| "webgl2 context is not ready".to_string())?;

    let name = wasm_bindgen::JsValue::from_str("drawingBufferColorSpace");
    let value = wasm_bindgen::JsValue::from_str("display-p3");
    js_sys::Reflect::set(&context, &name, &value)
        .map_err(|err| format!("could not set drawingBufferColorSpace: {err:?}"))?;
    let current = js_sys::Reflect::get(&context, &name)
        .map_err(|err| format!("could not read drawingBufferColorSpace: {err:?}"))?;
    let current = current.as_string().unwrap_or_default();
    if current != "display-p3" {
        return Err(format!("drawingBufferColorSpace is {current}"));
    }
    Ok(TagResult::Tagged)
}

fn is_wide_gamut(window: &web_sys::Window) -> Result<bool, String> {
    let list = window
        .match_media("(color-gamut: p3), (color-gamut: rec2020)")
        .map_err(|err| format!("matchMedia failed: {err:?}"))?;
    Ok(list.is_some_and(|query| query.matches()))
}
