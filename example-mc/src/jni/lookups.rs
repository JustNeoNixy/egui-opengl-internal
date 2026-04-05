use crate::esp_log;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::mappings;
use jni::objects::{GlobalRef, JFieldID, JMethodID};

/// Clear any pending JNI exception and log it.
fn clear_exception(env: &mut JniEnvironment, context: &str) -> bool {
    if env.env().exception_check().unwrap_or(false) {
        esp_log(&format!("[JniLookups] JNI exception at {}\n", context));
        let _ = env.env().exception_clear();
        return true;
    }
    false
}

/// Convert a GlobalRef that holds a java.lang.Class into a JClass.
///
/// get_method_id / get_field_id require a JClass, but GlobalRef derefs to
/// JObject — passing &GlobalRef directly causes silent failures.
fn global_ref_as_class(global: &GlobalRef) -> jni::objects::JClass<'_> {
    // Safety: the caller guarantees this GlobalRef was obtained from a jclass.
    unsafe { jni::objects::JClass::from_raw(global.as_raw()) }
}

fn find_class_from_cache(env: &mut JniEnvironment, name: &str) -> Option<GlobalRef> {
    let local_class = match class_cache::find_class(name) {
        Some(cls) => {
            esp_log(&format!(
                "[JniLookups] Found class '{}' in JVMTI cache\n",
                name
            ));
            cls
        }
        None => {
            esp_log(&format!(
                "[JniLookups] Class '{}' not found in JVMTI cache\n",
                name
            ));
            return None;
        }
    };

    match env.env().new_global_ref(local_class) {
        Ok(global_ref) => Some(global_ref),
        Err(e) => {
            esp_log(&format!(
                "[JniLookups] Failed to create global ref for '{}': {:?}\n",
                name, e
            ));
            None
        }
    }
}

/// Look up a method ID, clearing any exception on failure and logging the result.
macro_rules! get_method {
    ($env:expr, $class:expr, $member:expr, $out:expr, $label:literal) => {{
        $out = $env
            .env()
            .get_method_id($class, $member.name, $member.signature)
            .ok();
        if $out.is_none() {
            clear_exception($env, $label);
            esp_log(&format!("[JniLookups] MISS method: {}\n", $label));
        } else {
            esp_log(&format!("[JniLookups] OK   method: {}\n", $label));
        }
    }};
}

/// Look up a field ID, clearing any exception on failure and logging the result.
macro_rules! get_field {
    ($env:expr, $class:expr, $member:expr, $out:expr, $label:literal) => {{
        $out = $env
            .env()
            .get_field_id($class, $member.name, $member.signature)
            .ok();
        if $out.is_none() {
            clear_exception($env, $label);
            esp_log(&format!("[JniLookups] MISS field:  {}\n", $label));
        } else {
            esp_log(&format!("[JniLookups] OK   field:  {}\n", $label));
        }
    }};
}

pub struct JniLookups {
    // Classes
    pub level_class: Option<GlobalRef>,
    pub list_class: Option<GlobalRef>,
    pub delta_tracker_timer_class: Option<GlobalRef>,
    pub game_renderer_class: Option<GlobalRef>,
    pub camera_class: Option<GlobalRef>,
    pub vec3_class: Option<GlobalRef>,
    pub living_entity_class: Option<GlobalRef>,
    pub player_class: Option<GlobalRef>,
    pub component_class: Option<GlobalRef>,

    // Fields
    pub level_field: Option<JFieldID>,
    pub delta_tracker_field: Option<JFieldID>,
    pub game_renderer_field: Option<JFieldID>,
    pub old_x_field: Option<JFieldID>,
    pub old_y_field: Option<JFieldID>,
    pub old_z_field: Option<JFieldID>,
    pub vec3_x_field: Option<JFieldID>,
    pub vec3_y_field: Option<JFieldID>,
    pub vec3_z_field: Option<JFieldID>,

    // Methods
    pub get_players_method: Option<JMethodID>,
    pub list_size_method: Option<JMethodID>,
    pub list_get_method: Option<JMethodID>,
    pub get_x_method: Option<JMethodID>,
    pub get_y_method: Option<JMethodID>,
    pub get_eye_y_method: Option<JMethodID>,
    pub get_z_method: Option<JMethodID>,
    pub get_game_time_delta_method: Option<JMethodID>,
    pub get_health_method: Option<JMethodID>,
    pub get_entity_id_method: Option<JMethodID>,
    pub get_name_method: Option<JMethodID>,
    pub get_string_method: Option<JMethodID>,
    pub get_main_camera_method: Option<JMethodID>,
    pub get_camera_position_method: Option<JMethodID>,
    pub get_camera_yaw_method: Option<JMethodID>,
    pub get_camera_pitch_method: Option<JMethodID>,
    pub get_render_fov_method: Option<JMethodID>,
}

