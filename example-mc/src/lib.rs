//! Entry point, exports public API

mod core;
mod error;
mod features;
mod jni;
mod utils;

use crate::features::combat::hit_result;
use crate::features::combat::velocity::Velocity;
use crate::features::world::esp::Esp;
use crate::utils::renderer::EspRenderer;
use core::client::Minecraft;
use features::world::{
    fast_break::FastBreak, fast_place::FastPlace, fullbright::Fullbright, sprint::Sprint,
};
use once_cell::sync::OnceCell;

static ESP: OnceCell<Esp> = OnceCell::new();
static ESP_RENDERER: OnceCell<parking_lot::Mutex<EspRenderer>> = OnceCell::new();

static FAST_PLACE: OnceCell<FastPlace> = OnceCell::new();
static FAST_BREAK: OnceCell<FastBreak> = OnceCell::new();
static VELOCITY: OnceCell<Velocity> = OnceCell::new();
static SPRINT: OnceCell<Sprint> = OnceCell::new();
static FULLBRIGHT: OnceCell<Fullbright> = OnceCell::new();

use egui::{Color32, Context};

use egui_opengl_internal::OpenGLApp;
use retour::static_detour;
use std::{mem::transmute, sync::Once};
use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{WindowFromDC, HDC},
        System::Diagnostics::Debug::OutputDebugStringW,
        UI::WindowsAndMessaging::{CallWindowProcW, SetWindowLongPtrA, GWLP_WNDPROC, WNDPROC},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum CheatCategory {
    #[default]
    Combat,
    Movement,
    Player,
    Render,
}

static mut SELECTED_CATEGORY: CheatCategory = CheatCategory::Combat;

/// Logs to both console and debugger (DebugView, Visual Studio Output)
pub fn esp_log(msg: impl AsRef<str>) {
    let msg_str = msg.as_ref();

    // Output to Windows debugger (DebugView)
    let wide: Vec<u16> = msg_str.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        OutputDebugStringW(windows::core::PCWSTR(wide.as_ptr()));
    }

    // Also output to console for immediate visibility
    println!("{}", msg_str);
}

#[no_mangle]
extern "system" fn DllMain(hinst: usize, reason: u32) -> i32 {
    if reason == 1 {
        std::thread::spawn(move || unsafe { main_thread(hinst) });
    }

    if reason == 0 {
        unsafe {
            WglSwapBuffersHook.disable().unwrap();
            let wnd_proc = OLD_WND_PROC.unwrap().unwrap();
            let _: Option<WNDPROC> = Some(transmute(SetWindowLongPtrA(
                APP.get_window(),
                GWLP_WNDPROC,
                wnd_proc as usize as _,
            )));

            utils::free_console();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    1
}

static mut APP: OpenGLApp<i32> = OpenGLApp::new();
static mut OLD_WND_PROC: Option<WNDPROC> = None;
static mut EXITING: bool = false;
static mut HUD_VISIBLE: bool = true;

type _FnWglSwapBuffers = unsafe extern "system" fn(HDC) -> HRESULT;
static_detour! {
    static WglSwapBuffersHook: unsafe extern "system" fn(HDC) -> HRESULT;
}

fn hk_wgl_swap_buffers(hdc: HDC) -> HRESULT {
    unsafe {
        let window = WindowFromDC(hdc);

        static INIT: Once = Once::new();
        // Inside hk_wgl_swap_buffers, after INIT.call_once:
        INIT.call_once(|| {
            esp_log("[example-mc] wglSwapBuffers hooked.\n");
            println!("wglSwapBuffers successfully hooked.");

            APP.init_default(hdc, window, ui);

            OLD_WND_PROC = Some(transmute(SetWindowLongPtrA(
                window,
                GWLP_WNDPROC,
                hk_wnd_proc as usize as _,
            )));
        });

        if !APP.get_window().eq(&window) {
            SetWindowLongPtrA(window, GWLP_WNDPROC, hk_wnd_proc as usize as _);
        }

        if let Some(fp) = FAST_PLACE.get() {
            fp.on_update();
        }
        if let Some(fb) = FAST_BREAK.get() {
            fb.on_update();
        }
        if let Some(v) = VELOCITY.get() {
            v.on_update();
        }
        if let Some(s) = SPRINT.get() {
            s.on_update();
        }
        if let Some(esp) = ESP.get() {
            esp.tick();
        }
        if let Some(fb) = FULLBRIGHT.get() {
            fb.on_update();
        }
        hit_result::tick_trigger_bot();
        APP.render(hdc);
        WglSwapBuffersHook.call(hdc)
    }
}

unsafe extern "system" fn hk_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        println!("CallWindowProcW successfully hooked.");
    });

    let egui_wants_input = APP.wnd_proc(msg, wparam, lparam);
    if egui_wants_input {
        return LRESULT(1);
    }

    CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam)
}

