mod cache;
mod core;
pub mod types;

pub use core::Esp;

use egui::Context;

/// Call this in your egui UI update loop to render ESP overlay
pub fn render_esp_overlay(
    ctx: &Context,
    esp: &Esp,
    renderer: &crate::utils::renderer::EspRenderer,
) {
    let snapshot = esp.get_snapshot();

    // Empty targets is normal (solo world, no other players). Still draw debug / status when enabled.
    if !renderer.show_debug && snapshot.targets.is_empty() {
        return;
    }

    let render_cam = esp.capture_camera_state();
    let camera = render_cam
        .as_ref()
        .filter(|c| c.valid)
        .map(|c| &c.camera_state);

    renderer.render(ctx, &snapshot, camera);
}
