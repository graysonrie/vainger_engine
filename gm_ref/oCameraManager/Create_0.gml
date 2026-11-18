var cam_default_w = 400
var cam_default_h = 240
var default_win_scale = 2

var cam_scheme = variable_global_get(GLOBALVAR_CAMERASCHEME)
if cam_scheme != undefined {
    switch cam_scheme {
        case CameraScheme.MegaMan:
            cam_default_w = 256
            cam_default_h = 224
            default_win_scale = 2
        break
        case CameraScheme.PizzaTower:
            cam_default_w = 960
            cam_default_h = 540
            default_win_scale = 1
        break
        case CameraScheme.ShovelKnight:
            cam_default_w = 400
            cam_default_h = 240
            default_win_scale = 2
        break
        case CameraScheme.Antonblast:
            cam_default_w = 384
            cam_default_h = 216
            default_win_scale = 2
        break
    }
}

// Create
// --- Desired sizes ---
CAM_VIEW_W = cam_default_w;
CAM_VIEW_H = cam_default_h;

CAM_VIEW_W_HALF = CAM_VIEW_W / 2
CAM_VIEW_H_HALF = CAM_VIEW_H / 2


res = 2;  // For pixel-perfect: res must divide evenly into window_scale (3/3=1, perfect integer scaling)

window_scale = default_win_scale;  // Initial window scale multiplier (instance variable)

PORT_W     = CAM_VIEW_W * window_scale;
PORT_H     = CAM_VIEW_H * window_scale;

// When pixel-perfect is off, Begin Step refreshes the app surface if the window size changes.
// Pixel-perfect mode no longer forces the window size; use snap_window_to_design_scale() for integer snap.
allow_window_resize = true;

/// When true: fixed-size app surface, scaled to fit the window in Draw GUI (aspect preserved, nearest filter).
/// When false: app surface matches the window; views fill the window.
pixel_perfect_enabled = true;



/// @param {bool} enabled Pass false to allow any window size and scale the framebuffer to fit.
function set_pixel_perfect_enabled(enabled) {
	pixel_perfect_enabled = enabled;
	gpu_set_texfilter(!enabled);
	display_set_gui_size(CAM_VIEW_W, CAM_VIEW_H);
	if (enabled) {
		display_set_gui_maximise(-1, -1);
	} else {
		display_set_gui_maximise();
	}
	update_viewport_layout();
	application_surface_draw_enable(!enabled);
}

/// Snaps the window to CAM_VIEW × window_scale (optional menu action; not applied every frame).
function snap_window_to_design_scale() {
	window_set_size(CAM_VIEW_W * window_scale, CAM_VIEW_H * window_scale);
	window_center();
}

// For splitscreen, you can scale the camera up rather than take away from the
// view sizing. 1 means default (take away from view sizing)
_use_scale_up_for_splitscreen = false

/// Whether scale-up is enabled.
/// You can read the current horizontal scaling value (1 or 2) with `get_current_horizontal_scaling`
/// @param {bool} use
function set_use_scale_up_for_splitscreen(use){
    _use_scale_up_for_splitscreen = use
    if use {
        _current_horizontal_scaling = 2
    }
    else {
        _current_horizontal_scaling = 1
    }
}

function get_current_horizontal_scaling() {
    if _use_scale_up_for_splitscreen and get_num_cameras_active() == 2 {
        return 2
    }
    return 1 // normal scale
}

function get_num_cameras_active(){
    return array_length(camera_pms)
}

// Splitscreen support: arrays for multiple cameras and targets
targets = [];  // Array of target objects
cameras = [];  // Array of camera handles
camera_vms = [];  // Array of view matrices
camera_pms = [];  // Array of projection matrices
camera_positions = [];  // Array of {x, y} for each camera
camera_lerp_factors = [];  // Array of lerp factors per camera
camera_bounds = [];  // Array of CameraBounds structs for each camera
zooms = [];  // Camera zoom (controls what you see, not window size)
// Track last zoom value to detect changes
last_zooms = [];

// Initialize view matrices (same for all cameras)
vm = matrix_build_lookat(0,0,-10,0,0,0,0,1,0);

