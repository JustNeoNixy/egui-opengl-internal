use crate::features::world::esp::types::CameraState;
use glam::{DMat4, DVec3, DVec4};

pub fn normalize_degrees(degrees: f64) -> f64 {
    let mut normalized = degrees % 360.0;
    if normalized > 180.0 {
        normalized -= 360.0;
    } else if normalized < -180.0 {
        normalized += 360.0;
    }
    normalized
}

pub fn is_finite(v: f64) -> bool {
    v.is_finite()
}

pub fn lerp(start: f64, end: f64, alpha: f32) -> f64 {
    start + (end - start) * alpha as f64
}

// ---------------------------------------------------------------------------
// Project a world-space point to NDC [-1,1].
// Returns (ndc_x, ndc_y, in_front) or None on failure.
// Matches C++ ProjectToNdc exactly (double-precision view/proj).
// ---------------------------------------------------------------------------
pub fn project_to_ndc(
    world_pos: DVec3,
    camera: &CameraState,
    screen_width: f32,
    screen_height: f32,
) -> Option<(f64, f64, bool)> {
    if !camera.valid || screen_width <= 0.0 || screen_height <= 0.0 {
        return None;
    }

    if !is_finite(world_pos.x)
        || !is_finite(world_pos.y)
        || !is_finite(world_pos.z)
        || !is_finite(camera.current_pos.x)
        || !is_finite(camera.current_pos.y)
        || !is_finite(camera.current_pos.z)
        || !is_finite(camera.yaw_degrees)
        || !is_finite(camera.pitch_degrees)
        || !is_finite(camera.fov_degrees)
    {
        return None;
    }

    let aspect = screen_width as f64 / screen_height as f64;
    let fov_clamped = camera.fov_degrees.clamp(30.0, 170.0);
    let yaw_rad = normalize_degrees(camera.yaw_degrees).to_radians();
    let pitch_rad = normalize_degrees(-camera.pitch_degrees).to_radians();

    let eye = camera.current_pos;
    let forward = DVec3::new(
        -yaw_rad.sin() * pitch_rad.cos(),
        pitch_rad.sin(),
        yaw_rad.cos() * pitch_rad.cos(),
    );
    if !forward.is_finite() || forward.length() <= 0.0001 {
        return None;
    }

    let up = DVec3::Y;
    let view = DMat4::look_at_rh(eye, eye + forward.normalize(), up);
    let projection = DMat4::perspective_rh(fov_clamped.to_radians(), aspect, 0.05, 4096.0);
    let clip = projection * view * DVec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);

    if !clip.is_finite() || clip.w.abs() <= 0.0001 {
        return None;
    }

    let in_front = clip.w > 0.0;
    let inv_w = 1.0 / clip.w.abs();
    let ndc_x = (if in_front { clip.x } else { -clip.x }) * inv_w;
    let ndc_y = (if in_front { clip.y } else { -clip.y }) * inv_w;

    if !is_finite(ndc_x) || !is_finite(ndc_y) {
        return None;
    }

    Some((ndc_x, ndc_y, in_front))
}

// ---------------------------------------------------------------------------
// Compute the off-screen indicator direction in NDC space.
// Matches C++ ComputeOffscreenIndicatorNormalized — uses view-space projection
// so behind-camera targets get a valid edge direction (unlike raw NDC flip).
// ---------------------------------------------------------------------------
fn compute_offscreen_indicator_ndc(
    world_pos: DVec3,
    camera: &CameraState,
    screen_width: f32,
    screen_height: f32,
) -> Option<(f64, f64)> {
    if !camera.valid || screen_width <= 0.0 || screen_height <= 0.0 {
        return None;
    }

    if !is_finite(world_pos.x)
        || !is_finite(world_pos.y)
        || !is_finite(world_pos.z)
        || !is_finite(camera.current_pos.x)
        || !is_finite(camera.current_pos.y)
        || !is_finite(camera.current_pos.z)
        || !is_finite(camera.yaw_degrees)
        || !is_finite(camera.pitch_degrees)
        || !is_finite(camera.fov_degrees)
    {
        return None;
    }

    let aspect = screen_width as f64 / screen_height as f64;
    let yaw_rad = normalize_degrees(camera.yaw_degrees).to_radians();
    let pitch_rad = normalize_degrees(-camera.pitch_degrees).to_radians();

    let eye = camera.current_pos;
    let forward = DVec3::new(
        -yaw_rad.sin() * pitch_rad.cos(),
        pitch_rad.sin(),
        yaw_rad.cos() * pitch_rad.cos(),
    );
    if !forward.is_finite() || forward.length() <= 0.0001 {
        return None;
    }

    let up = DVec3::Y;
    let view = DMat4::look_at_rh(eye, eye + forward.normalize(), up);

    // Transform to view space only (no projection)
    let view_space = view * DVec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);

    if !is_finite(view_space.x) || !is_finite(view_space.y) || !is_finite(view_space.z) {
        return None;
    }

    // Divide X by aspect ratio to account for screen shape (matches C++)
    let mut dir_x = view_space.x / aspect;
    let mut dir_y = view_space.y;

    // If the target is behind the camera (view-space z >= 0 in RH = behind),
    // flip Y so the indicator still points toward the target's side.
    if view_space.z >= 0.0 {
        dir_y = -dir_y;
    }

    // Degenerate case: directly overhead/below with no horizontal offset
    if dir_x.abs() <= 0.000001 && dir_y.abs() <= 0.000001 {
        dir_y = -1.0;
    }

    let scale = 1.0 / dir_x.abs().max(dir_y.abs());
    let ndc_x = dir_x * scale;
    let ndc_y = dir_y * scale;

    if !is_finite(ndc_x) || !is_finite(ndc_y) {
        return None;
    }

    Some((ndc_x, ndc_y))
}

