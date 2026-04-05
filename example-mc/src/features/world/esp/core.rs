use super::cache::NameCache;
use super::types::*;
use crate::core::client::Minecraft;
use crate::jni::class_cache;
use crate::jni::env::JniEnvironment;
use crate::jni::lookups::JniLookups;
use crate::jni::mappings;
use crate::utils::math;
use glam::DVec3;
use jni::objects::{JObject, JValue};
use parking_lot::Mutex;
use windows::Win32::System::SystemInformation::GetTickCount64;

pub struct Esp {
    lookups: Option<JniLookups>,
    name_cache: Mutex<NameCache>,
    state: Mutex<EspState>,
    initialized: bool,
}

struct EspState {
    targets: Vec<Target>,
    debug: DebugState,
}

impl Esp {
    pub fn new() -> Self {
        let lookups = JniLookups::init();
        let initialized = lookups.as_ref().map_or(false, |l| l.is_ready());

        let last_status = if lookups.is_none() {
            "JniLookups::init failed (attach JVM / Minecraft.class / mappings)".into()
        } else if !initialized {
            "JniLookups loaded but is_ready() false".into()
        } else {
            "Ready (waiting for tick)".into()
        };

        Self {
            lookups,
            name_cache: Mutex::new(NameCache::new()),
            state: Mutex::new(EspState {
                targets: Vec::new(),
                debug: DebugState {
                    initialized,
                    last_status,
                    ..Default::default()
                },
            }),
            initialized,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    // ------------------------------------------------------------------
    // Read partial-tick frame time from DeltaTracker
    // ------------------------------------------------------------------
    fn read_frame_time(&self, env: &mut jni::JNIEnv) -> f32 {
        let Some(lookups) = &self.lookups else {
            return 1.0;
        };
        if !lookups.can_read_frame_time() {
            return 1.0;
        }

        let is_fabric = class_cache::is_fabric();

        let result: Option<f32> = Minecraft::with(|mc| {
            let inst = mc.instance()?;

            // ── DeltaTracker field (fabric vs vanilla) ────────────────────────
            let delta_field = if is_fabric {
                &mappings::fabric_minecraft::DELTA_TRACKER
            } else {
                &mappings::minecraft::DELTA_TRACKER
            };

            let delta_obj = env
                .get_field(&inst, delta_field.name, delta_field.signature)
                .ok()
                .and_then(|v| v.l().ok())?;

            // ── getGameTimeDeltaPartialTick (fabric vs vanilla) ───────────────
            let tick_method = if is_fabric {
                &mappings::fabric_delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
            } else {
                &mappings::delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
            };

            let frame_time = env
                .call_method(
                    &delta_obj,
                    tick_method.name,
                    tick_method.signature,
                    &[JValue::Bool(1)],
                )
                .ok()
                .and_then(|v| v.f().ok())?;

            if frame_time.is_finite() {
                Some(frame_time.clamp(0.0, 1.0))
            } else {
                None
            }
        })
        .flatten();

        result.unwrap_or(1.0)
    }

    // ------------------------------------------------------------------
    // Main tick — called every wglSwapBuffers
    // ------------------------------------------------------------------
    pub fn tick(&self) {
        if !self.initialized {
            return;
        }

        let Some(mut env) = JniEnvironment::get_current_or_attach("ESP tick") else {
            self.set_status("JNIEnv unavailable");
            return;
        };

        let current_tick = unsafe { GetTickCount64() };
        let Some(lookups) = &self.lookups else {
            return;
        };

        let is_fabric = class_cache::is_fabric();

        let _frame_time = self.read_frame_time(&mut env);

        // ── Minecraft.level (fabric vs vanilla) ───────────────────────────────
        let level_field = if is_fabric {
            &mappings::fabric_minecraft::LEVEL
        } else {
            &mappings::minecraft::LEVEL
        };

        let level_obj = Minecraft::with(|mc| {
            let Some(inst) = mc.instance() else {
                return None;
            };
            let Some(mut env) = JniEnvironment::get_current_or_attach("ESP level") else {
                return None;
            };
            env.get_field(&inst, level_field.name, level_field.signature)
                .ok()
                .and_then(|v| v.l().ok())
        });

        let Some(Some(level_obj)) = level_obj else {
            self.set_status("level unavailable");
            let mut state = self.state.lock();
            state.debug.level_valid = false;
            return;
        };

        // ── ClientLevel.players() (fabric vs vanilla) ─────────────────────────
        let players_method = if is_fabric {
            &mappings::fabric_client_level::PLAYERS
        } else {
            &mappings::client_level::PLAYERS
        };

        let player_list = match env.call_method(
            &level_obj,
            players_method.name,
            players_method.signature,
            &[],
        ) {
            Ok(val) => val.l().ok(),
            Err(_) => {
                self.set_status("player list unavailable");
                let mut state = self.state.lock();
                state.debug.player_list_valid = false;
                return;
            }
        };

        let Some(player_list) = player_list else {
            self.set_status("player list null");
            let mut state = self.state.lock();
            state.debug.player_list_valid = false;
            return;
        };

        // list.size() / list.get() — java.util.List, same on both loaders
        let count = match env.call_method(
            &player_list,
            mappings::java_list::SIZE.name,
            mappings::java_list::SIZE.signature,
            &[],
        ) {
            Ok(val) => match val.i() {
                Ok(c) => c as usize,
                Err(_) => return,
            },
            Err(_) => return,
        };

        let mut new_targets = Vec::with_capacity(count);
        let local_player = crate::core::local_player::LocalPlayer::get_object();

        let mut state = self.state.lock();
        state.debug.level_valid = true;
        state.debug.player_list_valid = true;
        state.debug.player_count = count;

        // ── Pick entity mappings once for the whole loop ──────────────────────
        let (old_x_m, old_y_m, old_z_m) = if is_fabric {
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

        let (get_x_m, get_y_m, get_z_m, get_eye_y_m) = if is_fabric {
            (
                &mappings::fabric_entity::GET_X,
                &mappings::fabric_entity::GET_Y,
                &mappings::fabric_entity::GET_Z,
                &mappings::fabric_entity::GET_EYE_Y,
            )
        } else {
            (
                &mappings::entity::GET_X,
                &mappings::entity::GET_Y,
                &mappings::entity::GET_Z,
                &mappings::entity::GET_EYE_Y,
            )
        };

        let (get_id_m, get_name_m) = if is_fabric {
            (
                &mappings::fabric_entity::GET_ID,
                &mappings::fabric_entity::GET_NAME,
            )
        } else {
            (&mappings::entity::GET_ID, &mappings::entity::GET_NAME)
        };

        let get_health_m = if is_fabric {
            &mappings::fabric_player::GET_HEALTH
        } else {
            &mappings::player::GET_HEALTH
        };

        let get_string_m = if is_fabric {
            &mappings::fabric_component::GET_STRING
        } else {
            &mappings::component::GET_STRING
        };

        for i in 0..count {
            let player = match env.call_method(
                &player_list,
                mappings::java_list::GET.name,
                mappings::java_list::GET.signature,
                &[JValue::Int(i.try_into().unwrap_or(0))],
            ) {
                Ok(val) => match val.l() {
                    Ok(o) => o,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            // Skip local player
            if let Some(ref lp) = local_player {
                if env.is_same_object(&player, lp).unwrap_or(false) {
                    continue;
                }
            }

            // ── Previous position ─────────────────────────────────────────────
            let old_x = env
                .get_field(&player, old_x_m.name, old_x_m.signature)
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);
            let old_y = env
                .get_field(&player, old_y_m.name, old_y_m.signature)
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);
            let old_z = env
                .get_field(&player, old_z_m.name, old_z_m.signature)
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);

            // ── Current position ──────────────────────────────────────────────
            let curr_x = env
                .call_method(&player, get_x_m.name, get_x_m.signature, &[])
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);
            let curr_y = env
                .call_method(&player, get_y_m.name, get_y_m.signature, &[])
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);
            let curr_z = env
                .call_method(&player, get_z_m.name, get_z_m.signature, &[])
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(0.0);
            let eye_y = env
                .call_method(&player, get_eye_y_m.name, get_eye_y_m.signature, &[])
                .ok()
                .and_then(|v| v.d().ok())
                .unwrap_or(curr_y);

            // ── Entity ID ─────────────────────────────────────────────────────
            let entity_id: i32 = if lookups.get_entity_id_method.is_some() {
                env.call_method(&player, get_id_m.name, get_id_m.signature, &[])
                    .ok()
                    .and_then(|v| v.i().ok())
                    .unwrap_or(0)
            } else {
                0
            };

            // ── Health ────────────────────────────────────────────────────────
            let health: f32 = if lookups.can_read_health() {
                let is_living = if let Some(ref living_cls) = lookups.living_entity_class {
                    env.is_instance_of(&player, living_cls).unwrap_or(false)
                } else {
                    true
                };

                if is_living {
                    env.call_method(&player, get_health_m.name, get_health_m.signature, &[])
                        .ok()
                        .and_then(|v| v.f().ok())
                        .unwrap_or(-1.0)
                } else {
                    -1.0
                }
            } else {
                -1.0
            };

            // ── Name (cached) ─────────────────────────────────────────────────
            let name = if entity_id != 0 && lookups.can_resolve_names() {
                let get_name_m = get_name_m;
                let get_string_m = get_string_m;
                self.name_cache
                    .lock()
                    .resolve_or_query(entity_id, current_tick, || {
                        Self::query_player_name_mapped(
                            &mut env,
                            &player,
                            lookups,
                            get_name_m,
                            get_string_m,
                        )
                    })
            } else {
                String::new()
            };

            new_targets.push(Target {
                previous_pos: DVec3::new(old_x, old_y, old_z),
                current_pos: DVec3::new(curr_x, curr_y, curr_z),
                entity_id,
                eye_height_offset: eye_y - curr_y,
                health,
                name,
            });
        }

        self.name_cache.lock().prune(current_tick);

        state.targets = new_targets;
        state.debug.target_count = state.targets.len();
        state.debug.last_status = "Tick OK".into();
    }