impl JniLookups {
    pub fn init() -> Option<Self> {
        esp_log("[JniLookups] Starting initialization...\n");

        let mut env = JniEnvironment::attach("JNI ESP")?;
        let is_fabric = class_cache::is_fabric();
        esp_log(&format!("[JniLookups] Fabric={}\n", is_fabric));

        let mut lookups = Self {
            level_class: None,
            list_class: None,
            delta_tracker_timer_class: None,
            game_renderer_class: None,
            camera_class: None,
            vec3_class: None,
            living_entity_class: None,
            player_class: None,
            component_class: None,
            level_field: None,
            delta_tracker_field: None,
            game_renderer_field: None,
            old_x_field: None,
            old_y_field: None,
            old_z_field: None,
            vec3_x_field: None,
            vec3_y_field: None,
            vec3_z_field: None,
            get_players_method: None,
            list_size_method: None,
            list_get_method: None,
            get_x_method: None,
            get_y_method: None,
            get_eye_y_method: None,
            get_z_method: None,
            get_game_time_delta_method: None,
            get_health_method: None,
            get_entity_id_method: None,
            get_name_method: None,
            get_string_method: None,
            get_main_camera_method: None,
            get_camera_position_method: None,
            get_camera_yaw_method: None,
            get_camera_pitch_method: None,
            get_render_fov_method: None,
        };

        // ── Minecraft class global ref ────────────────────────────────────────
        let mc_class_global: Option<GlobalRef> =
            crate::core::client::Minecraft::with(|mc| -> Option<GlobalRef> {
                let cls = mc.class_ref()?;
                let mut env = JniEnvironment::get_current_or_attach("ESP lookups")?;
                env.new_global_ref(cls).ok()
            })
            .flatten();

        let mc_class_global = match mc_class_global {
            Some(g) => g,
            None => {
                esp_log("[JniLookups] Failed to get Minecraft class global ref\n");
                return None;
            }
        };

        // ── Fields on Minecraft class (fabric vs vanilla) ─────────────────────
        let (level_f, delta_tracker_f, game_renderer_f) = if is_fabric {
            (
                &mappings::fabric_minecraft::LEVEL,
                &mappings::fabric_minecraft::DELTA_TRACKER,
                &mappings::fabric_minecraft::GAME_RENDERER,
            )
        } else {
            (
                &mappings::minecraft::LEVEL,
                &mappings::minecraft::DELTA_TRACKER,
                &mappings::minecraft::GAME_RENDERER,
            )
        };

        let mc_jclass = global_ref_as_class(&mc_class_global);

        get_field!(
            &mut env,
            &mc_jclass,
            level_f,
            lookups.level_field,
            "Minecraft.level"
        );
        if lookups.level_field.is_none() {
            esp_log("[JniLookups] CRITICAL: Minecraft.level field not found — ESP disabled\n");
            return None;
        }

        get_field!(
            &mut env,
            &mc_jclass,
            delta_tracker_f,
            lookups.delta_tracker_field,
            "Minecraft.deltaTracker"
        );
        get_field!(
            &mut env,
            &mc_jclass,
            game_renderer_f,
            lookups.game_renderer_field,
            "Minecraft.gameRenderer"
        );

        // ── Class name selection (fabric vs vanilla) ──────────────────────────
        let (
            client_level_name,
            delta_tracker_name,
            game_renderer_name,
            camera_name,
            vec3_name,
            living_entity_name,
            player_name,
            component_name,
            entity_name,
        ) = if is_fabric {
            (
                mappings::fabric_classes::CLIENT_LEVEL,
                mappings::fabric_classes::DELTA_TRACKER,
                mappings::fabric_classes::GAME_RENDERER,
                mappings::fabric_classes::CAMERA,
                mappings::fabric_classes::VEC3,
                mappings::fabric_classes::LIVING_ENTITY,
                mappings::fabric_classes::PLAYER,
                mappings::fabric_classes::COMPONENT,
                mappings::fabric_classes::ENTITY,
            )
        } else {
            (
                mappings::classes::CLIENT_LEVEL,
                mappings::classes::DELTA_TRACKER,
                mappings::classes::GAME_RENDERER,
                mappings::classes::CAMERA,
                mappings::classes::VEC3,
                mappings::classes::LIVING_ENTITY,
                mappings::classes::PLAYER,
                mappings::classes::COMPONENT,
                mappings::classes::ENTITY,
            )
        };

        // ── Cache all classes ─────────────────────────────────────────────────
        lookups.list_class = find_class_from_cache(&mut env, mappings::classes::JAVA_LIST);
        lookups.level_class = find_class_from_cache(&mut env, client_level_name);
        lookups.delta_tracker_timer_class = find_class_from_cache(&mut env, delta_tracker_name);
        lookups.game_renderer_class = find_class_from_cache(&mut env, game_renderer_name);
        lookups.camera_class = find_class_from_cache(&mut env, camera_name);
        lookups.vec3_class = find_class_from_cache(&mut env, vec3_name);
        lookups.living_entity_class = find_class_from_cache(&mut env, living_entity_name);
        lookups.player_class = find_class_from_cache(&mut env, player_name);
        lookups.component_class = find_class_from_cache(&mut env, component_name);

        // Always clear after the batch of find_class calls before doing get_method_id
        clear_exception(&mut env, "post-class-cache");

        // ── Entity fields & methods ───────────────────────────────────────────
        let (get_x, get_y, get_eye_y, get_z, get_id, get_name_m, old_x, old_y, old_z) = if is_fabric
        {
            (
                &mappings::fabric_entity::GET_X,
                &mappings::fabric_entity::GET_Y,
                &mappings::fabric_entity::GET_EYE_Y,
                &mappings::fabric_entity::GET_Z,
                &mappings::fabric_entity::GET_ID,
                &mappings::fabric_entity::GET_NAME,
                &mappings::fabric_entity::OLD_X,
                &mappings::fabric_entity::OLD_Y,
                &mappings::fabric_entity::OLD_Z,
            )
        } else {
            (
                &mappings::entity::GET_X,
                &mappings::entity::GET_Y,
                &mappings::entity::GET_EYE_Y,
                &mappings::entity::GET_Z,
                &mappings::entity::GET_ID,
                &mappings::entity::GET_NAME,
                &mappings::entity::OLD_X,
                &mappings::entity::OLD_Y,
                &mappings::entity::OLD_Z,
            )
        };

        if let Some(entity_global) = find_class_from_cache(&mut env, entity_name) {
            let entity_cls = global_ref_as_class(&entity_global);
            get_method!(
                &mut env,
                &entity_cls,
                get_x,
                lookups.get_x_method,
                "Entity.getX"
            );
            get_method!(
                &mut env,
                &entity_cls,
                get_y,
                lookups.get_y_method,
                "Entity.getY"
            );
            get_method!(
                &mut env,
                &entity_cls,
                get_eye_y,
                lookups.get_eye_y_method,
                "Entity.getEyeY"
            );
            get_method!(
                &mut env,
                &entity_cls,
                get_z,
                lookups.get_z_method,
                "Entity.getZ"
            );
            get_method!(
                &mut env,
                &entity_cls,
                get_id,
                lookups.get_entity_id_method,
                "Entity.getId"
            );
            get_method!(
                &mut env,
                &entity_cls,
                get_name_m,
                lookups.get_name_method,
                "Entity.getName"
            );
            get_field!(
                &mut env,
                &entity_cls,
                old_x,
                lookups.old_x_field,
                "Entity.oldX"
            );
            get_field!(
                &mut env,
                &entity_cls,
                old_y,
                lookups.old_y_field,
                "Entity.oldY"
            );
            get_field!(
                &mut env,
                &entity_cls,
                old_z,
                lookups.old_z_field,
                "Entity.oldZ"
            );
        } else {
            esp_log("[JniLookups] CRITICAL: Entity class not found\n");
        }

        clear_exception(&mut env, "post-entity");

        // ── LivingEntity.getHealth ────────────────────────────────────────────
        let get_health = if is_fabric {
            &mappings::fabric_player::GET_HEALTH
        } else {
            &mappings::player::GET_HEALTH
        };

        if let Some(ref living_global) = lookups.living_entity_class {
            let living_cls = global_ref_as_class(living_global);
            get_method!(
                &mut env,
                &living_cls,
                get_health,
                lookups.get_health_method,
                "LivingEntity.getHealth"
            );
        }

        clear_exception(&mut env, "post-living-entity");

        // ── Component.getString ───────────────────────────────────────────────
        let get_string = if is_fabric {
            &mappings::fabric_component::GET_STRING
        } else {
            &mappings::component::GET_STRING
        };

        if let Some(ref component_global) = lookups.component_class {
            let component_cls = global_ref_as_class(component_global);
            get_method!(
                &mut env,
                &component_cls,
                get_string,
                lookups.get_string_method,
                "Component.getString"
            );
        }

        clear_exception(&mut env, "post-component");

        // ── ClientLevel.players ───────────────────────────────────────────────
        let players = if is_fabric {
            &mappings::fabric_client_level::PLAYERS
        } else {
            &mappings::client_level::PLAYERS
        };

        if let Some(ref level_global) = lookups.level_class {
            let level_cls = global_ref_as_class(level_global);
            get_method!(
                &mut env,
                &level_cls,
                players,
                lookups.get_players_method,
                "ClientLevel.players"
            );
        } else {
            esp_log("[JniLookups] CRITICAL: level_class is None — get_players will be missing\n");
        }

        clear_exception(&mut env, "post-level");

        // ── java.util.List ────────────────────────────────────────────────────
        if let Some(ref list_global) = lookups.list_class {
            let list_cls = global_ref_as_class(list_global);
            get_method!(
                &mut env,
                &list_cls,
                mappings::java_list::SIZE,
                lookups.list_size_method,
                "List.size"
            );
            get_method!(
                &mut env,
                &list_cls,
                mappings::java_list::GET,
                lookups.list_get_method,
                "List.get"
            );
        } else {
            esp_log("[JniLookups] CRITICAL: list_class is None — list methods will be missing\n");
        }

        clear_exception(&mut env, "post-list");

        // ── DeltaTracker.getGameTimeDeltaPartialTick ──────────────────────────
        let get_game_time_delta = if is_fabric {
            &mappings::fabric_delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
        } else {
            &mappings::delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
        };

        if let Some(ref dt_global) = lookups.delta_tracker_timer_class {
            let dt_cls = global_ref_as_class(dt_global);
            get_method!(
                &mut env,
                &dt_cls,
                get_game_time_delta,
                lookups.get_game_time_delta_method,
                "DeltaTracker.getGameTimeDelta"
            );
        }

        clear_exception(&mut env, "post-delta-tracker");

        // ── GameRenderer methods ──────────────────────────────────────────────
        let (get_main_camera, get_fov) = if is_fabric {
            (
                &mappings::fabric_game_renderer::GET_MAIN_CAMERA,
                &mappings::fabric_game_renderer::GET_FOV,
            )
        } else {
            (
                &mappings::game_renderer::GET_MAIN_CAMERA,
                &mappings::game_renderer::GET_FOV,
            )
        };

        if let Some(ref gr_global) = lookups.game_renderer_class {
            let gr_cls = global_ref_as_class(gr_global);
            get_method!(
                &mut env,
                &gr_cls,
                get_main_camera,
                lookups.get_main_camera_method,
                "GameRenderer.getMainCamera"
            );
            get_method!(
                &mut env,
                &gr_cls,
                get_fov,
                lookups.get_render_fov_method,
                "GameRenderer.getFov"
            );
        }

        clear_exception(&mut env, "post-game-renderer");

        // ── Camera methods ────────────────────────────────────────────────────
        let (get_position, get_yaw, get_pitch) = if is_fabric {
            (
                &mappings::fabric_camera::GET_POSITION,
                &mappings::fabric_camera::GET_Y_ROT,
                &mappings::fabric_camera::GET_X_ROT,
            )
        } else {
            (
                &mappings::camera::GET_POSITION,
                &mappings::camera::GET_Y_ROT,
                &mappings::camera::GET_X_ROT,
            )
        };

        if let Some(ref cam_global) = lookups.camera_class {
            let cam_cls = global_ref_as_class(cam_global);
            get_method!(
                &mut env,
                &cam_cls,
                get_position,
                lookups.get_camera_position_method,
                "Camera.getPosition"
            );
            get_method!(
                &mut env,
                &cam_cls,
                get_yaw,
                lookups.get_camera_yaw_method,
                "Camera.getYaw"
            );
            get_method!(
                &mut env,
                &cam_cls,
                get_pitch,
                lookups.get_camera_pitch_method,
                "Camera.getPitch"
            );
        }

        clear_exception(&mut env, "post-camera");

        // ── Vec3 fields ───────────────────────────────────────────────────────
        let (vec3_x, vec3_y, vec3_z) = if is_fabric {
            (
                &mappings::fabric_vec3::X,
                &mappings::fabric_vec3::Y,
                &mappings::fabric_vec3::Z,
            )
        } else {
            (&mappings::vec3::X, &mappings::vec3::Y, &mappings::vec3::Z)
        };

        if let Some(ref v3_global) = lookups.vec3_class {
            let v3_cls = global_ref_as_class(v3_global);
            get_field!(&mut env, &v3_cls, vec3_x, lookups.vec3_x_field, "Vec3.x");
            get_field!(&mut env, &v3_cls, vec3_y, lookups.vec3_y_field, "Vec3.y");
            get_field!(&mut env, &v3_cls, vec3_z, lookups.vec3_z_field, "Vec3.z");
        }

        clear_exception(&mut env, "post-vec3");

        // ── Validate required lookups ─────────────────────────────────────────
        let missing_required = lookups.get_players_method.is_none()
            || lookups.list_size_method.is_none()
            || lookups.list_get_method.is_none()
            || lookups.get_x_method.is_none()
            || lookups.get_y_method.is_none()
            || lookups.get_z_method.is_none()
            || lookups.old_x_field.is_none()
            || lookups.old_y_field.is_none()
            || lookups.old_z_field.is_none();

        if missing_required {
            esp_log(
                "[JniLookups] CRITICAL: Required entity/level lookups missing — ESP disabled\n",
            );
            esp_log(&format!(
                "  get_players={} list_size={} list_get={}\n",
                lookups.get_players_method.is_some(),
                lookups.list_size_method.is_some(),
                lookups.list_get_method.is_some()
            ));
            esp_log(&format!(
                "  get_x={} get_y={} get_z={}\n",
                lookups.get_x_method.is_some(),
                lookups.get_y_method.is_some(),
                lookups.get_z_method.is_some()
            ));
            esp_log(&format!(
                "  old_x={} old_y={} old_z={}\n",
                lookups.old_x_field.is_some(),
                lookups.old_y_field.is_some(),
                lookups.old_z_field.is_some()
            ));
            return None;
        }

        let missing_camera = lookups.game_renderer_field.is_none()
            || lookups.get_main_camera_method.is_none()
            || lookups.get_render_fov_method.is_none()
            || lookups.get_camera_position_method.is_none()
            || lookups.get_camera_yaw_method.is_none()
            || lookups.get_camera_pitch_method.is_none()
            || lookups.vec3_x_field.is_none()
            || lookups.vec3_y_field.is_none()
            || lookups.vec3_z_field.is_none();

        if missing_camera {
            esp_log("[JniLookups] CRITICAL: Required camera lookups missing — ESP disabled\n");
            esp_log(&format!(
                "  game_renderer_field={} get_main_camera={} get_fov={}\n",
                lookups.game_renderer_field.is_some(),
                lookups.get_main_camera_method.is_some(),
                lookups.get_render_fov_method.is_some()
            ));
            esp_log(&format!(
                "  get_position={} get_yaw={} get_pitch={}\n",
                lookups.get_camera_position_method.is_some(),
                lookups.get_camera_yaw_method.is_some(),
                lookups.get_camera_pitch_method.is_some()
            ));
            esp_log(&format!(
                "  vec3_x={} vec3_y={} vec3_z={}\n",
                lookups.vec3_x_field.is_some(),
                lookups.vec3_y_field.is_some(),
                lookups.vec3_z_field.is_some()
            ));
            return None;
        }

        if lookups.get_entity_id_method.is_none() {
            esp_log("[JniLookups] WARNING: Entity.getId not found — name cache disabled\n");
        }
        if lookups.get_health_method.is_none() {
            esp_log("[JniLookups] WARNING: LivingEntity.getHealth not found — health display disabled\n");
        }

        esp_log("[JniLookups] All lookups initialized successfully\n");
        Some(lookups)
    }

