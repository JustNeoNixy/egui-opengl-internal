use std::sync::atomic::AtomicBool;

use crate::core::client::Minecraft;
use crate::core::local_player::LocalPlayer;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::JValue;
use jni::objects::{GlobalRef, JMethodID, JObject};
use jni::JNIEnv;
use once_cell::sync::OnceCell;

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[Sprint] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

// Cached class + method so we only resolve once at init time
static LOCAL_PLAYER_CLASS: OnceCell<GlobalRef> = OnceCell::new();
static SET_SPRINTING_MID: OnceCell<JMethodID> = OnceCell::new();

pub struct Sprint {
    enabled: AtomicBool,
}

unsafe impl Sync for Sprint {}
unsafe impl Send for Sprint {}

impl Sprint {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn on_update(&self) {
        if !self.is_enabled() {
            return;
        }

        let w_pressed = unsafe {
            windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
                windows::Win32::UI::Input::KeyboardAndMouse::VK_W.0 as i32,
            ) & 0x8000u16 as i16
        } != 0;

        if !w_pressed {
            return;
        }

        let Some(method_id) = SET_SPRINTING_MID.get().copied() else {
            esp_log("[Sprint] setSprinting method not cached — was init() called?\n");
            return;
        };

        let Some(mut env) = JniEnvironment::get_current_or_attach("Sprint") else {
            esp_log("[Sprint] Failed to attach JNI env\n");
            return;
        };

        let Some(player) = LocalPlayer::get_object() else {
            esp_log("[Sprint] Failed to get LocalPlayer object\n");
            return;
        };

        // Use the pre-resolved method ID directly — same pattern as minecraft.rs
        // uses pre-fetched field IDs for instance/options etc.
        let result = unsafe {
            env.call_method_unchecked(
                &player,
                method_id,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                &[jni::sys::jvalue { z: 1 }],
            )
        };

        if let Err(e) = result {
            esp_log(&format!("[Sprint] setSprinting call failed: {:?}\n", e));
            clear_exception(&mut env, "LocalPlayer.setSprinting");
        }
    }

    /// Called once at startup (after Minecraft::init) — mirrors the class/field
    /// resolution pattern in minecraft.rs: JVMTI cache first, FindClass fallback.
    pub fn init() {
        esp_log("[Sprint] Initializing...\n");

        let mut env_wrapper = match JniEnvironment::attach("Sprint-init") {
            Some(e) => e,
            None => {
                esp_log("[Sprint] Failed to attach JNI env during init\n");
                return;
            }
        };

        // ── 1. Resolve LocalPlayer class (fabric vs vanilla) ──────────────
        //    Mirrors minecraft.rs: JVMTI cache → FindClass fallback.
        let class_name = if class_cache::is_fabric() {
            mappings::fabric_classes::LOCAL_PLAYER
        } else {
            mappings::classes::LOCAL_PLAYER
        };

        let class_local = if let Some(cached) = class_cache::find_class(class_name) {
            esp_log("[Sprint] Found LocalPlayer class via JVMTI cache\n");
            cached
        } else {
            match env_wrapper.env().find_class(class_name) {
                Ok(c) => {
                    esp_log("[Sprint] Found LocalPlayer class via FindClass\n");
                    c
                }
                Err(e) => {
                    esp_log(&format!(
                        "[Sprint] Failed to find LocalPlayer class '{}': {:?}\n",
                        class_name, e
                    ));
                    clear_exception(env_wrapper.env(), "FindClass(LocalPlayer)");
                    return;
                }
            }
        };

        let class_global = match env_wrapper.env().new_global_ref(class_local) {
            Ok(g) => g,
            Err(e) => {
                esp_log(&format!(
                    "[Sprint] Failed to create global ref for LocalPlayer class: {:?}\n",
                    e
                ));
                return;
            }
        };

        // ── 2. Resolve setSprinting method (fabric vs vanilla) ─────────────
        let set_sprinting = if class_cache::is_fabric() {
            &mappings::fabric_local_player::SET_SPRINTING
        } else {
            &mappings::local_player::SET_SPRINTING
        };

        let method_id = match env_wrapper.env().get_method_id(
            &class_global,
            set_sprinting.name,
            set_sprinting.signature,
        ) {
            Ok(mid) => mid,
            Err(e) => {
                esp_log(&format!(
                    "[Sprint] Failed to resolve setSprinting '{}{}': {:?}\n",
                    set_sprinting.name, set_sprinting.signature, e
                ));
                clear_exception(env_wrapper.env(), "GetMethodID(setSprinting)");
                return;
            }
        };

        // ── 3. Store in statics (set once, reused every frame) ─────────────
        let _ = LOCAL_PLAYER_CLASS.set(class_global);
        let _ = SET_SPRINTING_MID.set(method_id);

        // ── 4. Also initialise LocalPlayer's own cached class ref ──────────
        Minecraft::with(|mc| {
            if LocalPlayer::init(mc) {
                esp_log("[Sprint] LocalPlayer::init succeeded\n");
            } else {
                esp_log("[Sprint] LocalPlayer::init failed\n");
            }
        });

        esp_log("[Sprint] Initialization complete\n");
    }
}
