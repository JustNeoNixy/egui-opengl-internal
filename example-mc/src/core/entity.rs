// entity.rs
use crate::core::client::Minecraft;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JClass, JObject};
use jni::JNIEnv;
use once_cell::sync::OnceCell;

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[Entity] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

static ENTITY_CLASS: OnceCell<GlobalRef> = OnceCell::new();

pub struct Entity {
    pub entity_object: Option<GlobalRef>,
    pub mc: Option<&'static Minecraft>,
}

impl Entity {
    pub fn new(mc: &'static Minecraft) -> Option<Self> {
        esp_log("[DEBUG] Starting Entity initialization...\n");

        let mut env = JniEnvironment::attach("JNI Entity")?;

        // ── Resolve Entity class: JVMTI cache → FindClass fallback ──────────
        // Mirrors minecraft.rs pattern exactly.
        let class_name = if class_cache::is_fabric() {
            mappings::fabric_classes::ENTITY
        } else {
            mappings::classes::ENTITY
        };

        esp_log(&format!(
            "[DEBUG] Looking for Entity class: {}\n",
            class_name
        ));

        let local_class = if let Some(cached) = class_cache::find_class(class_name) {
            esp_log("[DEBUG] Found Entity class via JVMTI cache\n");
            cached
        } else {
            match env.env().find_class(class_name) {
                Ok(c) => {
                    esp_log("[DEBUG] Found Entity class via FindClass\n");
                    c
                }
                Err(e) => {
                    esp_log(&format!(
                        "[ERROR] Failed to find Entity class '{}': {:?}\n",
                        class_name, e
                    ));
                    clear_exception(env.env(), "FindClass(Entity)");
                    return None;
                }
            }
        };

        let class_global = match env.env().new_global_ref(local_class) {
            Ok(g) => {
                esp_log("[DEBUG] Successfully created Entity global class reference\n");
                g
            }
            Err(e) => {
                esp_log(&format!(
                    "[ERROR] Failed to create Entity global class ref: {:?}\n",
                    e
                ));
                return None;
            }
        };

        let _ = ENTITY_CLASS.set(class_global);

        esp_log("[DEBUG] Entity initialization completed successfully!\n");

        Some(Self {
            entity_object: None,
            mc: Some(mc),
        })
    }

    pub fn from_object(obj: JObject<'_>) -> Option<Self> {
        Some(Self {
            entity_object: JniEnvironment::get_current_or_attach("Entity")
                .and_then(|env| env.new_global_ref(obj).ok()),
            mc: None,
        })
    }