    pub fn is_ready(&self) -> bool {
        self.level_field.is_some()
            && self.get_players_method.is_some()
            && self.list_size_method.is_some()
            && self.list_get_method.is_some()
            && self.get_x_method.is_some()
            && self.get_y_method.is_some()
            && self.get_z_method.is_some()
            && self.old_x_field.is_some()
            && self.old_y_field.is_some()
            && self.old_z_field.is_some()
            && self.game_renderer_field.is_some()
            && self.get_main_camera_method.is_some()
            && self.get_render_fov_method.is_some()
            && self.get_camera_position_method.is_some()
            && self.get_camera_yaw_method.is_some()
            && self.get_camera_pitch_method.is_some()
            && self.vec3_x_field.is_some()
            && self.vec3_y_field.is_some()
            && self.vec3_z_field.is_some()
    }

    pub fn can_resolve_names(&self) -> bool {
        self.get_entity_id_method.is_some()
            && self.get_name_method.is_some()
            && self.get_string_method.is_some()
    }

    pub fn can_read_health(&self) -> bool {
        self.get_health_method.is_some()
    }

    pub fn can_read_frame_time(&self) -> bool {
        self.delta_tracker_field.is_some() && self.get_game_time_delta_method.is_some()
    }
}

impl Drop for JniLookups {
    fn drop(&mut self) {}
}
