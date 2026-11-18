// Step
// Handle multiple cameras following their respective targets
for (var i = 0; i < array_length(targets); i++) {
    if (!instance_exists(targets[i])) {
        continue;  // Skip if target doesn't exist
    }

    var target = targets[i];
    var current_lerp = camera_lerp_factors[i] * GAMESPEED;

    // --- Compute look-ahead offset toward the mouse while shooting ---
    var mx = device_mouse_x(0);
    var my = device_mouse_y(0);

    // vector from player to mouse in ROOM space
    var dx = mx - target.x;
    var dy = my - target.y;

    var dist = point_distance(0, 0, dx, dy);
    var nx = 0, ny = 0;
    if (dist > 0) {
        nx = dx / dist;
        ny = dy / dist;
    }

    // --- Camera target (player + look-ahead), then center view on it ---
    var vw = camera_get_view_width(cameras[i]);
    var vh = camera_get_view_height(cameras[i]);

    var target_x = (target.x) - (vw * 0.5);
    var target_y = (target.y) - (vh * 0.5);

    // Smoothly move the camera using lerp
    var current_x = camera_get_view_x(cameras[i]);
    var current_y = camera_get_view_y(cameras[i]);
    var new_x = lerp(current_x, target_x, current_lerp);
    var new_y = lerp(current_y, target_y, current_lerp);

    // Apply bounds (per-camera) with optional smooth interpolation
    if (i < array_length(camera_bounds)) {
        var bounds = camera_bounds[i];

        // Interpolate bounds towards target bounds if smooth interpolation is enabled
        if (smooth_bounds_interpolation && i < array_length(camera_target_bounds)) {
            var target_bounds = camera_target_bounds[i];

            // Check if bounds need to interpolate
            var bounds_match = (bounds.xmin == target_bounds.xmin && bounds.xmax == target_bounds.xmax &&
                               bounds.ymin == target_bounds.ymin && bounds.ymax == target_bounds.ymax);

            if (!bounds_match) {
                // Interpolate bounds towards target
                var bounds_lerp = bounds_lerp_factor * GAMESPEED;
                if (bounds_lerp <= 0) {
                    bounds_lerp = 0.001;  // Ensure some interpolation happens
                }

                bounds.xmin = lerp(bounds.xmin, target_bounds.xmin, bounds_lerp);
                bounds.xmax = lerp(bounds.xmax, target_bounds.xmax, bounds_lerp);
                bounds.ymin = lerp(bounds.ymin, target_bounds.ymin, bounds_lerp);
                bounds.ymax = lerp(bounds.ymax, target_bounds.ymax, bounds_lerp);

                // Update camera_bounds array
                camera_bounds[i] = bounds;
            }
        }

        // Calculate clamped position (authored for 1p view size; expand max limits for split-screen)
        var eb = get_effective_camera_clamp_bounds(bounds, i);
        var clamped_x = clamp(new_x, eb.xmin, eb.xmax);
        var clamped_y = clamp(new_y, eb.ymin, eb.ymax);
        new_x = clamped_x;
        new_y = clamped_y;
    }

    // --- Screen shake - independent per camera ---
    for (var j = 0; j < array_length(camera_screen_shake_damping[i]); j++) {
        var shake = camera_screen_shake_strength[i][j];
        var damp  = camera_screen_shake_damping[i][j];
        if (shake > 0) {
            new_x += random_range(-shake, shake);
            new_y += random_range(-shake, shake);
            camera_screen_shake_strength[i][j] = lerp(shake, 0, damp);
        }
    }

    // Store + apply
    camera_positions[i].x = new_x;
    camera_positions[i].y = new_y;
    camera_set_view_pos(cameras[i], new_x, new_y);
}