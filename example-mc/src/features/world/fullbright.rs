use std::sync::atomic::AtomicBool;

use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JObject, JValue};
use jni::JNIEnv;
use once_cell::sync::OnceCell;

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[Fullbright] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

// Cached global ref to the gamma OptionInstance object, resolved once at init.
static GAMMA_OBJ: OnceCell<GlobalRef> = OnceCell::new();
// The field name/sig for OptionInstance.value, resolved at init.
static VALUE_FIELD_NAME: OnceCell<&'static str> = OnceCell::new();
static VALUE_FIELD_SIG: OnceCell<&'static str> = OnceCell::new();

pub struct Fullbright {
    enabled: AtomicBool,
}

unsafe impl Sync for Fullbright {}
unsafe impl Send for Fullbright {}

impl Fullbright {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
        self.apply(enabled);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Called every frame (from hk_wgl_swap_buffers) so that gamma stays
    /// overridden even if the game resets it (e.g. options save/load).
    pub fn on_update(&self) {
        if self.is_enabled() {
            self.set_gamma(1000.0);
        }
    }

    fn apply(&self, enabled: bool) {
        let gamma = if enabled { 1000.0 } else { 1.0 };
        self.set_gamma(gamma);
    }

    fn set_gamma(&self, value: f64) {
        let Some(gamma_obj) = GAMMA_OBJ.get() else {
            esp_log("[Fullbright] Gamma object not cached — was try_init() called?\n");
            return;
        };
        let Some(field_name) = VALUE_FIELD_NAME.get() else {
            return;
        };
        let Some(field_sig) = VALUE_FIELD_SIG.get() else {
            return;
        };

        // get_current_or_attach returns JNIEnv<'static> directly (not JniEnvironment)
        let Some(mut env) = JniEnvironment::get_current_or_attach("Fullbright") else {
            esp_log("[Fullbright] Failed to attach JNI env\n");
            return;
        };

        let gamma_jobj = unsafe { JObject::from_raw(gamma_obj.as_obj().as_raw()) };

        let double_class = match env.find_class(mappings::classes::JAVA_DOUBLE) {
            Ok(c) => c,
            Err(_) => {
                clear_exception(&mut env, "FindClass(Double)");
                return;
            }
        };

        let boxed = match env.new_object(
            &double_class,
            mappings::java_double::CONSTRUCTOR.signature,
            &[JValue::Double(value)],
        ) {
            Ok(o) => o,
            Err(_) => {
                clear_exception(&mut env, "Double.<init>");
                return;
            }
        };

        if let Err(_) = env.set_field(&gamma_jobj, field_name, field_sig, JValue::Object(&boxed)) {
            clear_exception(&mut env, "OptionInstance.value write");
        }
    }
}

/// Resolve and cache the gamma OptionInstance object and its value field.
/// Must be called after `Minecraft::init()`, mirroring the pattern in minecraft.rs.
pub fn try_init(mc: &crate::core::client::Minecraft) -> bool {
    esp_log("[Fullbright] Initializing...\n");

    // attach() returns JniEnvironment; we call .env() on it to get the inner &mut JNIEnv
    let mut env_wrapper = match JniEnvironment::attach("Fullbright-init") {
        Some(e) => e,
        None => {
            esp_log("[Fullbright] Failed to attach JNI env during init\n");
            return false;
        }
    };

    let using_fabric = class_cache::is_fabric();

    let instance = match mc.instance_ref() {
        Some(i) => i,
        None => {
            esp_log("[Fullbright] Minecraft instance not available\n");
            return false;
        }
    };

    // ── 1. Get options object ─────────────────────────────────────────────
    let options_field = if using_fabric {
        &mappings::fabric_minecraft::OPTIONS
    } else {
        &mappings::minecraft::OPTIONS
    };

    let options_obj = {
        let env = env_wrapper.env();
        match env.get_field(instance, options_field.name, options_field.signature) {
            Ok(v) => match v.l() {
                Ok(o) => o,
                Err(_) => {
                    clear_exception(env, "Minecraft.options read");
                    esp_log("[Fullbright] Failed to read options object\n");
                    return false;
                }
            },
            Err(_) => {
                clear_exception(env, "Minecraft.options");
                esp_log("[Fullbright] Failed to get options field\n");
                return false;
            }
        }
    };

    // ── 2. Get gamma OptionInstance object ────────────────────────────────
    let gamma_field = if using_fabric {
        &mappings::fabric_options::GAMMA
    } else {
        &mappings::options::GAMMA
    };

    let gamma_obj = {
        let env = env_wrapper.env();
        match env.get_field(&options_obj, gamma_field.name, gamma_field.signature) {
            Ok(v) => match v.l() {
                Ok(o) => o,
                Err(_) => {
                    clear_exception(env, "Options.gamma read");
                    esp_log("[Fullbright] Failed to read gamma object\n");
                    return false;
                }
            },
            Err(_) => {
                clear_exception(env, "Options.gamma");
                esp_log("[Fullbright] Failed to get gamma field\n");
                return false;
            }
        }
    };

    // ── 3. Cache the gamma object as a global ref ─────────────────────────
    let gamma_global = match env_wrapper.env().new_global_ref(gamma_obj) {
        Ok(g) => g,
        Err(e) => {
            esp_log(&format!(
                "[Fullbright] Failed to create global ref for gamma: {:?}\n",
                e
            ));
            return false;
        }
    };

    // ── 4. Cache value field name/sig ─────────────────────────────────────
    let value_field = if using_fabric {
        &mappings::fabric_option_instance::VALUE
    } else {
        &mappings::option_instance::VALUE
    };

    let _ = GAMMA_OBJ.set(gamma_global);
    let _ = VALUE_FIELD_NAME.set(value_field.name);
    let _ = VALUE_FIELD_SIG.set(value_field.signature);

    esp_log("[Fullbright] Initialization complete\n");
    true
}