// ---------------------------------------------------------------------------
// Full world-to-screen conversion with optional off-screen clamping.
// Matches C++ WorldToScreen behaviour exactly.
// ---------------------------------------------------------------------------
pub fn world_to_screen(
    world_pos: DVec3,
    camera: &CameraState,
    screen_width: f32,
    screen_height: f32,
    clamp_offscreen: bool,
) -> Option<(f32, f32)> {
    let (ndc_x, ndc_y, in_front) = project_to_ndc(world_pos, camera, screen_width, screen_height)?;

    if !clamp_offscreen && !in_front {
        return None;
    }

    let on_screen = in_front && ndc_x.abs() <= 1.0 && ndc_y.abs() <= 1.0;

    let (mut clamped_x, mut clamped_y) = (ndc_x, ndc_y);

    if clamp_offscreen && !on_screen {
        if !in_front {
            // Use proper view-space indicator (matches C++ exactly)
            let (ix, iy) =
                compute_offscreen_indicator_ndc(world_pos, camera, screen_width, screen_height)?;
            clamped_x = ix;
            clamped_y = iy;
        }

        // Clamp to NDC [-1,1] square
        let abs_x = clamped_x.abs();
        let abs_y = clamped_y.abs();
        if abs_x > 1.0 || abs_y > 1.0 {
            let scale = 1.0 / abs_x.max(abs_y);
            clamped_x *= scale;
            clamped_y *= scale;
        }

        // Height-delta priority check (matches C++)
        let height_delta = world_pos.y - camera.current_pos.y;
        let vertical_priority = height_delta.abs() >= 2.0;

        if vertical_priority {
            let vertical_sign = if height_delta >= 0.0 {
                1.0_f64
            } else {
                -1.0_f64
            };
            // Spread horizontally but clamp, matches C++ clampedX * 2.4 with min-spread guard
            let mut horizontal_spread = clamped_x * 2.4;
            if horizontal_spread.abs() < 0.12 && clamped_x.abs() > 0.02 {
                horizontal_spread = clamped_x.signum() * 0.12;
            }
            clamped_x = horizontal_spread.clamp(-0.78, 0.78);
            clamped_y = vertical_sign;
        } else {
            let side_sign = if clamped_x >= 0.0 { 1.0_f64 } else { -1.0_f64 };
            let side_bias = clamped_x.abs().max(0.0001);
            clamped_y = (clamped_y / side_bias).clamp(-0.72, 0.72);
            clamped_x = side_sign;
        }
    }

    if !clamp_offscreen && (clamped_x.abs() > 1.0 || clamped_y.abs() > 1.0) {
        return None;
    }

    let padding = 18.0_f32;
    let screen_x = ((clamped_x + 1.0) * 0.5 * screen_width as f64) as f32;
    let screen_y = ((1.0 - clamped_y) * 0.5 * screen_height as f64) as f32;

    Some((
        screen_x.clamp(padding, screen_width - padding),
        screen_y.clamp(padding, screen_height - padding),
    ))
}

// ---------------------------------------------------------------------------
// Project to screen without clamping — returns None if not in front.
// Used by draw_target_box for head/feet projection.
// ---------------------------------------------------------------------------
pub fn world_to_screen_no_clamp(
    world_pos: DVec3,
    camera: &CameraState,
    screen_width: f32,
    screen_height: f32,
) -> Option<(f64, f64)> {
    let (ndc_x, ndc_y, in_front) = project_to_ndc(world_pos, camera, screen_width, screen_height)?;
    if !in_front || ndc_x.abs() > 1.0 || ndc_y.abs() > 1.0 {
        return None;
    }
    Some((ndc_x, ndc_y))
}