disp_w = display_get_width();
disp_h = display_get_height();

// Screen shake arrays - one per camera (array of arrays)
camera_screen_shake_strength = [];  // Array of shake strength arrays
camera_screen_shake_damping = [];  // Array of shake damping arrays

lerp_factor = 0.1;  // Default lerp factor for new cameras

// Smooth bounds interpolation - one per camera
smooth_bounds_interpolation = false;  // Enable smooth interpolation when bounds change
bounds_lerp_factor = 0.15;  // Lerp factor for bounds interpolation (higher = faster)
camera_target_bounds = [];  // Array of target CameraBounds structs to interpolate towards

// Optional: window & GUI to match the viewport
if (pixel_perfect_enabled) {
	window_set_size(PORT_W, PORT_H);
}
gpu_set_texfilter(!pixel_perfect_enabled);
display_set_gui_size(CAM_VIEW_W, CAM_VIEW_H);
if (pixel_perfect_enabled) {
	display_set_gui_maximise(-1, -1);
} else {
	display_set_gui_maximise();
}
update_viewport_layout();

// High depth: Draw GUI runs before lower-depth instances (background scale-to-fit pass).
depth = 99999;
application_surface_draw_enable(!pixel_perfect_enabled);

/// Returns a rectangle that represents the AREA ON THE SCREEN that the camera occupies
/// @param {real} camera_idx
function get_containing_rect_for_cam(camera_idx) {
    var num_players = array_length(targets)
    var xoff, yoff
    if camera_idx == 0 {
        xoff = 0
        yoff = 0
        if num_players == 1 {
            return new Rect(xoff,yoff,xoff + CAM_VIEW_W,yoff + CAM_VIEW_H)
        }
        else if num_players == 2 {
            return new Rect(xoff,yoff,xoff + CAM_VIEW_W,yoff + CAM_VIEW_H_HALF)
        } else {
            return new Rect(xoff,yoff,xoff + CAM_VIEW_W_HALF,yoff + CAM_VIEW_H_HALF)
        }
    }
    if camera_idx == 1 {
        xoff = 0
        yoff = CAM_VIEW_H_HALF
        if num_players == 1 {
            show_error("Cannot get 2p bounds w only 1 player", true)
        }
        else if num_players == 2 {
            // on bottom of screen
            return new Rect(xoff,yoff,xoff + CAM_VIEW_W,yoff + CAM_VIEW_H_HALF)
        } else {
            // on top right of screen
            xoff = CAM_VIEW_W_HALF
            yoff = 0
            return new Rect(xoff,yoff,xoff + CAM_VIEW_W_HALF,yoff + CAM_VIEW_H_HALF)
        }
    }
    if camera_idx == 2 {
        xoff = 0
        yoff = CAM_VIEW_H_HALF
        if num_players <= 2 {
            show_error("Cannot get 3p bounds w only 1-2 player", true)
        }
        // on bottom left of screen
        return new Rect(xoff,yoff,xoff + CAM_VIEW_W_HALF,yoff + CAM_VIEW_H_HALF)
    }
    if camera_idx == 3 {
        xoff = CAM_VIEW_W_HALF
        yoff = CAM_VIEW_H_HALF
        if num_players <= 3 {
            show_error("Cannot get 4p bounds w only 1-3 player", true)
        }
        // on bottom right of screen
        return new Rect(xoff,yoff,xoff + CAM_VIEW_W_HALF,yoff + CAM_VIEW_H_HALF)
    }
    show_error("inv camera idx " + string(camera_idx), true)
}

function get_sticky_coords(sticky_obj, camera_idx = 0){
	if (camera_idx >= array_length(camera_positions)) {
		return {xx: 0, yy: 0};
	}
	return {
		xx: camera_positions[camera_idx].x + sticky_obj.x,
		yy: camera_positions[camera_idx].y + sticky_obj.y
	}
}

function get_sticky_coords_xy(xx, yy, camera_idx = 0){
	if (camera_idx >= array_length(camera_positions)) {
		return {xx: 0, yy: 0};
	}
	return {
		xx: camera_positions[camera_idx].x + xx,
		yy: camera_positions[camera_idx].y + yy
	}
}

