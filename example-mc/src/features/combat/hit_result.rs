//! Port of C++ `HitResult` / `BlockHitResult` / `EntityHitResult` (crosshair + trigger bot).

use crate::core::client::Minecraft;
use crate::core::LocalPlayer;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::JNIEnv;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn clear_exception(env: &mut JNIEnv, context: &str) -> bool {
    if env.exception_check().unwrap_or(false) {
        esp_log(&format!("[HitResult] JNI call failed at {}\n", context));
        let _ = env.exception_clear();
        return true;
    }
    false
}

// ── Helper: resolve a class via JVMTI cache → FindClass fallback ─────────────
fn find_class_any<'a>(env: &mut JNIEnv<'a>, name: &str) -> Option<jni::objects::JClass<'a>> {
    if let Some(cached) = class_cache::find_class(name) {
        return Some(cached);
    }
    match env.find_class(name) {
        Ok(c) => Some(c),
        Err(_) => {
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_clear();
            }
            None
        }
    }
}

#[derive(Clone)]
pub struct HitResult {
    inner: Arc<HitResultInner>,
}

struct HitResultInner {
    #[allow(dead_code)]
    hit_result_class: GlobalRef,
}

impl HitResult {
    pub fn new(env: &mut JNIEnv) -> Option<Self> {
        // ── HitResult class (fabric vs vanilla) ───────────────────────────────
        let class_name = if class_cache::is_fabric() {
            mappings::fabric_classes::HIT_RESULT
        } else {
            mappings::classes::HIT_RESULT
        };

        let local = find_class_any(env, class_name)?;
        if clear_exception(env, "FindClass(HitResult)") {
            return None;
        }
        let hit_result_class = env.new_global_ref(&local).ok()?;
        Some(Self {
            inner: Arc::new(HitResultInner { hit_result_class }),
        })
    }

    pub fn get_hit_result_object<'a>(&self, env: &mut JNIEnv<'a>) -> Option<JObject<'a>> {
        let inst = Minecraft::with(|m| {
            let o = m.instance()?;
            Some(unsafe { JObject::from_raw(o.as_raw()) })
        })
        .flatten()?;

        // ── Minecraft.hitResult field (fabric vs vanilla) ─────────────────────
        let hit_result_field = if class_cache::is_fabric() {
            &mappings::fabric_minecraft::HIT_RESULT
        } else {
            &mappings::minecraft::HIT_RESULT
        };

        let val = env
            .get_field(&inst, hit_result_field.name, hit_result_field.signature)
            .ok();
        if clear_exception(env, "Minecraft.hitResult read") {
            return None;
        }
        val?.l().ok()
    }
}

pub struct BlockHitResult {
    hit: HitResult,
    block_class: GlobalRef,
}

impl BlockHitResult {
    pub fn new(env: &mut JNIEnv, hit: HitResult) -> Option<Self> {
        // ── BlockHitResult class (fabric vs vanilla) ──────────────────────────
        let class_name = if class_cache::is_fabric() {
            mappings::fabric_classes::BLOCK_HIT_RESULT
        } else {
            mappings::classes::BLOCK_HIT_RESULT
        };

        let local = find_class_any(env, class_name)?;
        if clear_exception(env, "FindClass(BlockHitResult)") {
            return None;
        }
        let block_class = env.new_global_ref(&local).ok()?;
        Some(Self { hit, block_class })
    }

    fn block_as_class(&self) -> &JClass {
        unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(self.block_class.as_obj()) }
    }

    pub fn is_block(&self, env: &mut JNIEnv) {
        let Some(hit_obj) = self.hit.get_hit_result_object(env) else {
            return;
        };
        let is_block = env
            .is_instance_of(&hit_obj, self.block_as_class())
            .unwrap_or(false);
        if clear_exception(env, "BlockHitResult.is_instance_of") {
            return;
        }
        if is_block {
            esp_log("[BlockHitResult] Block target detected.\n");
        } else {
            esp_log("[BlockHitResult] Target is not a block.\n");
        }
    }
}

pub struct EntityHitResult {
    hit: HitResult,
    entity_hit_class: GlobalRef,
    #[allow(dead_code)]
    entity_class: GlobalRef,
}

impl EntityHitResult {
    pub fn new(env: &mut JNIEnv, hit: HitResult) -> Option<Self> {
        // ── EntityHitResult class (fabric vs vanilla) ─────────────────────────
        let ehr_name = if class_cache::is_fabric() {
            mappings::fabric_classes::ENTITY_HIT_RESULT
        } else {
            mappings::classes::ENTITY_HIT_RESULT
        };

        let ehr_local = find_class_any(env, ehr_name)?;
        if clear_exception(env, "FindClass(EntityHitResult)") {
            return None;
        }
        let entity_hit_class = env.new_global_ref(&ehr_local).ok()?;

        // ── Entity class (fabric vs vanilla) ──────────────────────────────────
        let ent_name = if class_cache::is_fabric() {
            mappings::fabric_classes::ENTITY
        } else {
            mappings::classes::ENTITY
        };

        let ent_local = find_class_any(env, ent_name)?;
        if clear_exception(env, "FindClass(Entity)") {
            return None;
        }
        let entity_class = env.new_global_ref(&ent_local).ok()?;

        Some(Self {
            hit,
            entity_hit_class,
            entity_class,
        })
    }

