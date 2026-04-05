// minecraft.rs
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::JNIEnv;
use once_cell::sync::OnceCell;

use crate::esp_log;

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[Minecraft] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

pub struct Minecraft {
    class: Option<GlobalRef>,
    instance: Option<GlobalRef>,
}

static MINECRAFT_WRAPPER: OnceCell<Minecraft> = OnceCell::new();

impl Minecraft {
    /// Get the global Minecraft instance (like C++ getMcInstance)
    pub fn get_instance() -> Option<JObject<'static>> {
        MINECRAFT_WRAPPER.get()?.instance()
    }

    /// Get the global Minecraft class (like C++ getMinecraftClass)
    pub fn get_class() -> Option<&'static JClass<'static>> {
        MINECRAFT_WRAPPER.get()?.class_ref()
    }

    pub fn init() -> bool {
        esp_log("[DEBUG] Starting Minecraft initialization...\n");

        let mut env = match JniEnvironment::attach("JNI Cheat") {
            Some(e) => e,
            None => {
                esp_log("[ERROR] Failed to resolve JNIEnv for Minecraft wrapper.\n");
                return false;
            }
        };

        esp_log(&format!(
            "[DEBUG] Successfully attached to JNI environment\n"
        ));
        // Determine which class name to use based on detected loader.
        // On Fabric, Minecraft is loaded by Knot classloader under the
        // intermediary name — FindClass cannot reach it from a native thread.
        // The JVMTI cache (class_cache) enumerates all classes regardless of
        // classloader, so we try that first.
        let mc_class_name = mappings::minecraft_class();
        esp_log(&format!(
            "[DEBUG] Looking for Minecraft class: {} (Fabric={})\n",
            mc_class_name,
            class_cache::is_fabric()
        ));

        // Try JVMTI cache first (works for Fabric AND vanilla).
        // Fall back to FindClass for vanilla (works if not yet cached).
        let class_local = if let Some(cached) = class_cache::find_class(mc_class_name) {
            esp_log("[DEBUG] Found Minecraft class via JVMTI cache\n");
            cached
        } else {
            match env.env().find_class(mc_class_name) {
                Ok(c) => {
                    esp_log("[DEBUG] Found Minecraft class via FindClass\n");
                    c
                }
                Err(e) => {
                    esp_log(&format!(
                        "[ERROR] Failed to find Minecraft class '{}': {:?}\n",
                        mc_class_name, e
                    ));
                    clear_exception(env.env(), "FindClass(Minecraft)");
                    esp_log("[ERROR] Minecraft class not found. If using Fabric, ensure class_cache::init() ran before Minecraft::init().\n");
                    return false;
                }
            }
        };

        let class_global = match env.env().new_global_ref(class_local) {
            Ok(g) => {
                esp_log("[DEBUG] Successfully created global class reference\n");
                g
            }
            Err(e) => {
                esp_log(&format!(
                    "[ERROR] Failed to create global Minecraft class ref: {:?}\n",
                    e
                ));
                esp_log("[ERROR] Failed to create global Minecraft class ref.\n");
                return false;
            }
        };

        // Use loader-aware field mapping (Fabric uses "field_1700", vanilla uses "A").
        let instance_field = mappings::minecraft_instance_field();

        // Get static field ID first (like C++ GetStaticFieldID)
        let _field_id = match env.env().get_static_field_id(
            &class_global,
            instance_field.name,
            instance_field.signature,
        ) {
            Ok(id) => {
                esp_log(&format!(
                    "[DEBUG] Successfully found instance field ID: {}\n",
                    instance_field.name
                ));
                id
            }
            Err(e) => {
                esp_log(&format!(
                    "[ERROR] Failed to get instance field ID: {:?}\n",
                    e
                ));
                clear_exception(env.env(), "Minecraft.Instance");
                esp_log("[ERROR] Minecraft instance field not found.\n");
                return false;
            }
        };

        // Get static object field using direct field access (Rust JNI compatible)
        let instance_local = match env.env().get_static_field(
            &class_global,
            instance_field.name,
            instance_field.signature,
        ) {
            Ok(val) => {
                esp_log("[DEBUG] Successfully got static field value\n");
                match val.l() {
                    Ok(obj) => {
                        esp_log("[DEBUG] Successfully extracted object from field value\n");
                        obj
                    }
                    Err(e) => {
                        esp_log(&format!(
                            "[ERROR] Failed to extract object from field value: {:?}\n",
                            e
                        ));
                        esp_log("[ERROR] Minecraft instance object not found.\n");
                        return false;
                    }
                }
            }
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to get static field: {:?}\n", e));
                clear_exception(env.env(), "Minecraft.Instance read");
                esp_log("[ERROR] Minecraft instance object not found.\n");
                return false;
            }
        };

        let instance_global = match env.env().new_global_ref(instance_local) {
            Ok(g) => {
                esp_log("[DEBUG] Successfully created global instance reference\n");
                g
            }
            Err(e) => {
                esp_log(&format!(
                    "[ERROR] Failed to create global Minecraft instance ref: {:?}\n",
                    e
                ));
                esp_log("[ERROR] Failed to create global Minecraft instance ref.\n");
                return false;
            }
        };

        let _ = MINECRAFT_WRAPPER.set(Minecraft {
            class: Some(class_global),
            instance: Some(instance_global),
        });

        esp_log("[DEBUG] Minecraft initialization completed successfully!\n");
        true
    }

    pub fn with<R>(f: impl FnOnce(&Minecraft) -> R) -> Option<R> {
        MINECRAFT_WRAPPER.get().map(f)
    }

    /// Execute a function with the same JNI environment context that was used for Minecraft initialization
    pub fn with_jni_env<R>(f: impl FnOnce(&mut JNIEnv) -> R) -> Option<R> {
        let mut env = match JniEnvironment::get_current_or_attach("MinecraftContext") {
            Some(e) => e,
            None => return None,
        };
        Some(f(&mut env))
    }

    // Return reference to avoid lifetime issues
    pub fn instance_ref(&self) -> Option<&JObject<'_>> {
        self.instance.as_ref().map(|g| g.as_obj())
    }

    pub fn instance(&self) -> Option<JObject<'_>> {
        self.instance.as_ref().and_then(|g| {
            // Reconstruct JObject with current scope lifetime
            Some(unsafe { JObject::from_raw(g.as_obj().as_raw()) })
        })
    }

    pub fn class_ref(&self) -> Option<&JClass<'_>> {
        self.class
            .as_ref()
            .map(|g| unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(g.as_obj()) })
    }

    pub fn enable_fullbright(&self) -> bool {
        let mut env = match JniEnvironment::get_current_or_attach("FullBright") {
            Some(e) => e,
            None => return false,
        };

        let instance = match self.instance_ref() {
            Some(i) => i,
            None => return false,
        };

        let using_fabric = class_cache::is_fabric();

        let options_class = if using_fabric {
            &mappings::fabric_minecraft::OPTIONS
        } else {
            &mappings::minecraft::OPTIONS
        };

        // Get options object directly (Rust JNI compatible)
        let options_obj = match env.get_field(instance, options_class.name, options_class.signature)
        {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => {
                    clear_exception(&mut env, "Minecraft.options read");
                    esp_log("[ERROR] options object not found.\n");
                    return false;
                }
            },
            Err(_) => {
                clear_exception(&mut env, "Minecraft.options");
                esp_log("[ERROR] options object not found.\n");
                return false;
            }
        };

        // Get options class (like C++ GetObjectClass)
        let options_class = match env.get_object_class(&options_obj) {
            Ok(cls) => cls,
            Err(_) => return false,
        };

        let gamma_field = if using_fabric {
            &mappings::fabric_options::GAMMA
        } else {
            &mappings::options::GAMMA
        };

        // Get field ID for gamma field (like C++ GetFieldID)
        let _gamma_field_id =
            match env.get_field_id(&options_class, gamma_field.name, gamma_field.signature) {
                Ok(id) => id,
                Err(_) => {
                    clear_exception(&mut env, "Options.gamma");
                    esp_log("[ERROR] gamma field ID not found.\n");
                    return false;
                }
            };

        // Get gamma object directly (Rust JNI compatible)
        let gamma_obj = match env.get_field(&options_obj, gamma_field.name, gamma_field.signature) {
            Ok(val) => match val.l() {
                Ok(obj) => obj,
                Err(_) => {
                    clear_exception(&mut env, "Options.gamma read");
                    esp_log("[ERROR] gamma object not found.\n");
                    return false;
                }
            },
            Err(_) => {
                clear_exception(&mut env, "Options.gamma");
                esp_log("[ERROR] gamma object not found.\n");
                return false;
            }
        };

        // Get OptionInstance class (like C++ GetObjectClass)
        let option_instance_class = match env.get_object_class(&gamma_obj) {
            Ok(cls) => cls,
            Err(_) => return false,
        };

        let value_field = if using_fabric {
            &mappings::fabric_option_instance::VALUE
        } else {
            &mappings::option_instance::VALUE
        };

        // Get field ID for value field (like C++ GetFieldID)
        let _value_field_id = match env.get_field_id(
            &option_instance_class,
            value_field.name,
            value_field.signature,
        ) {
            Ok(id) => id,
            Err(_) => {
                clear_exception(&mut env, "OptionInstance.value");
                esp_log("[ERROR] value field ID not found.\n");
                return false;
            }
        };

        // Find Double class (like C++ FindClass)
        let double_class = match env.find_class(mappings::classes::JAVA_DOUBLE) {
            Ok(cls) => cls,
            Err(_) => {
                clear_exception(&mut env, "FindClass(Double)");
                return false;
            }
        };

        // Get Double constructor method ID (like C++ GetMethodID)
        let _double_constructor_id = match env.get_method_id(
            &double_class,
            mappings::java_double::CONSTRUCTOR.name,
            mappings::java_double::CONSTRUCTOR.signature,
        ) {
            Ok(id) => id,
            Err(_) => {
                clear_exception(&mut env, "Double.<init>");
                return false;
            }
        };

        // Create new Double object (like C++ NewObject)
        let new_gamma_value = match env.new_object(
            &double_class,
            mappings::java_double::CONSTRUCTOR.signature,
            &[JValue::Double(1000.0)],
        ) {
            Ok(obj) => obj,
            Err(_) => {
                clear_exception(&mut env, "Double new");
                return false;
            }
        };

        // Set the value field (Rust JNI compatible)
        if let Err(_) = env.set_field(
            &gamma_obj,
            value_field.name,
            value_field.signature,
            JValue::Object(&new_gamma_value),
        ) {
            clear_exception(&mut env, "OptionInstance.value write");
            return false;
        }

        esp_log("[INFO] FullBright enabled.\n");
        true
    }
}