    // ------------------------------------------------------------------
    // Query a player's display name — mapping-aware version
    // ------------------------------------------------------------------
    fn query_player_name_mapped(
        env: &mut jni::JNIEnv,
        player: &JObject,
        lookups: &JniLookups,
        get_name_m: &mappings::MemberId,
        get_string_m: &mappings::MemberId,
    ) -> String {
        if lookups.get_name_method.is_none() || lookups.get_string_method.is_none() {
            return String::new();
        }

        let component = match env.call_method(player, get_name_m.name, get_name_m.signature, &[]) {
            Ok(val) => match val.l() {
                Ok(o) => o,
                Err(_) => return String::new(),
            },
            Err(_) => return String::new(),
        };

        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return String::new();
        }

        let name_jstr =
            match env.call_method(&component, get_string_m.name, get_string_m.signature, &[]) {
                Ok(val) => match val.l() {
                    Ok(o) => o,
                    Err(_) => return String::new(),
                },
                Err(_) => return String::new(),
            };

        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return String::new();
        }

        let jstr: jni::objects::JString =
            unsafe { jni::objects::JString::from_raw(name_jstr.as_raw()) };
        env.get_string(&jstr).map(|s| s.into()).unwrap_or_default()
    }

    fn set_status(&self, status: &str) {
        self.state.lock().debug.last_status = status.into();
    }

    pub fn get_instance() -> Option<&'static Esp> {
        crate::ESP.get()
    }

    pub fn get_snapshot(&self) -> RenderSnapshot {
        let (targets, debug_state, initialized) = if let Some(guard) = self.state.try_lock() {
            (guard.targets.clone(), guard.debug.clone(), self.initialized)
        } else {
            (
                Vec::new(),
                DebugState {
                    initialized: false,
                    last_status: "ESP State Error".into(),
                    ..Default::default()
                },
                false,
            )
        };

        let _camera = self.capture_camera_state();

        RenderSnapshot {
            targets,
            debug_state,
            initialized,
        }
    }

    // ------------------------------------------------------------------
    // Capture camera state
    // ------------------------------------------------------------------
    pub fn capture_camera_state(&self) -> Option<RenderCameraState> {
        let Some(_lookups) = &self.lookups else {
            return None;
        };

        let is_fabric = class_cache::is_fabric();

        // ── Pick all camera-related mappings up front ─────────────────────────
        let game_renderer_field = if is_fabric {
            &mappings::fabric_minecraft::GAME_RENDERER
        } else {
            &mappings::minecraft::GAME_RENDERER
        };

        let get_main_camera_m = if is_fabric {
            &mappings::fabric_game_renderer::GET_MAIN_CAMERA
        } else {
            &mappings::game_renderer::GET_MAIN_CAMERA
        };

        let get_fov_m = if is_fabric {
            &mappings::fabric_game_renderer::GET_FOV
        } else {
            &mappings::game_renderer::GET_FOV
        };

        let get_position_m = if is_fabric {
            &mappings::fabric_camera::GET_POSITION
        } else {
            &mappings::camera::GET_POSITION
        };

        let get_yaw_m = if is_fabric {
            &mappings::fabric_camera::GET_Y_ROT
        } else {
            &mappings::camera::GET_Y_ROT
        };

        let get_pitch_m = if is_fabric {
            &mappings::fabric_camera::GET_X_ROT
        } else {
            &mappings::camera::GET_X_ROT
        };

        let (vec3_x_m, vec3_y_m, vec3_z_m) = if is_fabric {
            (
                &mappings::fabric_vec3::X,
                &mappings::fabric_vec3::Y,
                &mappings::fabric_vec3::Z,
            )
        } else {
            (&mappings::vec3::X, &mappings::vec3::Y, &mappings::vec3::Z)
        };

        let camera_data = Minecraft::with(|mc| {
            let Some(inst) = mc.instance() else {
                return None;
            };
            let Some(mut env) = JniEnvironment::get_current_or_attach("ESP camera") else {
                return None;
            };

            // Minecraft.gameRenderer
            let game_renderer = env
                .get_field(
                    &inst,
                    game_renderer_field.name,
                    game_renderer_field.signature,
                )
                .ok()?
                .l()
                .ok()?;

            // GameRenderer.getMainCamera()
            let camera = env
                .call_method(
                    &game_renderer,
                    get_main_camera_m.name,
                    get_main_camera_m.signature,
                    &[],
                )
                .ok()?
                .l()
                .ok()?;

            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_clear();
                return None;
            }

            // Camera.getPosition() -> Vec3
            let position = env
                .call_method(&camera, get_position_m.name, get_position_m.signature, &[])
                .ok()?
                .l()
                .ok()?;

            let cam_x = env
                .get_field(&position, vec3_x_m.name, vec3_x_m.signature)
                .ok()
                .and_then(|v| v.d().ok())?;
            let cam_y = env
                .get_field(&position, vec3_y_m.name, vec3_y_m.signature)
                .ok()
                .and_then(|v| v.d().ok())?;
            let cam_z = env
                .get_field(&position, vec3_z_m.name, vec3_z_m.signature)
                .ok()
                .and_then(|v| v.d().ok())?;

            let yaw = env
                .call_method(&camera, get_yaw_m.name, get_yaw_m.signature, &[])
                .ok()
                .and_then(|v| v.f().ok())? as f64;

            let pitch = env
                .call_method(&camera, get_pitch_m.name, get_pitch_m.signature, &[])
                .ok()
                .and_then(|v| v.f().ok())? as f64;

            // getFov(Camera, float, boolean)
            let frame_time: f32 = 1.0;
            let fov = env
                .call_method(
                    &game_renderer,
                    get_fov_m.name,
                    get_fov_m.signature,
                    &[
                        JValue::Object(&camera),
                        JValue::Float(frame_time),
                        JValue::Bool(1),
                    ],
                )
                .ok()
                .and_then(|v| v.f().ok())
                .unwrap_or(70.0) as f64;

            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_clear();
            }

            Some((cam_x, cam_y, cam_z, yaw, pitch, fov))
        })?;

        let Some((cam_x, cam_y, cam_z, yaw, pitch, fov)) = camera_data else {
            return None;
        };

        Some(RenderCameraState {
            camera_state: CameraState {
                previous_pos: DVec3::new(cam_x, cam_y, cam_z),
                current_pos: DVec3::new(cam_x, cam_y, cam_z),
                eye_height_offset: 0.0,
                yaw_degrees: math::normalize_degrees(yaw),
                pitch_degrees: math::normalize_degrees(pitch),
                fov_degrees: fov.clamp(30.0, 170.0),
                valid: true,
            },
            interpolation_alpha: 1.0,
            valid: true,
        })
    }
}