    pub fn get_class() -> Option<&'static JClass<'static>> {
        ENTITY_CLASS
            .get()
            .map(|g| unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(g.as_obj()) })
    }

    pub fn get_id(&self) -> Option<i32> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        // Pick fabric vs vanilla mapping
        let member = if class_cache::is_fabric() {
            &mappings::fabric_entity::GET_ID
        } else {
            &mappings::entity::GET_ID
        };

        let result = env.call_method(obj, member.name, member.signature, &[]);

        match result {
            Ok(val) => match val.i() {
                Ok(id) => Some(id),
                Err(e) => {
                    esp_log(&format!("[ERROR] Failed to get entity ID: {:?}\n", e));
                    clear_exception(&mut env, "Entity.getId");
                    None
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getId: {:?}\n", e));
                clear_exception(&mut env, "Entity.getId");
                None
            }
        }
    }

    pub fn is_alive(&self) -> Option<bool> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        let member = if class_cache::is_fabric() {
            &mappings::fabric_entity::IS_ALIVE
        } else {
            &mappings::entity::IS_ALIVE
        };

        let result = env.call_method(obj, member.name, member.signature, &[]);

        match result {
            Ok(val) => match val.z() {
                Ok(alive) => Some(alive),
                Err(e) => {
                    esp_log(&format!(
                        "[ERROR] Failed to get entity alive status: {:?}\n",
                        e
                    ));
                    clear_exception(&mut env, "Entity.isAlive");
                    None
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.isAlive: {:?}\n", e));
                clear_exception(&mut env, "Entity.isAlive");
                None
            }
        }
    }

    pub fn get_name(&self) -> Option<String> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        let member = if class_cache::is_fabric() {
            &mappings::fabric_entity::GET_NAME
        } else {
            &mappings::entity::GET_NAME
        };

        let result = env.call_method(obj, member.name, member.signature, &[]);

        match result {
            Ok(val) => match val.l() {
                Ok(name_obj) => {
                    let j_string: jni::objects::JString = name_obj.into();
                    let name_str = env.get_string(&j_string).ok()?;
                    Some(name_str.to_string_lossy().to_string())
                }
                Err(e) => {
                    esp_log(&format!(
                        "[ERROR] Failed to get entity name object: {:?}\n",
                        e
                    ));
                    clear_exception(&mut env, "Entity.getName");
                    None
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getName: {:?}\n", e));
                clear_exception(&mut env, "Entity.getName");
                None
            }
        }
    }

    pub fn get_position(&self) -> Option<(f64, f64, f64)> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        let (get_x, get_y, get_z) = if class_cache::is_fabric() {
            (
                &mappings::fabric_entity::GET_X,
                &mappings::fabric_entity::GET_Y,
                &mappings::fabric_entity::GET_Z,
            )
        } else {
            (
                &mappings::entity::GET_X,
                &mappings::entity::GET_Y,
                &mappings::entity::GET_Z,
            )
        };

        let x = match env.call_method(obj, get_x.name, get_x.signature, &[]) {
            Ok(val) => match val.d() {
                Ok(v) => v,
                Err(e) => {
                    esp_log(&format!("[ERROR] Failed to get entity X: {:?}\n", e));
                    clear_exception(&mut env, "Entity.getX");
                    return None;
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getX: {:?}\n", e));
                clear_exception(&mut env, "Entity.getX");
                return None;
            }
        };

        let y = match env.call_method(obj, get_y.name, get_y.signature, &[]) {
            Ok(val) => match val.d() {
                Ok(v) => v,
                Err(e) => {
                    esp_log(&format!("[ERROR] Failed to get entity Y: {:?}\n", e));
                    clear_exception(&mut env, "Entity.getY");
                    return None;
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getY: {:?}\n", e));
                clear_exception(&mut env, "Entity.getY");
                return None;
            }
        };

        let z = match env.call_method(obj, get_z.name, get_z.signature, &[]) {
            Ok(val) => match val.d() {
                Ok(v) => v,
                Err(e) => {
                    esp_log(&format!("[ERROR] Failed to get entity Z: {:?}\n", e));
                    clear_exception(&mut env, "Entity.getZ");
                    return None;
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getZ: {:?}\n", e));
                clear_exception(&mut env, "Entity.getZ");
                return None;
            }
        };

        Some((x, y, z))
    }

    pub fn get_eye_y(&self) -> Option<f64> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        let member = if class_cache::is_fabric() {
            &mappings::fabric_entity::GET_EYE_Y
        } else {
            &mappings::entity::GET_EYE_Y
        };

        let result = env.call_method(obj, member.name, member.signature, &[]);

        match result {
            Ok(val) => match val.d() {
                Ok(eye_y) => Some(eye_y),
                Err(e) => {
                    esp_log(&format!("[ERROR] Failed to get entity eye Y: {:?}\n", e));
                    clear_exception(&mut env, "Entity.getEyeY");
                    None
                }
            },
            Err(e) => {
                esp_log(&format!("[ERROR] Failed to call Entity.getEyeY: {:?}\n", e));
                clear_exception(&mut env, "Entity.getEyeY");
                None
            }
        }
    }

    pub fn get_old_position(&self) -> Option<glam::DVec3> {
        let mut env = JniEnvironment::get_current_or_attach("Entity")?;
        let obj = self.entity_object.as_ref()?.as_obj();

        let (old_x_m, old_y_m, old_z_m) = if class_cache::is_fabric() {
            (
                &mappings::fabric_entity::OLD_X,
                &mappings::fabric_entity::OLD_Y,
                &mappings::fabric_entity::OLD_Z,
            )
        } else {
            (
                &mappings::entity::OLD_X,
                &mappings::entity::OLD_Y,
                &mappings::entity::OLD_Z,
            )
        };

        let old_x = env
            .get_field(obj, old_x_m.name, old_x_m.signature)
            .ok()
            .and_then(|v| v.d().ok())?;
        let old_y = env
            .get_field(obj, old_y_m.name, old_y_m.signature)
            .ok()
            .and_then(|v| v.d().ok())?;
        let old_z = env
            .get_field(obj, old_z_m.name, old_z_m.signature)
            .ok()
            .and_then(|v| v.d().ok())?;

        Some(glam::DVec3::new(old_x, old_y, old_z))
    }
}

impl Drop for Entity {
    fn drop(&mut self) {}
}

pub fn get_entity_class() -> Option<&'static JClass<'static>> {
    ENTITY_CLASS
        .get()
        .map(|g| unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(g.as_obj()) })
}