fn ui(ctx: &Context, _: &mut i32) {
    // Initialize nerd fonts (only once)
    static INIT_FONTS: Once = Once::new();
    INIT_FONTS.call_once(|| {
        let mut fonts = egui::FontDefinitions::default();
        egui_nerdfonts::add_to_fonts(&mut fonts, egui_nerdfonts::Variant::Regular);
        ctx.set_fonts(fonts);
    });
    // Check for right shift key toggle
    let right_shift_pressed = unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
            windows::Win32::UI::Input::KeyboardAndMouse::VK_RSHIFT.0 as i32,
        ) as i16
            & 0x8000u16 as i16
    } != 0;

    static mut PREV_RSHIFT_STATE: bool = false;
    let rshift_just_pressed = unsafe {
        let current = right_shift_pressed;
        let prev = PREV_RSHIFT_STATE;
        PREV_RSHIFT_STATE = current;
        current && !prev
    };

    if rshift_just_pressed {
        unsafe {
            HUD_VISIBLE = !HUD_VISIBLE;
        }
    }

    // Only show UI if HUD is visible
    if unsafe { HUD_VISIBLE } {
        custom_window_frame(ctx, "Cheat Menu", |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // ── LEFT SIDEBAR ──
                        ui.vertical(|ui| {
                            ui.set_width(160.0);
                            ui.set_min_height(260.0);

                            egui::Frame::NONE
                                //.fill(Color32::from_gray(45))
                                .inner_margin(10.0)
                                .corner_radius(8.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.add_space(5.0);

                                        ui.heading("Name");

                                        ui.add_space(10.0);

                                        // Category buttons
                                        for (category, label) in [
                                            (CheatCategory::Combat, "Combat"),
                                            (CheatCategory::Movement, "Movement"),
                                            (CheatCategory::Player, "Player"),
                                            (CheatCategory::Render, "Render"),
                                        ] {
                                            let selected = unsafe { SELECTED_CATEGORY } == category;
                                            if ui.add(
                                                egui::Button::new(label)
                                                    .fill(if selected { Color32::from_rgb(100, 60, 120) } else { Color32::from_gray(50) })
                                                    .min_size(egui::Vec2::new(ui.available_width(), 35.0))
                                                    .corner_radius(4.0)
                                            ).clicked() {
                                                unsafe { SELECTED_CATEGORY = category };
                                            }
                                            ui.add_space(4.0);
                                        }

                                        ui.add_space(180.0);

                                        if ui.add(
                                            egui::Button::new("Self Destruct")
                                                .fill(Color32::from_rgb(180, 50, 50))
                                                .min_size(egui::Vec2::new(ui.available_width(), 35.0))
                                                .corner_radius(4.0)
                                        ).clicked() {
                                            // Self destruct logic
                                            unsafe { crate::EXITING = true };
                                        }
                                    });
                                });
                        });

                        ui.add_space(5.0);

                        // ── MAIN CONTENT ──
                        ui.vertical(|ui| {
                            ui.set_min_height(260.0);

                            egui::Frame::NONE
                                //.fill(Color32::from_gray(50))
                                .inner_margin(15.0)
                                .corner_radius(8.0)
                                .show(ui, |ui| {
                                    let available_width = ui.available_width();
                                    let available_height = ui.available_height();

                                    let title = match unsafe { SELECTED_CATEGORY } {
                                        CheatCategory::Combat => "Combat",
                                        CheatCategory::Movement => "Movement",
                                        CheatCategory::Player => "Player",
                                        CheatCategory::Render => "Render",
                                    };

                                    // Simple centering using horizontal layout with spacing
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(title).size(16.0).color(Color32::WHITE));
                                    });
                                    ui.add_space(10.0);

                                    egui::ScrollArea::vertical()
                                        .max_height(available_height - 40.0)
                                        .show(ui, |ui| {
                                            ui.set_max_width(available_width - 10.0);

                                            match unsafe { SELECTED_CATEGORY } {
                                                CheatCategory::Combat => {
                                                    // ── Trigger Bot ──
                                                    if let Some(_hit_result) = crate::features::combat::HitResultBundle::get_instance() {
                                                        let trigger_bot = FeatureToggle::new("Trigger Bot");
                                                        trigger_bot.show(
                                                            ui,
                                                            hit_result::trigger_bot_enabled(),
                                                            |enabled| hit_result::set_trigger_bot_enabled(enabled),
                                                            |ui| {
                                                                ui.label("Coming Soon!");
                                                            }
                                                        );
                                                    }

                                                    // ── Anti Knockback (Velocity) ──
                                                    if let Some(velocity) = VELOCITY.get() {
                                                        let anti_kb = FeatureToggle::new("Anti Knockback");
                                                        anti_kb.show(
                                                            ui,
                                                            velocity.is_enabled(),
                                                            |enabled| velocity.set_enabled(enabled),
                                                            |ui| {
                                                                ui.vertical(|ui| {
                                                                    // Strength slider
                                                                    ui.horizontal(|ui| {
                                                                        ui.label("Strength:");
                                                                        ui.add_space(8.0);

                                                                        let mut strength = velocity.get_strength();
                                                                        if ui.add(egui::DragValue::new(&mut strength).speed(0.1).range(0.0..=1.0)).changed() {
                                                                            velocity.set_strength(strength);
                                                                        }
                                                                    });

                                                                    // Vertical only
                                                                    ui.horizontal(|ui| {
                                                                        let mut vertical_only = velocity.vertical_only();
                                                                        if ui.checkbox(&mut vertical_only, "Vertical Only").changed() {
                                                                            velocity.set_vertical_only(vertical_only);
                                                                        }
                                                                    });

                                                                    // Only when hurt
                                                                    ui.horizontal(|ui| {
                                                                        let mut only_hurt = velocity.only_when_hurt();
                                                                        if ui.checkbox(&mut only_hurt, "Only When Hurt").changed() {
                                                                            velocity.set_only_when_hurt(only_hurt);
                                                                        }
                                                                    });
                                                                });
                                                            }
                                                        );
                                                    }
                                                }

                                                CheatCategory::Movement => {
                                                    if let Some(sprint) = SPRINT.get() {
                                                        let sprint_toggle = FeatureToggle::new("Sprint");
                                                        sprint_toggle.show(
                                                            ui,
                                                            sprint.is_enabled(),
                                                            |enabled| sprint.set_enabled(enabled),
                                                            |ui| {
                                                                ui.label("Coming Soon!");
                                                            }
                                                        );
                                                    }

                                                    let fly_enabled = {
                                                        let fly = crate::features::world::fly::get_fly();
                                                        fly.is_enabled()
                                                    };
                                                    let fly_toggle = FeatureToggle::new("Fly");
                                                    fly_toggle.show(
                                                        ui,
                                                        fly_enabled,
                                                        |enabled| {
                                                            let fly = crate::features::world::fly::get_fly();
                                                            fly.set_enabled(enabled);
                                                        },
                                                        |ui| {
                                                            ui.horizontal(|ui| {
                                                                ui.label("Speed:");
                                                                ui.add_space(8.0);

                                                                let mut speed = {
                                                                    let fly = crate::features::world::fly::get_fly();
                                                                    fly.get_speed()
                                                                };
                                                                if ui.add(egui::DragValue::new(&mut speed).speed(0.01).range(0.01..=2.0)).changed() {
                                                                    let fly = crate::features::world::fly::get_fly();
                                                                    fly.set_speed(speed);
                                                                }
                                                            });
                                                        }
                                                    );
                                                }

                                                CheatCategory::Player => {
                                                    if let Some(fp) = FAST_PLACE.get() {
                                                        let fast_place = FeatureToggle::new("Fast Place");
                                                        fast_place.show(
                                                            ui,
                                                            fp.is_enabled(),
                                                            |enabled| fp.set_enabled(enabled),
                                                            |ui| {
                                                                ui.label("Coming Soon!");
                                                            }
                                                        );
                                                    }
                                                    if let Some(fb) = FAST_BREAK.get() {
                                                        let fast_break = FeatureToggle::new("Fast Break");
                                                        fast_break.show(
                                                            ui,
                                                            fb.is_enabled(),
                                                            |enabled| fb.set_enabled(enabled),
                                                            |ui| {
                                                                ui.vertical(|ui| {
                                                                    // Speed multiplier
                                                                    ui.horizontal(|ui| {
                                                                        ui.label("Speed:");
                                                                        ui.add_space(8.0);

                                                                        let mut speed = fb.get_speed();
                                                                        if ui.add(egui::DragValue::new(&mut speed).speed(0.1).range(1.0..=10.0)).changed() {
                                                                            fb.set_speed(speed);
                                                                        }
                                                                    });

                                                                    // Tool only option
                                                                    ui.horizontal(|ui| {
                                                                        let mut tool_only = fb.tool_only();
                                                                        if ui.checkbox(&mut tool_only, "Tool Only").changed() {
                                                                            fb.set_tool_only(tool_only);
                                                                        }
                                                                    });

                                                                    // Creative mode only
                                                                    ui.horizontal(|ui| {
                                                                        let mut creative_only = fb.creative_only();
                                                                        if ui.checkbox(&mut creative_only, "Creative Only").changed() {
                                                                            fb.set_creative_only(creative_only);
                                                                        }
                                                                    });
                                                                });
                                                            }
                                                        );
                                                    }
                                                }

                                                CheatCategory::Render => {
                                                    if let Some(renderer) = ESP_RENDERER.get() {
                                                        let is_enabled = {
                                                            let renderer_guard = renderer.lock();
                                                            renderer_guard.is_enabled()
                                                        };

                                                        let esp_toggle = FeatureToggle::new("ESP");
                                                        esp_toggle.show(
                                                            ui,
                                                            is_enabled,
                                                            |enabled| {
                                                                let mut renderer_guard = renderer.lock();
                                                                renderer_guard.set_enabled(enabled);
                                                            },
                                                            |ui| {
                                                                let mut renderer_guard = renderer.lock();
                                                                renderer_guard.ui(ui);
                                                            }
                                                        );
                                                    }

                                                    if let Some(fb) = FULLBRIGHT.get() {
                                                        let fullbright_toggle = FeatureToggle::new("Fullbright");
                                                        fullbright_toggle.show(ui, fb.is_enabled(), |enabled| fb.set_enabled(enabled), |ui| {
                                                            ui.label("Coming Soon!");
                                                        });
                                                    }
                                                }

                                            }
                                        });
                                    });
                                });
                        });
                    });
        });
    }

    if let (Some(esp), Some(renderer)) = (ESP.get(), ESP_RENDERER.get()) {
        crate::features::world::esp::render_esp_overlay(ctx, esp, &renderer.lock());
    }
}

