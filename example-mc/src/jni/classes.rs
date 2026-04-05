//! Cached GlobalRef to Java Classes

use crate::jni::class_cache::is_fabric;
use crate::jni::mappings::{self, classes, fabric_classes};
use jni::objects::GlobalRef;
use once_cell::sync::OnceCell;

/// Cached Java class references
pub struct JavaClasses {
    pub minecraft: Option<GlobalRef>,
    pub entity: Option<GlobalRef>,
    pub living_entity: Option<GlobalRef>,
    pub local_player: Option<GlobalRef>,
    pub client_level: Option<GlobalRef>,
    pub vec3: Option<GlobalRef>,
    pub camera: Option<GlobalRef>,
    pub game_renderer: Option<GlobalRef>,
    pub hit_result: Option<GlobalRef>,
    pub block_hit_result: Option<GlobalRef>,
    pub entity_hit_result: Option<GlobalRef>,
    pub component: Option<GlobalRef>,
    pub java_list: Option<GlobalRef>,
}

impl Default for JavaClasses {
    fn default() -> Self {
        Self {
            minecraft: None,
            entity: None,
            living_entity: None,
            local_player: None,
            client_level: None,
            vec3: None,
            camera: None,
            game_renderer: None,
            hit_result: None,
            block_hit_result: None,
            entity_hit_result: None,
            component: None,
            java_list: None,
        }
    }
}

macro_rules! class_name {
    ($vanilla:expr, $fabric:expr) => {
        if is_fabric() {
            $fabric
        } else {
            $vanilla
        }
    };
}

impl JavaClasses {
    /// Get the global instance of cached classes
    pub fn instance() -> &'static mut Self {
        static mut INSTANCE: OnceCell<JavaClasses> = OnceCell::new();
        unsafe {
            INSTANCE.get_or_init(|| {
                let mut classes = JavaClasses::default();
                // We'll initialize lazily
                classes
            });
            INSTANCE.get_mut().unwrap()
        }
    }

    /// Initialize all class references
    pub fn initialize(&mut self) -> crate::error::Result<()> {
        use crate::jni::env::JniEnvironment;

        let mut env = JniEnvironment::attach("JavaClasses::init")
            .ok_or(crate::error::ExampleMcError::JvmAttachFailed)?;

        macro_rules! cache_class {
            ($field:ident, $vanilla:expr, $fabric:expr) => {
                let class_name = class_name!($vanilla, $fabric);
                self.$field = env
                    .env()
                    .find_class(class_name)
                    .ok()
                    .and_then(|cls| env.env().new_global_ref(cls).ok());

                if self.$field.is_none() {
                    eprintln!(
                        "[ESP] WARNING: Failed to cache class: {} (using {})",
                        class_name,
                        if is_fabric() { "Fabric" } else { "Vanilla" }
                    );
                }
            };
        }

        cache_class!(minecraft, classes::MINECRAFT, fabric_classes::MINECRAFT);
        cache_class!(entity, classes::ENTITY, fabric_classes::ENTITY);
        cache_class!(
            living_entity,
            classes::LIVING_ENTITY,
            fabric_classes::LIVING_ENTITY
        );
        cache_class!(
            local_player,
            classes::LOCAL_PLAYER,
            fabric_classes::LOCAL_PLAYER
        );
        cache_class!(
            client_level,
            classes::CLIENT_LEVEL,
            fabric_classes::CLIENT_LEVEL
        );
        cache_class!(vec3, classes::VEC3, fabric_classes::VEC3);
        cache_class!(camera, classes::CAMERA, fabric_classes::CAMERA);
        cache_class!(
            game_renderer,
            classes::GAME_RENDERER,
            fabric_classes::GAME_RENDERER
        );
        cache_class!(hit_result, classes::HIT_RESULT, fabric_classes::HIT_RESULT);
        cache_class!(
            block_hit_result,
            classes::BLOCK_HIT_RESULT,
            fabric_classes::BLOCK_HIT_RESULT
        );
        cache_class!(
            entity_hit_result,
            classes::ENTITY_HIT_RESULT,
            fabric_classes::ENTITY_HIT_RESULT
        );
        cache_class!(component, classes::COMPONENT, fabric_classes::COMPONENT);
        cache_class!(java_list, classes::JAVA_LIST, fabric_classes::JAVA_LIST);

        Ok(())
    }

    /// Get a cached class reference
    pub fn get_class(&self, name: &str) -> Option<&GlobalRef> {
        match name {
            "minecraft" => self.minecraft.as_ref(),
            "entity" => self.entity.as_ref(),
            "living_entity" => self.living_entity.as_ref(),
            "local_player" => self.local_player.as_ref(),
            "client_level" => self.client_level.as_ref(),
            "vec3" => self.vec3.as_ref(),
            "camera" => self.camera.as_ref(),
            "game_renderer" => self.game_renderer.as_ref(),
            "hit_result" => self.hit_result.as_ref(),
            "block_hit_result" => self.block_hit_result.as_ref(),
            "entity_hit_result" => self.entity_hit_result.as_ref(),
            "component" => self.component.as_ref(),
            "java_list" => self.java_list.as_ref(),
            _ => None,
        }
    }
}
