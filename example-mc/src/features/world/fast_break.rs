//! Port of C++ `FastBreak` — lowers block break delay via `GameMode.destroyDelay`.

use crate::core::client::Minecraft;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static READY: AtomicBool = AtomicBool::new(false);

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[FastBreak] JNI error at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

/// Resolve Minecraft `gameMode` field and `GameMode` destroy fields once.
pub fn try_init() -> bool {
    let Some(mut env) = JniEnvironment::get_current_or_attach("FastBreak init") else {
        return false;
    };

    let is_fabric = class_cache::is_fabric();

    let Some(success) = Minecraft::with(|mc| {
        let Some(mc_class) = mc.class_ref() else {
            return false;
        };

        // ── Minecraft.gameMode field (fabric vs vanilla) ──────────────────────
        let game_mode_field = if is_fabric {
            &mappings::fabric_minecraft::GAME_MODE
        } else {
            &mappings::minecraft::GAME_MODE
        };

        if env
            .get_field_id(mc_class, game_mode_field.name, game_mode_field.signature)
            .is_err()
            || clear_exception(&mut env, "Minecraft.gameMode field")
        {
            esp_log("[FastBreak] GameMode field ID not found\n");
            return false;
        }

        // ── GameMode class (fabric vs vanilla) ────────────────────────────────
        let game_mode_class_name = if is_fabric {
            mappings::fabric_classes::GAME_MODE
        } else {
            mappings::classes::GAME_MODE
        };

        let gm_local = if let Some(cached) = class_cache::find_class(game_mode_class_name) {
            esp_log("[FastBreak] Found GameMode class via JVMTI cache\n");
            cached
        } else {
            match env.find_class(game_mode_class_name) {
                Ok(c) => {
                    esp_log("[FastBreak] Found GameMode class via FindClass\n");
                    c
                }
                Err(_) => {
                    let _ = clear_exception(&mut env, "FindClass(GameMode)");
                    esp_log("[FastBreak] GameMode class not found\n");
                    return false;
                }
            }
        };

        // ── GameMode destroy fields (fabric vs vanilla) ───────────────────────
        let (destroy_delay, destroy_progress) = if is_fabric {
            (
                &mappings::fabric_game_mode::DESTROY_DELAY,
                &mappings::fabric_game_mode::DESTROY_PROGRESS,
            )
        } else {
            (
                &mappings::game_mode::DESTROY_DELAY,
                &mappings::game_mode::DESTROY_PROGRESS,
            )
        };

        if env
            .get_field_id(&gm_local, destroy_delay.name, destroy_delay.signature)
            .is_err()
            || env
                .get_field_id(&gm_local, destroy_progress.name, destroy_progress.signature)
                .is_err()
            || clear_exception(&mut env, "GameMode fields")
        {
            esp_log("[FastBreak] GameMode destroy field lookup failed\n");
            return false;
        }

        true
    }) else {
        return false;
    };

    if success {
        READY.store(true, Ordering::Relaxed);
    }
    success
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub struct FastBreak {
    pub break_damage: Mutex<f32>,
    enabled: AtomicBool,
    speed: Mutex<f32>,
    tool_only: AtomicBool,
    creative_only: AtomicBool,
}

impl FastBreak {
    pub fn new() -> Self {
        Self {
            break_damage: Mutex::new(0.0),
            enabled: AtomicBool::new(false),
            speed: Mutex::new(5.0),
            tool_only: AtomicBool::new(false),
            creative_only: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn get_speed(&self) -> f32 {
        *self.speed.lock()
    }
    pub fn set_speed(&self, speed: f32) {
        *self.speed.lock() = speed.clamp(1.0, 10.0);
    }
    pub fn tool_only(&self) -> bool {
        self.tool_only.load(Ordering::Relaxed)
    }
    pub fn set_tool_only(&self, v: bool) {
        self.tool_only.store(v, Ordering::Relaxed);
    }
    pub fn creative_only(&self) -> bool {
        self.creative_only.load(Ordering::Relaxed)
    }
    pub fn set_creative_only(&self, v: bool) {
        self.creative_only.store(v, Ordering::Relaxed);
    }

    pub fn break_fast(&self) {
        if !is_ready() || !self.is_enabled() {
            return;
        }

        let Some(mut env) = JniEnvironment::get_current_or_attach("FastBreak") else {
            return;
        };

        let Some(inst) = Minecraft::with(|m| {
            let o = m.instance()?;
            Some(unsafe { JObject::from_raw(o.as_raw()) })
        })
        .flatten() else {
            return;
        };

        let is_fabric = class_cache::is_fabric();

        // ── Minecraft.gameMode field (fabric vs vanilla) ──────────────────────
        let game_mode_field = if is_fabric {
            &mappings::fabric_minecraft::GAME_MODE
        } else {
            &mappings::minecraft::GAME_MODE
        };

        let gm_obj = match env.get_field(&inst, game_mode_field.name, game_mode_field.signature) {
            Ok(v) => match v.l() {
                Ok(o) => o,
                Err(_) => {
                    let _ = clear_exception(&mut env, "Minecraft.gameMode read");
                    return;
                }
            },
            Err(_) => {
                let _ = clear_exception(&mut env, "Minecraft.gameMode read");
                return;
            }
        };

        if gm_obj.as_raw().is_null() {
            return;
        }

        // ── GameMode destroy fields (fabric vs vanilla) ───────────────────────
        let (destroy_delay_m, destroy_progress_m) = if is_fabric {
            (
                &mappings::fabric_game_mode::DESTROY_DELAY,
                &mappings::fabric_game_mode::DESTROY_PROGRESS,
            )
        } else {
            (
                &mappings::game_mode::DESTROY_DELAY,
                &mappings::game_mode::DESTROY_PROGRESS,
            )
        };

        let speed = self.get_speed();

        // ── Read current destroy progress ─────────────────────────────────────
        let current_progress = env
            .get_field(
                &gm_obj,
                destroy_progress_m.name,
                destroy_progress_m.signature,
            )
            .ok()
            .and_then(|v| v.f().ok())
            .unwrap_or_else(|| {
                let _ = clear_exception(&mut env, "GameMode.destroyProgress read");
                0.0
            });

        // ── Calculate new progress ────────────────────────────────────────────
        let new_progress = if speed >= 10.0 {
            1.0
        } else {
            let boost = (speed - 1.0) / 9.0;
            (current_progress + 0.05 * boost).min(1.0)
        };

        let _ = env.set_field(
            &gm_obj,
            destroy_progress_m.name,
            destroy_progress_m.signature,
            JValue::Float(new_progress),
        );
        let _ = clear_exception(&mut env, "GameMode.destroyProgress write");

        // ── Destroy delay (only at speed extremes) ────────────────────────────
        if speed >= 10.0 {
            let _ = env.set_field(
                &gm_obj,
                destroy_delay_m.name,
                destroy_delay_m.signature,
                JValue::Int(-5),
            );
            let _ = clear_exception(&mut env, "GameMode.destroyDelay write");
        } else if speed <= 1.0 {
            let _ = env.set_field(
                &gm_obj,
                destroy_delay_m.name,
                destroy_delay_m.signature,
                JValue::Int(5),
            );
            let _ = clear_exception(&mut env, "GameMode.destroyDelay write");
        }
    }

    pub fn insta_break(&self) {}

    pub fn on_update(&self) {
        self.break_fast();
    }
}