unsafe fn main_thread(_hinst: usize) {
    utils::alloc_console();

    // Build the JVMTI class cache BEFORE Minecraft::init().
    // This enumerates ALL loaded classes regardless of classloader, which is
    // required for Fabric where Minecraft classes live in Knot's classloader
    // and cannot be found via JNI's FindClass from a native thread.
    if crate::jni::class_cache::init() {
        println!(
            "[Cheat] JVMTI class cache built (Fabric={}, Vanilla={})",
            crate::jni::class_cache::is_fabric(),
            crate::jni::class_cache::is_vanilla()
        );
    } else {
        println!("[Cheat] Warning: JVMTI class cache failed — Fabric class loading may not work");
    }

    if Minecraft::init() {
        println!("[Cheat] Minecraft wrapper initialized");
        let _ = FAST_PLACE.set(FastPlace::new());
        println!("[Cheat] FastPlace initialized");
        let _ = FAST_BREAK.set(FastBreak::new());
        let _ = VELOCITY.set(Velocity::new());
        let _ = SPRINT.set(Sprint::new());
        let _ = FULLBRIGHT.set(Fullbright::new());
        println!("[Cheat] Sprint initialized");
        Sprint::init();
        println!("[Cheat] Sprint JNI initialized");
        Minecraft::with(|mc| {
            let _ = crate::core::local_player::LocalPlayer::init(mc);

            if crate::features::world::fullbright::try_init(mc) {
                println!("[Cheat] Fullbright JNI initialized");
            } else {
                println!("[Cheat] Fullbright init failed");
            }
        });
        if crate::features::world::fast_break::try_init() {
            println!("[Cheat] FastBreak JNI initialized");
        } else {
            println!("[Cheat] FastBreak init failed");
        }
        if crate::features::combat::velocity::try_init() {
            println!("[Cheat] Velocity JNI initialized");
        } else {
            println!("[Cheat] Velocity init failed");
        }
        if hit_result::try_init_bundle() {
            println!("[Cheat] HitResult / trigger-bot JNI initialized");
        } else {
            println!("[Cheat] HitResult init failed (mappings / game version)");
        }
    }

    let _ = ESP.set(Esp::new());
    let _ = ESP_RENDERER.set(parking_lot::Mutex::new(EspRenderer::default()));

    let proc_addr = utils::get_proc_address(&mut APP);
    unsafe {
        let target_fn: unsafe extern "system" fn(HDC) -> HRESULT = std::mem::transmute(proc_addr);
        match WglSwapBuffersHook.initialize(target_fn, hk_wgl_swap_buffers) {
            Ok(_) => {
                if let Err(e) = WglSwapBuffersHook.enable() {
                    esp_log(&format!(
                        "[example-mc] Failed to enable static detour: {:?}\n",
                        e
                    ));
                } else {
                    esp_log("[example-mc] Hook setup successful\n");
                }
            }
            Err(e) => esp_log(&format!(
                "[example-mc] Failed to initialize hook: {:?}\n",
                e
            )),
        }
    }

    if EXITING {
        utils::unload(&mut APP);
    }

    loop {
        if EXITING {
            utils::unload(&mut APP);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn custom_window_frame(
    ctx: &egui::Context,
    _title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::{Area, Frame, Id, Margin, ScrollArea, Vec2};

    let window_size = Vec2::new(800.0, 600.0);
    let screen_rect = ctx.screen_rect();
    let window_pos = screen_rect.min + (screen_rect.size() - window_size) / 2.0;

    // ✅ Area constrained to screen bounds
    Area::new(Id::new("custom_window"))
        .fixed_pos(window_pos)
        .constrain_to(screen_rect) // Keep window on-screen
        .show(ctx, |ui| {
            // Force exact window size
            ui.set_min_size(window_size);
            ui.set_max_size(window_size);

            // Draw window frame
            Frame::new()
                .fill(ctx.style().visuals.window_fill())
                .corner_radius(10)
                .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
                .inner_margin(Margin::ZERO)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // ── Content Area ──
                        let content_height = window_size.y - 16.0;

                        ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .max_height(content_height)
                            .show(ui, |ui| {
                                // ✅ Critical: constrain width to prevent horizontal overflow
                                ui.set_max_width(window_size.x - 24.0);

                                // ✅ Optional: manual clipping for extra safety
                                let clip_rect = ui.max_rect().intersect(screen_rect);
                                ui.set_clip_rect(clip_rect);

                                Frame::NONE
                                    .inner_margin(Margin::same(8))
                                    .show(ui, add_contents);
                            });
                    });
                });
        });
}