function lock_to_point(point, camera_idx = 0){
	show_debug_message("locking to point {0}",point)
	if (camera_idx >= 0 && camera_idx < array_length(camera_bounds)) {
		camera_bounds[camera_idx].xmin = point.xx;
		camera_bounds[camera_idx].xmax = point.xx;
		camera_bounds[camera_idx].ymin = point.yy;
		camera_bounds[camera_idx].ymax = point.yy;
	}
}

/// Sets the camera bounds for a specific camera and sets up smooth interpolation
/// @param {Struct.CameraBounds} new_bounds - The new bounds to set
/// @param {Real} camera_idx - The camera index (default: 0)
function set_camera_bounds(new_bounds, camera_idx = 0){
	if (camera_idx < 0 || camera_idx >= array_length(camera_bounds)) {
		show_error("set_camera_bounds: Invalid camera index " + string(camera_idx), false);
		return;
	}

	// Ensure target bounds array is properly sized
	while (camera_idx >= array_length(camera_target_bounds)) {
		var default_bounds = new CameraBounds();
		array_push(camera_target_bounds, default_bounds);
	}

	// Check if this is the first time setting bounds (current bounds are default/infinite)
	var current_bounds = camera_bounds[camera_idx];
	var is_initial_set = (current_bounds.xmin == -infinity && current_bounds.xmax == infinity &&
	                      current_bounds.ymin == -infinity && current_bounds.ymax == infinity);

	// Check if target bounds are also default/infinite (meaning bounds were never set before)
	var target_bounds_exist = (camera_idx < array_length(camera_target_bounds));
	var target_is_default = false;
	if (target_bounds_exist) {
		var current_target = camera_target_bounds[camera_idx];
		target_is_default = (current_target.xmin == -infinity && current_target.xmax == infinity &&
		                    current_target.ymin == -infinity && current_target.ymax == infinity);
	} else {
		target_is_default = true;
	}

	// Set target bounds to the new bounds (this is what we'll interpolate towards)
	camera_target_bounds[camera_idx] = new_bounds;

	// If smooth interpolation is disabled, or this is the initial set, set bounds immediately
	// Otherwise, bounds will interpolate towards target in Step event
	if (!smooth_bounds_interpolation || is_initial_set || target_is_default) {
		camera_bounds[camera_idx] = new_bounds;
	}
}

/// Bounds are usually authored for a full single-player view (CAM_VIEW_W × CAM_VIEW_H at current zoom).
/// Split-screen uses smaller camera view sizes; finite xmax/ymax must be widened so the same room edges still line up.
function get_effective_camera_clamp_bounds_for_idx(idx) {
    return get_effective_camera_clamp_bounds(camera_bounds[idx],idx)
}

/// Bounds are usually authored for a full single-player view (CAM_VIEW_W × CAM_VIEW_H at current zoom).
/// Split-screen uses smaller camera view sizes; finite xmax/ymax must be widened so the same room edges still line up.
/// @param {Struct.CameraBounds} bounds
/// @param {Real} camera_idx
function get_effective_camera_clamp_bounds(bounds, camera_idx) {
	var actual_w = camera_get_view_width(cameras[camera_idx]);
	var actual_h = camera_get_view_height(cameras[camera_idx]);
	var eff_xmax = bounds.xmax;
	var eff_ymax = bounds.ymax;
	if (bounds.xmax != infinity) {
		eff_xmax = bounds.xmax - actual_w;
	}
	if (bounds.ymax != infinity) {
		eff_ymax = bounds.ymax - actual_h;
	}
	return {
		xmin: bounds.xmin,
		xmax: eff_xmax,
		ymin: bounds.ymin,
		ymax: eff_ymax
	};
}

/// Returns the width in pixels of the given camera with zoom factored in
function get_width(idx = 0){
	if (idx >= array_length(cameras)) return 0
    return camera_get_view_width(cameras[idx])
}