    fn entity_hit_as_class(&self) -> &JClass {
        unsafe { std::mem::transmute::<&JObject<'_>, &JClass<'_>>(self.entity_hit_class.as_obj()) }
    }

    fn is_attack_ready(&self, env: &mut JNIEnv<'_>) -> bool {
        let Some(lp) = LocalPlayer::get_object() else {
            return false;
        };

        // ── getAttackStrengthScale (fabric vs vanilla) ────────────────────────
        let attack_scale = if class_cache::is_fabric() {
            &mappings::fabric_local_player::ATTACK_STRENGTH_SCALE
        } else {
            &mappings::local_player::ATTACK_STRENGTH_SCALE
        };

        let strength = env
            .call_method(
                &lp,
                attack_scale.name,
                attack_scale.signature,
                &[JValue::Float(0.0)],
            )
            .ok()
            .and_then(|v| v.f().ok());

        if clear_exception(env, "LocalPlayer.getAttackStrengthScale") {
            return false;
        }
        strength.is_some_and(|s| s >= 1.0)
    }

    pub fn tick_entity_attack(&self, env: &mut JNIEnv) {
        let Some(mc_inst) = Minecraft::with(|m| {
            let o = m.instance()?;
            Some(unsafe { JObject::from_raw(o.as_raw()) })
        })
        .flatten() else {
            return;
        };

        let Some(hit_obj) = self.hit.get_hit_result_object(env) else {
            return;
        };

        if !env
            .is_instance_of(&hit_obj, self.entity_hit_as_class())
            .unwrap_or(false)
        {
            return;
        }
        if clear_exception(env, "EntityHitResult.is_instance_of") {
            return;
        }

        // ── EntityHitResult.getEntity (fabric vs vanilla) ─────────────────────
        let get_entity = if class_cache::is_fabric() {
            &mappings::fabric_entity_hit_result::GET_ENTITY
        } else {
            &mappings::entity_hit_result::GET_ENTITY
        };

        let entity = env
            .call_method(&hit_obj, get_entity.name, get_entity.signature, &[])
            .ok()
            .and_then(|v| v.l().ok());

        let Some(entity) = entity else {
            let _ = clear_exception(env, "EntityHitResult.getEntity");
            return;
        };
        if entity.as_raw().is_null() {
            return;
        }

        // ── Entity.isAlive (fabric vs vanilla) ────────────────────────────────
        let is_alive = if class_cache::is_fabric() {
            &mappings::fabric_entity::IS_ALIVE
        } else {
            &mappings::entity::IS_ALIVE
        };

        let alive = env
            .call_method(&entity, is_alive.name, is_alive.signature, &[])
            .ok()
            .and_then(|v| v.z().ok())
            .unwrap_or(false);

        if clear_exception(env, "Entity.isAlive") {
            return;
        }

        // ── Minecraft.startAttack (fabric vs vanilla) ─────────────────────────
        let start_attack = if class_cache::is_fabric() {
            &mappings::fabric_minecraft::START_ATTACK
        } else {
            &mappings::minecraft::START_ATTACK
        };

        if alive && self.is_attack_ready(env) {
            let _ = env.call_method(&mc_inst, start_attack.name, start_attack.signature, &[]);
            let _ = clear_exception(env, "Minecraft.startAttack");
        }
    }
}

pub struct HitResultBundle {
    pub block: BlockHitResult,
    pub entity: EntityHitResult,
}

impl HitResultBundle {
    pub fn new(env: &mut JNIEnv) -> Option<Self> {
        let base = HitResult::new(env)?;
        let block = BlockHitResult::new(env, base.clone())?;
        let entity = EntityHitResult::new(env, base)?;
        Some(Self { block, entity })
    }

    pub fn get_instance() -> Option<&'static HitResultBundle> {
        BUNDLE.get()
    }
}

static BUNDLE: OnceCell<HitResultBundle> = OnceCell::new();
static TRIGGER_BOT_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn try_init_bundle() -> bool {
    if BUNDLE.get().is_some() {
        return true;
    }
    let Some(mut env) = JniEnvironment::get_current_or_attach("HitResult init") else {
        return false;
    };
    let Some(bundle) = HitResultBundle::new(&mut env) else {
        esp_log("[HitResult] HitResultBundle::new failed (check mappings)\n");
        return false;
    };
    BUNDLE.set(bundle).is_ok()
}

pub fn bundle() -> Option<&'static HitResultBundle> {
    BUNDLE.get()
}

pub fn set_trigger_bot_enabled(on: bool) {
    TRIGGER_BOT_ENABLED.store(on, Ordering::Relaxed);
}

pub fn trigger_bot_enabled() -> bool {
    TRIGGER_BOT_ENABLED.load(Ordering::Relaxed)
}

pub fn tick_trigger_bot() {
    if !trigger_bot_enabled() {
        return;
    }
    let Some(b) = HitResultBundle::get_instance() else {
        return;
    };
    let Some(mut env) = JniEnvironment::get_current_or_attach("TriggerBot") else {
        return;
    };
    b.entity.tick_entity_attack(&mut env);
}
