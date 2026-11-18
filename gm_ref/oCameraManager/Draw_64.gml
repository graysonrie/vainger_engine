if crt != undefined {
    // CRT effect is active. Use post draw
    exit
}

// Draw GUI — aspect-fit scale to fill window (runs first among GUI due to high instance depth)
if (!pixel_perfect_enabled) {
	return;
}

var ww = window_get_width();
var wh = window_get_height();
if (ww < 1 || wh < 1) {
	return;
}

var sw = surface_get_width(application_surface);
var sh = surface_get_height(application_surface);
if (sw < 1 || sh < 1) {
	return;
}

gpu_set_texfilter(false);
display_set_gui_size(ww, wh);
draw_clear_alpha(c_black, 1);

// HTML5 / GX.games: canvas is the design size; window/surface can disagree slightly (rounding, DPR),
// which made aspect-fit use min() and pillarbox. Stretch to the full canvas instead.
if (os_type == os_operagx) {
	draw_surface_stretched(application_surface, 0, 0, ww, wh);
} else {
	// Fit inside window while preserving aspect; touches two edges (letterbox/pillarbox only on one axis)
	var scale = min(ww / sw, wh / sh);
	if (scale <= 0) {
		display_set_gui_size(CAM_VIEW_W, CAM_VIEW_H);
		return;
	}
	var dw = sw * scale;
	var dh = sh * scale;
	var dx = (ww - dw) * 0.5;
	var dy = (wh - dh) * 0.5;
    
	draw_surface_ext(application_surface, dx, dy, scale, scale, 0, c_white, 1);
}

display_set_gui_size(CAM_VIEW_W, CAM_VIEW_H);