/// Returns the width in pixels of the given camera with zoom factored in
function get_height(idx = 0){
	if (idx >= array_length(cameras)) return 0
    return camera_get_view_height(cameras[idx])
}

function get_view_height(){
     return view_get_hport(0)
}

function snap_to_obj(obj, camera_idx = 0){
	if (camera_idx >= array_length(cameras)) return;
	var cam_w = get_width(camera_idx);
	var cam_h = get_height(camera_idx);
	camera_positions[camera_idx].x = obj.x - cam_w/2;
	camera_positions[camera_idx].y = obj.y - cam_h/2;
	camera_set_view_pos(cameras[camera_idx], camera_positions[camera_idx].x, camera_positions[camera_idx].y);
}

/// Checks if the object is out of view for every camera
function is_offscreen_all_cams(obj, obj_width_offset = 0, obj_height_offset = 0) {
    for (var i = 0; i < array_length(cameras); i++) {
    	if is_offscreen_x(obj,i,obj_width_offset) or is_offscreen_y(obj,i,obj_height_offset){
            return true
        }
    }
    return false
}

function is_offscreen_y(obj, camera_idx = 0, obj_height_offset=0){
	if (camera_idx >= array_length(cameras)) return false;
	var obj_height = sprite_get_height(obj.sprite_index);
	var h_half = get_height(camera_idx) / 2;
	var yy = camera_positions[camera_idx].y + h_half; // center the camera
	if obj.y + obj_height_offset - obj_height > yy + h_half { // on bottom
		return true;
	}
	if obj.y + obj_height_offset + obj_height < yy - h_half { // on top
		return true;
	}
	return false;
}

function is_offscreen_bottom_of_screen(obj, camera_idx = 0, obj_height_offset=0){
	if (camera_idx >= array_length(cameras)) return false;
	var obj_height = sprite_get_height(obj.sprite_index);
	var h_half = get_height(camera_idx) / 2;
	var yy = camera_positions[camera_idx].y + h_half; // center the camera
	if obj.y + obj_height_offset - obj_height > yy + h_half { // on bottom
		return true;
	}
	return false;
}

function is_offscreen_left_of_screen(obj, camera_idx = 0, obj_width_offset = 0){
	if (camera_idx >= array_length(cameras)) return false;
	var obj_w = sprite_get_width(obj.sprite_index);
	var w_half = get_width(camera_idx) / 2;
	var xx = camera_positions[camera_idx].x + w_half; // center the camera
	if obj.x + obj_width_offset + obj_w < xx - w_half { // on left
		return true;
	}
	return false;
}

function is_offscreen_x(obj, camera_idx = 0, obj_width_offset = 0){
	if (camera_idx >= array_length(cameras)) return false;
	var obj_w = sprite_get_width(obj.sprite_index);
	var w_half = get_width(camera_idx) / 2;
	var xx = camera_positions[camera_idx].x + w_half; // center the camera
	if obj.x + obj_width_offset - obj_w > xx + w_half { // on right
		return true;
	}
	if obj.x + obj_width_offset + obj_w < xx - w_half { // on left
		return true;
	}
	return false;
}

function get_zoom_for_idx(idx) {
    if (idx < 0 || idx >= array_length(zooms)) return 1;
    return zooms[idx]
}

/// Sets the zoom for a specific camera. Begin Step detects the change and refreshes the viewport.
/// @param {Real} idx
/// @param {Real} new_zoom
function set_zoom_for_idx(idx, new_zoom) {
    if (idx < 0 || idx >= array_length(zooms)) {
        show_error("set_zoom_for_idx: Invalid camera index " + string(idx), false);
        return;
    }
    if (new_zoom <= 0) {
        show_error("set_zoom_for_idx: zoom must be > 0 (got " + string(new_zoom) + ")", false);
        return;
    }
    zooms[idx] = new_zoom
}

