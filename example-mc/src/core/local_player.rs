// local_player.rs
use crate::core::client::Minecraft;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JClass, JMethodID, JObject};
use once_cell::sync::OnceCell;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

fn clear_exception(env: &mut JniEnvironment, context: &str) -> bool {
    if env.env().exception_check().unwrap_or(false) {
        esp_log(&format!("[LocalPlayer] JNI call failed at {}\n", context));
        let _ = env.env().exception_clear();
        return true;
    }
    false
}

fn clear_exception_direct(env: &mut jni::JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[LocalPlayer] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

struct CachedLocalPlayer {
    object: Option<GlobalRef>,
    last_update: AtomicU64,
}

unsafe impl Send for CachedLocalPlayer {}
unsafe impl Sync for CachedLocalPlayer {}

static CACHED_LOCAL_PLAYER: Mutex<CachedLocalPlayer> = Mutex::new(CachedLocalPlayer {
    object: None,
    last_update: AtomicU64::new(0),
});

pub struct LocalPlayer {
    player_object: Option<GlobalRef>,
    set_sprinting_method: Option<JMethodID>,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct FieldIdPtr(*mut jni::sys::jfieldID);
unsafe impl Send for FieldIdPtr {}
unsafe impl Sync for FieldIdPtr {}

static LOCAL_PLAYER_CLASS: OnceCell<GlobalRef> = OnceCell::new();
static LOCAL_PLAYER_FIELD: OnceCell<FieldIdPtr> = OnceCell::new();

impl LocalPlayer {
    pub fn init(mc: &Minecraft) -> bool {
        let Some(mut env) = JniEnvironment::attach("JNI LocalPlayer") else {
            return false;
        };

        // ── Resolve LocalPlayer class: JVMTI cache → FindClass fallback ─────
        let class_name = if class_cache::is_fabric() {
            mappings::fabric_classes::LOCAL_PLAYER
        } else {
            mappings::classes::LOCAL_PLAYER
        };

        let local_class = if let Some(cached) = class_cache::find_class(class_name) {
            esp_log("[LocalPlayer] Found class via JVMTI cache\n");
            cached
        } else {
            match env.env().find_class(class_name) {
                Ok(c) => {
                    esp_log("[LocalPlayer] Found class via FindClass\n");
                    c
                }
                Err(e) => {
                    esp_log(&format!(
                        "[LocalPlayer] Failed to find class '{}': {:?}\n",
                        class_name, e
                    ));
                    clear_exception_direct(env.env(), "FindClass(LocalPlayer)");
                    return false;
                }
            }
        };

        let local_player_class_global = match env.env().new_global_ref(local_class) {
            Ok(g) => g,
            Err(e) => {
                esp_log(&format!(
                    "[LocalPlayer] Failed to create global class ref: {:?}\n",
                    e
                ));
                return false;
            }
        };

        let _ = LOCAL_PLAYER_CLASS.set(local_player_class_global);
        true
    }

    pub fn new(mc: &'static Minecraft) -> Option<Self> {
        let mut env = JniEnvironment::attach("JNI LocalPlayer")?;

        // ── Pick setSprinting mapping (fabric vs vanilla) ────────────────────
        let set_sprinting_member = if class_cache::is_fabric() {
            &mappings::fabric_local_player::SET_SPRINTING
        } else {
            &mappings::local_player::SET_SPRINTING
        };

        let set_sprinting = LOCAL_PLAYER_CLASS.get().and_then(|cls| {
            env.env()
                .get_method_id(
                    cls,
                    set_sprinting_member.name,
                    set_sprinting_member.signature,
                )
                .ok()
        });

        Some(Self {
            player_object: None,
            set_sprinting_method: set_sprinting,
        })
    }

    pub fn get_object() -> Option<JObject<'static>> {
        let current_time = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() };

        match CACHED_LOCAL_PLAYER.lock() {
            Ok(cached) => {
                let last_update = cached.last_update.load(Ordering::Relaxed);
                if current_time.saturating_sub(last_update) < 50 {
                    if let Some(ref obj) = cached.object {
                        return Some(unsafe { JObject::from_raw(obj.as_raw()) });
                    }
                }
            }
            Err(_) => {
                esp_log("[LocalPlayer] Failed to acquire cache lock\n");
            }
        }

        // ── Pick LOCAL_PLAYER field mapping (fabric vs vanilla) ──────────────
        let lp_field = if class_cache::is_fabric() {
            &mappings::fabric_minecraft::LOCAL_PLAYER
        } else {
            &mappings::minecraft::LOCAL_PLAYER
        };

        let fresh_object = crate::core::client::Minecraft::with(|mc| {
            let mut env = JniEnvironment::get_current_or_attach("LocalPlayer")?;
            let mc_inst = mc.instance()?;

            let player = env
                .get_field(mc_inst, lp_field.name, lp_field.signature)
                .ok()
                .and_then(|v| v.l().ok())?;

            Some(player)
        })?;

        if let Ok(mut cached) = CACHED_LOCAL_PLAYER.lock() {
            if let Some(env) = JniEnvironment::get_current_or_attach("LocalPlayer") {
                if let Some(ref fresh_obj) = fresh_object {
                    match env.new_global_ref(fresh_obj) {
                        Ok(global_ref) => {
                            cached.object = Some(global_ref);
                            cached.last_update.store(current_time, Ordering::Relaxed);
                        }
                        Err(_) => {
                            cached.object = None;
                            cached.last_update.store(0, Ordering::Relaxed);
                        }
                    }
                }
            }
        }

        fresh_object
    }

    pub fn get_class() -> Option<&'static JClass<'static>> {
        LOCAL_PLAYER_CLASS
            .get()
            .map(|g| unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(g.as_obj()) })
    }

    pub fn sprint(&self) {
        let Some(mut env) = JniEnvironment::get_current_or_attach("LocalPlayer") else {
            return;
        };
        let Some(method_id) = self.set_sprinting_method else {
            return;
        };
        let Some(player) = self.player_object.as_ref().map(|g| g.as_obj()) else {
            return;
        };

        let w_pressed = unsafe {
            windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
                windows::Win32::UI::Input::KeyboardAndMouse::VK_W.0 as i32,
            ) & 0x8000u16 as i16
        } != 0;

        if w_pressed {
            let _ = env.call_method(
                player,
                // Use the already-resolved method ID's names — but since sprint()
                // uses the cached method_id path, we re-derive the member here
                // only for the name/sig strings (negligible cost, called rarely).
                if class_cache::is_fabric() {
                    mappings::fabric_local_player::SET_SPRINTING.name
                } else {
                    mappings::local_player::SET_SPRINTING.name
                },
                if class_cache::is_fabric() {
                    mappings::fabric_local_player::SET_SPRINTING.signature
                } else {
                    mappings::local_player::SET_SPRINTING.signature
                },
                &[jni::objects::JValue::Bool(1)],
            );
        }
    }

    pub fn update(&mut self) {
        if let Some(player) = Self::get_object() {
            if let Some(env) = JniEnvironment::get_current_or_attach("LocalPlayer") {
                self.player_object = env.new_global_ref(player).ok();
            }
        }
    }
}

impl Drop for LocalPlayer {
    fn drop(&mut self) {}
}