fn color_lerp(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (c1.r() as f32 + (c2.r() as f32 - c1.r() as f32) * t) as u8,
        (c1.g() as f32 + (c2.g() as f32 - c1.g() as f32) * t) as u8,
        (c1.b() as f32 + (c2.b() as f32 - c1.b() as f32) * t) as u8,
        (c1.a() as f32 + (c2.a() as f32 - c1.a() as f32) * t) as u8,
    )
}

fn toggle_switch(ui: &mut egui::Ui, enabled: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::Vec2::new(40.0, 22.0), egui::Sense::click());

    let clicked = response.clicked();
    let new_value = if clicked { !enabled } else { enabled };

    let t = ui.ctx().animate_bool(response.id, new_value);

    let rect = rect.shrink(1.0);
    let radius = rect.height() / 2.0;

    // 🟢 Green when ON, 🔴 Red when OFF
    let on_color = Color32::from_rgb(50, 180, 50);
    let off_color = Color32::from_rgb(180, 50, 50);
    let bg_color = color_lerp(off_color, on_color, t);

    ui.painter().rect_filled(rect, radius, bg_color);

    let circle_radius = radius - 4.0;
    let start_x = rect.left() + circle_radius + 2.0;
    let end_x = rect.right() - circle_radius - 2.0;
    let circle_x = start_x + (end_x - start_x) * t;

    ui.painter().circle_filled(
        egui::pos2(circle_x, rect.center().y),
        circle_radius,
        Color32::WHITE,
    );

    new_value
}