/// Repositions a camera after its view size changes from a zoom update so the same
/// world point stays at the view center. Follow lag is preserved; Step continues
/// lerping toward the target from the adjusted position.
/// @param {Real} camera_idx
/// @param {Real} old_view_w
/// @param {Real} old_view_h
function recenter_camera_after_zoom_change(camera_idx, old_view_w, old_view_h) {
	if (camera_idx < 0 || camera_idx >= array_length(cameras)) return;

	var new_w = camera_get_view_width(cameras[camera_idx]);
	var new_h = camera_get_view_height(cameras[camera_idx]);
	var pos = camera_positions[camera_idx];
	var center_x = pos.x + old_view_w * 0.5;
	var center_y = pos.y + old_view_h * 0.5;
	var new_x = center_x - new_w * 0.5;
	var new_y = center_y - new_h * 0.5;

	if (camera_idx < array_length(camera_bounds)) {
		var eb = get_effective_camera_clamp_bounds(camera_bounds[camera_idx], camera_idx);
		new_x = clamp(new_x, eb.xmin, eb.xmax);
		new_y = clamp(new_y, eb.ymin, eb.ymax);
	}

	camera_positions[camera_idx].x = new_x;
	camera_positions[camera_idx].y = new_y;
	camera_set_view_pos(cameras[camera_idx], new_x, new_y);
}

/// Add a target to the splitscreen system
/// @returns {Real} the tgt idx
function add_target(tgt){
	return scrCameraManager_AddTarget(tgt)
}

// Remove a target from the splitscreen system
function remove_target(tgt){
  	scrCameraManager_RemoveTarget(tgt)
}

// Update viewport layout based on number of targets
function update_viewport_layout(){
	scrCameraManager_UpdateViewportLayout()
}

// Legacy function for backwards compatibility
function set_target(tgt){
	// Destroy any existing cameras before clearing arrays
	for (var i = 0; i < array_length(cameras); i++) {
		if (cameras[i] != -1) {
			camera_destroy(cameras[i]);
		}
	}

	// Destroy any existing cameras before clearing arrays
	for (var i = 0; i < array_length(cameras); i++) {
		if (cameras[i] != -1) {
			camera_destroy(cameras[i]);
		}
	}

	// Clear existing targets and add this one
	targets = [];
	cameras = [];
	camera_vms = [];
	camera_pms = [];
	camera_positions = [];
	camera_lerp_factors = [];
	camera_bounds = [];
	camera_target_bounds = [];
	camera_screen_shake_strength = [];
	camera_screen_shake_damping = [];

	add_target(tgt);
}

// Helper function to trigger screen shake for a specific target
// Usage: oCamera.shake_for_target(player1, 5, 0.1, 0)
function shake_for_target(target_obj, strength, damping, layer = 0){
	var idx = array_indexof(targets, target_obj);
	if (idx == -1) {
		show_debug_message("shake_for_target: Target object not found in camera manager");
		return;
	}
	camera_screen_shake_strength[idx][layer] = strength;
	camera_screen_shake_damping[idx][layer] = damping;
}

// Helper function to trigger screen shake for a camera by index
// Usage: oCamera.shake_for_camera(0, 5, 0.1, 0)
function shake_for_camera(camera_idx, strength, damping, layer = 0){
	if (camera_idx < 0 || camera_idx >= array_length(camera_screen_shake_strength)) {
		show_debug_message("shake_for_camera: Invalid camera index: " + string(camera_idx));
		return;
	}
	camera_screen_shake_strength[camera_idx][layer] = strength;
	camera_screen_shake_damping[camera_idx][layer] = damping;
}

/// Gets the width (in pixels) of the given camera. Default = 0
function get_width_of_cam(cam_idx=0){
    return camera_get_view_width(cameras[cam_idx])
}

/// Gets the height (in pixels) of the given camera. Default = 0
function get_height_of_cam(cam_idx=0){
    var e = oCameraManager
    return camera_get_view_height(cameras[cam_idx])
}

var crt_enable = variable_global_get(GLOBALVAR_CAMERAEFFECT_CRT_ENABLE) ?? false

crt = undefined
if crt_enable {
    var demo_config = variable_global_get(GLOBALVAR_CAMERAEFFECT_CRT_CONFIG)
    if demo_config == undefined {
        show_error("CRT Config does not exist", true)
    }
    crt = new CRT(demo_config)
}
