use std::f32;

use crate::features::world::esp::types::{CameraState, RenderSnapshot, Target};
use crate::utils::math;
use egui::{Color32, Context, LayerId, Pos2, Rect, Stroke, StrokeKind, Ui};
use glam::DVec3;

pub struct EspRenderer {
    pub show_debug: bool,
    pub draw_tracers: bool,
    pub draw_boxes: bool,
    pub tracer_color: [f32; 4],
    pub tracer_thickness: f32,
    pub box_color: [f32; 4],
    pub box_thickness: f32,
}

impl Default for EspRenderer {
    fn default() -> Self {
        Self {
            show_debug: false,
            draw_tracers: false,
            draw_boxes: false,
            tracer_color: [1.0, 0.27, 0.27, 0.86],
            tracer_thickness: 1.5,
            box_color: [1.0, 0.27, 0.27, 0.82],
            box_thickness: 1.0,
        }
    }
}

impl EspRenderer {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show_debug, "Show Debug");
        ui.checkbox(&mut self.draw_tracers, "Draw Tracers");
        ui.checkbox(&mut self.draw_boxes, "Draw Boxes");
        ui.color_edit_button_rgba_unmultiplied(&mut self.tracer_color);
        ui.add(egui::Slider::new(&mut self.tracer_thickness, 0.5..=8.0).text("Tracer Thickness"));
        ui.color_edit_button_rgba_unmultiplied(&mut self.box_color);
        ui.add(egui::Slider::new(&mut self.box_thickness, 0.5..=8.0).text("Box Thickness"));
    }

    pub fn is_enabled(&self) -> bool {
        self.draw_tracers || self.draw_boxes // || self.show_debug
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        // For simplicity, enable all features when enabled is true
        // self.show_debug = enabled;
        self.draw_tracers = enabled;
        self.draw_boxes = enabled;
    }

    /// `camera` is [`None`] when capture failed or JNI camera is invalid — world ESP is skipped but debug can still draw.
    pub fn render(&self, ctx: &Context, snapshot: &RenderSnapshot, camera: Option<&CameraState>) {
        // Foreground so lines are composited with the rest of the frame (Background can end up fully obscured depending on pass order).
        let layer_id = LayerId::new(egui::Order::Foreground, egui::Id::new("esp_overlay"));
        let painter = ctx.layer_painter(layer_id);
        let screen_size = ctx.screen_rect().size();

        let Some(camera) = camera.filter(|c| c.valid) else {
            if self.show_debug {
                self.render_debug(&painter, &snapshot.debug_state, screen_size);
            }
            return;
        };

        let tracer_start = Pos2::new(screen_size.x * 0.5, screen_size.y - 42.0);
        let tracer_stroke = Stroke::new(self.tracer_thickness, self.to_color32(&self.tracer_color));

        for target in &snapshot.targets {
            // Interpolate — alpha 1.0 since we captured camera at render time.
            // For full smoothness you'd pass the frame alpha through; 1.0 is safe.
            let alpha = 1.0_f32;
            let interp_x = math::lerp(target.previous_pos.x, target.current_pos.x, alpha);
            let interp_base_y = math::lerp(target.previous_pos.y, target.current_pos.y, alpha);
            let interp_y = interp_base_y + target.eye_height_offset;
            let interp_z = math::lerp(target.previous_pos.z, target.current_pos.z, alpha);
            let interp_pos = DVec3::new(interp_x, interp_y, interp_z);

            // Tracer aims at the entity centre (eye-level offset applied, shift down 0.60 like C++)
            let tracer_world = DVec3::new(interp_x, interp_y - 0.60, interp_z);
            if let Some((screen_x, screen_y)) = math::world_to_screen(
                tracer_world,
                camera,
                screen_size.x,
                screen_size.y,
                true, // clamp offscreen
            ) {
                if self.draw_tracers {
                    painter
                        .line_segment([tracer_start, Pos2::new(screen_x, screen_y)], tracer_stroke);
                    let dot_radius = (self.tracer_thickness + 1.1).max(2.0);
                    painter.circle_filled(
                        Pos2::new(screen_x, screen_y),
                        dot_radius,
                        self.to_color32(&self.tracer_color),
                    );
                }
            }

            if self.draw_boxes {
                // Head and feet world positions (matches C++)
                let head_world_y = interp_base_y + target.eye_height_offset + 0.18;
                let feet_world_y = interp_base_y - 0.08;

                self.draw_target_box(
                    &painter,
                    target,
                    DVec3::new(interp_x, head_world_y, interp_z),
                    DVec3::new(interp_x, feet_world_y, interp_z),
                    camera,
                    screen_size,
                    &interp_pos,
                    interp_base_y,
                );
            }
        }

        if self.show_debug {
            self.render_debug(&painter, &snapshot.debug_state, screen_size);
        }
    }

    fn draw_target_box(
        &self,
        painter: &egui::Painter,
        target: &Target,
        head_world: DVec3,
        feet_world: DVec3,
        camera: &CameraState,
        screen_size: egui::Vec2,
        interp_pos: &DVec3, // eye-height position for distance calc
        interp_base_y: f64, // raw Y for distance calc (matches C++)
    ) {
        // Use world_to_screen_no_clamp — only render box when fully in front (matches C++)
        let Some((head_ndc_x, head_ndc_y)) =
            math::world_to_screen_no_clamp(head_world, camera, screen_size.x, screen_size.y)
        else {
            return;
        };
        let Some((feet_ndc_x, feet_ndc_y)) =
            math::world_to_screen_no_clamp(feet_world, camera, screen_size.x, screen_size.y)
        else {
            return;
        };

        // Cull completely off-screen (matching C++ min/max NDC checks)
        let min_ndc_x = head_ndc_x.min(feet_ndc_x);
        let max_ndc_x = head_ndc_x.max(feet_ndc_x);
        let min_ndc_y = head_ndc_y.min(feet_ndc_y);
        let max_ndc_y = head_ndc_y.max(feet_ndc_y);
        if max_ndc_x < -1.0 || min_ndc_x > 1.0 || max_ndc_y < -1.0 || min_ndc_y > 1.0 {
            return;
        }

        // Clamp NDC to visible range and convert to screen pixels
        let to_screen_x = |ndc: f64| -> f32 {
            ((ndc.clamp(-1.0, 1.0) + 1.0) * 0.5 * screen_size.x as f64) as f32
        };
        let to_screen_y = |ndc: f64| -> f32 {
            ((1.0 - ndc.clamp(-1.0, 1.0)) * 0.5 * screen_size.y as f64) as f32
        };

        let head_sx = to_screen_x(head_ndc_x);
        let head_sy = to_screen_y(head_ndc_y);
        let feet_sx = to_screen_x(feet_ndc_x);
        let feet_sy = to_screen_y(feet_ndc_y);

        let top = head_sy.min(feet_sy);
        let bottom = head_sy.max(feet_sy);
        let height = bottom - top;
        if height < 6.0 {
            return; // too small
        }

        let center_x = (head_sx + feet_sx) * 0.5;
        let half_width = height * 0.19; // proportional to height, matches C++

        // Draw box outline
        let box_rect = Rect::from_min_size(
            Pos2::new(center_x - half_width, top),
            egui::vec2(half_width * 2.0, height),
        );
        painter.rect(
            box_rect,
            0.0,
            Color32::TRANSPARENT,
            Stroke::new(self.box_thickness, self.to_color32(&self.box_color)),
            StrokeKind::Middle,
        );

        // Distance from camera to entity base (matches C++ distanceX/Y/Z calculation)
        let dx = interp_pos.x - camera.current_pos.x;
        let dy = interp_base_y - camera.current_pos.y;
        let dz = interp_pos.z - camera.current_pos.z;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        // Check whether head/feet are both fully on screen (for label)
        let head_fully_on = head_ndc_x.abs() <= 1.0 && head_ndc_y.abs() <= 1.0;
        let feet_fully_on = feet_ndc_x.abs() <= 1.0 && feet_ndc_y.abs() <= 1.0;
        let fully_on_screen = head_fully_on && feet_fully_on;

        if target.health >= 0.0 && distance.is_finite() && fully_on_screen {
            let label = if !target.name.is_empty() {
                format!(
                    "{} [{} HP] [{:.0}m]",
                    target.name, target.health as i32, distance
                )
            } else {
                format!("[{} HP] [{:.0}m]", target.health as i32, distance)
            };

            let font = egui::FontId::proportional(16.0);
            let galley = painter.layout_no_wrap(label.clone(), font.clone(), Color32::WHITE);
            let label_x = center_x - galley.size().x * 0.5;
            let label_y = top - 22.0;
            let pos = Pos2::new(label_x, label_y);

            let text_width = painter.ctx().fonts(|fonts| {
                fonts
                    .layout(label.clone(), font.clone(), Color32::WHITE, f32::INFINITY)
                    .rect
                    .width()
            });

            let text_rect =
                Rect::from_min_size(Pos2::new(label_x, label_y), egui::vec2(text_width, 17.0));

            // Outline (4-directional, matches C++)
            for (ox, oy) in [(-1.0_f32, 0.0_f32), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                painter.rect(
                    text_rect,
                    6.0,
                    Color32::from_gray(50),
                    Stroke::new(1.0, Color32::WHITE),
                    StrokeKind::Outside,
                );

                painter.text(
                    Pos2::new(pos.x + ox, pos.y + oy),
                    egui::Align2::LEFT_TOP,
                    &label,
                    font.clone(),
                    Color32::BLACK,
                );
            }
            painter.text(pos, egui::Align2::LEFT_TOP, &label, font, Color32::WHITE);
        }

        // Distance below box (always shown when finite, matches C++)
        if distance.is_finite() {
            let dist_label = format!("[{:.0}m]", distance);
            let font = egui::FontId::proportional(14.0);
            let galley = painter.layout_no_wrap(dist_label.clone(), font.clone(), Color32::WHITE);
            let dist_pos = Pos2::new(center_x - galley.size().x * 0.5, bottom + 4.0);

            let dist_width = painter.ctx().fonts(|fonts| {
                fonts
                    .layout(
                        dist_label.clone(),
                        font.clone(),
                        Color32::WHITE,
                        f32::INFINITY,
                    )
                    .rect
                    .width()
            });

            let dist_rect = Rect::from_min_size(dist_pos, egui::vec2(dist_width, 17.0));

            painter.rect(
                dist_rect,
                6.0,
                Color32::from_gray(50),
                Stroke::new(1.0, Color32::WHITE),
                StrokeKind::Outside,
            );

            painter.text(
                dist_pos,
                egui::Align2::LEFT_TOP,
                dist_label,
                font,
                Color32::from_rgba_unmultiplied(245, 248, 255, 235),
            );
        }
    }

    fn render_debug(
        &self,
        painter: &egui::Painter,
        debug: &crate::features::world::esp::types::DebugState,
        _screen_size: egui::Vec2,
    ) {
        let mut pos = Pos2::new(15.0, 15.0);
        let font = egui::FontId::monospace(14.0);
        let color = Color32::WHITE;

        let lines = [
            format!(
                "init={} local={} level={} list={}",
                debug.initialized as u8,
                debug.local_player_valid as u8,
                debug.level_valid as u8,
                debug.player_list_valid as u8
            ),
            format!(
                "cam={} rcam={}/{} targets={}",
                debug.camera_valid as u8,
                debug.render_camera_available as u8,
                debug.render_camera_used as u8,
                debug.target_count
            ),
            format!(
                "pos={:.1} {:.1} {:.1}",
                debug.camera_pos.x, debug.camera_pos.y, debug.camera_pos.z
            ),
            format!(
                "yaw/pitch/fov={:.1}/{:.1}/{:.1}",
                debug.yaw_degrees, debug.pitch_degrees, debug.fov_degrees
            ),
            format!("status: {}", debug.last_status),
        ];

        for line in lines {
            painter.text(pos, egui::Align2::LEFT_TOP, line, font.clone(), color);
            pos.y += 16.0;
        }
    }

    fn to_color32(&self, rgba: &[f32; 4]) -> Color32 {
        Color32::from_rgba_unmultiplied(
            (rgba[0].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[1].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[2].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[3].clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
}
