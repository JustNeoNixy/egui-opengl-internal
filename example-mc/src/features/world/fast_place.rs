use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::client::Minecraft;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::JValue;

pub struct FastPlace {
    enabled: AtomicBool,
}

unsafe impl Sync for FastPlace {}
unsafe impl Send for FastPlace {}

impl FastPlace {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn on_update(&self) {
        if !self.is_enabled() {
            return;
        }

        let Some(mut env) = JniEnvironment::get_current_or_attach("FastPlace") else {
            return;
        };

        // Pick the right field mapping (fabric vs vanilla)
        let field = if class_cache::is_fabric() {
            &mappings::fabric_minecraft::RIGHT_CLICK_DELAY
        } else {
            &mappings::minecraft::RIGHT_CLICK_DELAY
        };

        Minecraft::with(|mc| {
            let Some(instance) = mc.instance_ref() else {
                return;
            };

            let _ = env.set_field(instance, field.name, field.signature, JValue::Int(0));
        });
    }
}
