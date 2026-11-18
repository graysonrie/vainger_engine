// Begin Step
if(!view_enabled){
    // Initialize all views and cameras
    for (var i = 0; i < array_length(cameras); i++) {
        camera_set_view_mat(cameras[i], camera_vms[i]);
        camera_set_proj_mat(cameras[i], camera_pms[i]);
    }
    update_viewport_layout();
    view_enabled = true;
}

// Detect per-camera zoom changes, resize views, and keep the current view center
// fixed in world space so zoom pivots smoothly without snapping to the target.
var zoom_changed = false;
var zoom_changed_idxs = [];
var zoom_old_ws = [];
var zoom_old_hs = [];
for (var i = 0; i < array_length(zooms); i++) {
    if (zooms[i] != last_zooms[i]) {
        zoom_changed = true;
        array_push(zoom_changed_idxs, i);
        array_push(zoom_old_ws, camera_get_view_width(cameras[i]));
        array_push(zoom_old_hs, camera_get_view_height(cameras[i]));
        last_zooms[i] = zooms[i];
    }
}
if (zoom_changed) {
    update_viewport_layout();
    for (var j = 0; j < array_length(zoom_changed_idxs); j++) {
        recenter_camera_after_zoom_change(zoom_changed_idxs[j], zoom_old_ws[j], zoom_old_hs[j]);
    }
}

if (!pixel_perfect_enabled) {
    var ww = max(1, floor(window_get_width()));
    var wh = max(1, floor(window_get_height()));
    var sw = surface_get_width(application_surface);
    var sh = surface_get_height(application_surface);
    if (sw != ww || sh != wh) {
        update_viewport_layout();
    }
}
