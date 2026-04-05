use crate::core::local_player::LocalPlayer;
use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::JValue;

pub struct Fly {
    enabled: bool,
    fly_speed: f32,
}

impl Fly {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fly_speed: 0.05,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        self.apply();
        esp_log(&format!(
            "[Fly] Fly {} (speed: {:.2})\n",
            if self.enabled { "enabled" } else { "disabled" },
            self.fly_speed
        ));
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.apply();
        esp_log(&format!(
            "[Fly] Fly {} (speed: {:.2})\n",
            if self.enabled { "enabled" } else { "disabled" },
            self.fly_speed
        ));
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.fly_speed = speed.clamp(0.01, 2.0);
        if self.enabled {
            self.apply();
        }
        esp_log(&format!("[Fly] Speed set to {:.2}\n", self.fly_speed));
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_speed(&self) -> f32 {
        self.fly_speed
    }

    fn apply(&self) {
        let Some(player) = LocalPlayer::get_object() else {
            esp_log("[Fly] No LocalPlayer object available\n");
            return;
        };

        let Some(mut env) = JniEnvironment::get_current_or_attach("Fly") else {
            esp_log("[Fly] Failed to attach JNI env\n");
            return;
        };

        let is_fabric = class_cache::is_fabric();

        // ── Get abilities object (fabric vs vanilla) ──────────────────────────
        // On vanilla: player::GET_ABILITIES is a plain field ("cG", "Lddi;")
        // On fabric:  fabric_player::GET_ABILITIES is a method "()Lnet/minecraft/class_1656;"
        // We handle both cases here.
        let abilities_obj = if is_fabric {
            let m = &mappings::fabric_player::GET_ABILITIES;
            env.call_method(&player, m.name, m.signature, &[])
                .ok()
                .and_then(|v| v.l().ok())
        } else {
            let f = &mappings::player::GET_ABILITIES;
            env.get_field(&player, f.name, f.signature)
                .ok()
                .and_then(|v| v.l().ok())
        };

        let Some(abilities_obj) = abilities_obj else {
            esp_log("[Fly] Failed to get abilities object\n");
            return;
        };

        // ── Pick abilities field mappings ─────────────────────────────────────
        let (may_fly, flying, fly_speed) = if is_fabric {
            (
                &mappings::fabric_abilities::MAY_FLY,
                &mappings::fabric_abilities::FLYING,
                &mappings::fabric_abilities::FLY_SPEED,
            )
        } else {
            (
                &mappings::abilities::MAY_FLY,
                &mappings::abilities::FLYING,
                &mappings::abilities::FLY_SPEED,
            )
        };

        let enabled_bool = JValue::Bool(if self.enabled { 1 } else { 0 });

        if let Err(e) = env.set_field(
            &abilities_obj,
            may_fly.name,
            may_fly.signature,
            enabled_bool,
        ) {
            esp_log(&format!("[Fly] Failed to set mayFly: {:?}\n", e));
        }

        if let Err(e) = env.set_field(&abilities_obj, flying.name, flying.signature, enabled_bool) {
            esp_log(&format!("[Fly] Failed to set flying: {:?}\n", e));
        }

        if let Err(e) = env.set_field(
            &abilities_obj,
            fly_speed.name,
            fly_speed.signature,
            JValue::Float(self.fly_speed),
        ) {
            esp_log(&format!("[Fly] Failed to set flySpeed: {:?}\n", e));
        }

        esp_log(&format!(
            "[Fly] Applied: mayfly={}, flying={}, speed={:.2}\n",
            self.enabled, self.enabled, self.fly_speed
        ));
    }
}

pub static mut FLY_INSTANCE: Option<Fly> = None;

pub fn get_fly() -> &'static mut Fly {
    unsafe {
        if FLY_INSTANCE.is_none() {
            FLY_INSTANCE = Some(Fly::new());
        }
        FLY_INSTANCE.as_mut().unwrap()
    }
}
