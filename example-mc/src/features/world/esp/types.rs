use glam::DVec3;

#[derive(Clone, Debug)]
pub struct Target {
    pub previous_pos: DVec3,
    pub current_pos: DVec3,
    pub entity_id: i32,
    pub eye_height_offset: f64,
    pub health: f32, // -1.0 if unavailable
    pub name: String,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            previous_pos: DVec3::ZERO,
            current_pos: DVec3::ZERO,
            entity_id: 0,
            eye_height_offset: 0.0,
            health: -1.0,
            name: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CameraState {
    pub previous_pos: DVec3,
    pub current_pos: DVec3,
    pub eye_height_offset: f64,
    pub yaw_degrees: f64,
    pub pitch_degrees: f64,
    pub fov_degrees: f64,
    pub valid: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            previous_pos: DVec3::ZERO,
            current_pos: DVec3::ZERO,
            eye_height_offset: 0.0,
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
            fov_degrees: 90.0,
            valid: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DebugState {
    pub initialized: bool,
    pub local_player_valid: bool,
    pub level_valid: bool,
    pub player_list_valid: bool,
    pub camera_valid: bool,
    pub render_camera_available: bool,
    pub render_camera_used: bool,
    pub player_count: usize,
    pub target_count: usize,
    pub camera_pos: DVec3,
    pub yaw_degrees: f64,
    pub pitch_degrees: f64,
    pub fov_degrees: f64,
    pub last_status: String,
    pub lookup_details: String,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            initialized: false,
            local_player_valid: false,
            level_valid: false,
            player_list_valid: false,
            camera_valid: false,
            render_camera_available: false,
            render_camera_used: false,
            player_count: 0,
            target_count: 0,
            camera_pos: DVec3::ZERO,
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
            fov_degrees: 90.0,
            last_status: "ESP not initialized".into(),
            lookup_details: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct NameCacheEntry {
    pub entity_id: i32,
    pub name: String,
    pub last_refresh_tick: u64,
    pub last_seen_tick: u64,
}

#[derive(Clone)]
pub struct RenderCameraState {
    pub camera_state: CameraState,
    pub interpolation_alpha: f32,
    pub valid: bool,
}

#[derive(Clone)]
pub struct RenderSnapshot {
    pub targets: Vec<Target>,
    pub debug_state: DebugState,
    pub initialized: bool,
}