struct FeatureToggle {
    name: &'static str,
    settings_id: egui::Id,
}

impl FeatureToggle {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            settings_id: egui::Id::new(format!("settings_{}", name)),
        }
    }

    fn show(
        &self,
        ui: &mut egui::Ui,
        is_enabled: bool,
        on_toggle: impl FnOnce(bool),
        add_settings: impl FnOnce(&mut egui::Ui),
    ) {
        // Get current settings state
        let mut show_settings =
            ui.data_mut(|d| *d.get_temp_mut_or_default::<bool>(self.settings_id));

        // Main feature row
        let _response = egui::Frame::NONE
            .fill(if is_enabled {
                Color32::from_rgb(60, 80, 60)
            } else {
                Color32::from_rgb(50, 50, 50)
            })
            .inner_margin(egui::vec2(10.0, 8.0))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Gear button for settings
                    let gear_response = ui.add(
                        egui::Button::new(
                            egui::RichText::new(egui_nerdfonts::regular::GEAR).size(16.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    );

                    if gear_response.clicked() {
                        show_settings = !show_settings;
                        ui.data_mut(|d| d.insert_temp(self.settings_id, show_settings));
                    }

                    ui.add_space(8.0);

                    // Feature name
                    ui.label(
                        egui::RichText::new(self.name)
                            .size(14.0)
                            .color(Color32::WHITE),
                    );

                    // Spacer
                    ui.add_space(ui.available_width() - 120.0);

                    // Toggle switch
                    let new_enabled = toggle_switch(ui, is_enabled);
                    if new_enabled != is_enabled {
                        on_toggle(new_enabled);
                    }
                });
            })
            .response;

        // Settings panel
        if show_settings {
            ui.vertical(|ui| {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .fill(Color32::from_gray(48))
                    .inner_margin(egui::vec2(16.0, 8.0))
                    .corner_radius(4.0)
                    .show(ui, add_settings);
                ui.add_space(4.0);
            });
        }
    }
}

// ... (rest of the code remains the same)
