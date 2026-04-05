//! Port of C++ `Velocity` — anti-knockback via `setDeltaMovement(0,0,0)` when `hurtTime >= 8`.

use crate::core::local_player::LocalPlayer;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::JValue;
use jni::JNIEnv;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static READY: AtomicBool = AtomicBool::new(false);

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[Velocity] JNI error at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

/// Resolve `LocalPlayer.setDeltaMovement` and `hurtTime` field once.
pub fn try_init() -> bool {
    let Some(mut env) = JniEnvironment::get_current_or_attach("Velocity init") else {
        return false;
    };
    let Some(lp_class) = LocalPlayer::get_class() else {
        esp_log("[Velocity] LocalPlayer class not ready\n");
        return false;
    };

    // ── Pick mappings (fabric vs vanilla) ─────────────────────────────────────
    let (set_delta_movement, hurt_time) = if class_cache::is_fabric() {
        (
            &mappings::fabric_local_player::SET_DELTA_MOVEMENT,
            &mappings::fabric_local_player::HURT_TIME,
        )
    } else {
        (
            &mappings::local_player::SET_DELTA_MOVEMENT,
            &mappings::local_player::HURT_TIME,
        )
    };

    if env
        .get_method_id(
            lp_class,
            set_delta_movement.name,
            set_delta_movement.signature,
        )
        .is_err()
        || clear_exception(&mut env, "LocalPlayer.setDeltaMovement")
    {
        esp_log("[Velocity] setDeltaMovement method ID not found\n");
        return false;
    }

    if env
        .get_field_id(lp_class, hurt_time.name, hurt_time.signature)
        .is_err()
        || clear_exception(&mut env, "LocalPlayer.hurtTime")
    {
        esp_log("[Velocity] hurtTime field ID not found\n");
        return false;
    }

    READY.store(true, Ordering::Relaxed);
    true
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub struct Velocity {
    enabled: AtomicBool,
    strength: Mutex<f32>,
    vertical_only: AtomicBool,
    only_when_hurt: AtomicBool,
}

impl Velocity {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            strength: Mutex::new(1.0),
            vertical_only: AtomicBool::new(false),
            only_when_hurt: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn get_strength(&self) -> f32 {
        *self.strength.lock()
    }
    pub fn set_strength(&self, v: f32) {
        *self.strength.lock() = v.clamp(0.0, 1.0);
    }
    pub fn vertical_only(&self) -> bool {
        self.vertical_only.load(Ordering::Relaxed)
    }
    pub fn set_vertical_only(&self, v: bool) {
        self.vertical_only.store(v, Ordering::Relaxed);
    }
    pub fn only_when_hurt(&self) -> bool {
        self.only_when_hurt.load(Ordering::Relaxed)
    }
    pub fn set_only_when_hurt(&self, v: bool) {
        self.only_when_hurt.store(v, Ordering::Relaxed);
    }

    pub fn anti_knockback(&self) {
        if !is_ready() || !self.is_enabled() {
            return;
        }

        let Some(mut env) = JniEnvironment::get_current_or_attach("Velocity") else {
            return;
        };

        let Some(lp) = LocalPlayer::get_object() else {
            return;
        };

        // ── Pick mappings (fabric vs vanilla) ─────────────────────────────────
        let (set_delta_movement, hurt_time) = if class_cache::is_fabric() {
            (
                &mappings::fabric_local_player::SET_DELTA_MOVEMENT,
                &mappings::fabric_local_player::HURT_TIME,
            )
        } else {
            (
                &mappings::local_player::SET_DELTA_MOVEMENT,
                &mappings::local_player::HURT_TIME,
            )
        };

        // ── Read hurtTime ─────────────────────────────────────────────────────
        let hurt = match env.get_field(&lp, hurt_time.name, hurt_time.signature) {
            Ok(v) => match v.i() {
                Ok(h) => h,
                Err(_) => {
                    let _ = clear_exception(&mut env, "LocalPlayer.hurtTime read");
                    return;
                }
            },
            Err(_) => {
                let _ = clear_exception(&mut env, "LocalPlayer.hurtTime read");
                return;
            }
        };

        if self.only_when_hurt() && hurt < 8 {
            return;
        }

        if hurt >= 8 {
            let strength = self.get_strength();
            let multiplier = 1.0 - strength;

            // vertical_only: zero horizontal, preserve vertical scaled by multiplier.
            // full cancel (strength=1.0): multiplier=0.0, all axes zeroed.
            let (x_vel, y_vel, z_vel) = if self.vertical_only() {
                (multiplier as f64, 0.0, multiplier as f64)
            } else {
                (multiplier as f64, multiplier as f64, multiplier as f64)
            };

            let _ = env.call_method(
                &lp,
                set_delta_movement.name,
                set_delta_movement.signature,
                &[
                    JValue::Double(x_vel),
                    JValue::Double(y_vel),
                    JValue::Double(z_vel),
                ],
            );
            let _ = clear_exception(&mut env, "LocalPlayer.setDeltaMovement");
        }
    }

    pub fn on_update(&self) {
        self.anti_knockback();
    }
}
