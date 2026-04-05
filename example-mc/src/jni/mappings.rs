// Centralized Minecraft JNI symbol table.
// Port of C++ `mc_mappings` — update obfuscated names here for game updates.

pub struct MemberId {
    pub name: &'static str,
    pub signature: &'static str,
}

/// Vanilla 1.21 ProGuard-obfuscated class names (used when running unmodded).
pub mod classes {
    pub const MINECRAFT: &str = "gfj";
    pub const CLIENT_LEVEL: &str = "hif";
    pub const DELTA_TRACKER: &str = "gez";
    pub const DELTA_TRACKER_TIMER: &str = "gez$b";
    pub const DELTA_TRACKER_DEFAULT_VALUE: &str = "gez$a";
    pub const GAME_RENDERER: &str = "hob";
    pub const CAMERA: &str = "ger";
    pub const VEC3: &str = "ftm";
    pub const OPTIONS: &str = "gfo";
    pub const OPTION_INSTANCE: &str = "gfn";
    pub const ENTITY: &str = "cgk";
    pub const LIVING_ENTITY: &str = "chl";
    pub const ABILITIES: &str = "ddi";
    pub const LOCAL_PLAYER: &str = "hnh";
    pub const PLAYER: &str = "ddm";
    pub const INVENTORY: &str = "ddl";
    pub const GAME_MODE: &str = "hio";
    pub const HIT_RESULT: &str = "ftk";
    pub const BLOCK_HIT_RESULT: &str = "fti";
    pub const ENTITY_HIT_RESULT: &str = "ftj";
    pub const COMPONENT: &str = "yh";
    pub const ITEM_STACK: &str = "dlt";
    pub const ITEM: &str = "dlp";
    pub const AXE_ITEM: &str = "djy";
    pub const CLIENT_PACKET_LISTENER: &str = "hig";
    pub const CLIENT_COMMON_PACKET_LISTENER_IMPL: &str = "hia";
    pub const PACKET: &str = "aay";
    pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET: &str = "ajt";
    pub const JAVA_LIST: &str = "java/util/List";
    pub const JAVA_NUMBER: &str = "java/lang/Number";
    pub const JAVA_DOUBLE: &str = "java/lang/Double";
}

// ============================================================================
// Fabric 1.21 intermediary mappings
// Class names use dots replaced with slashes (JNI internal format).
// These are used when class_cache::is_fabric() returns true.
// ============================================================================

/// Fabric 1.21 intermediary class names.
pub mod fabric_classes {
    pub const MINECRAFT: &str = "net/minecraft/class_310";
    pub const CLIENT_LEVEL: &str = "net/minecraft/class_638";
    pub const DELTA_TRACKER: &str = "net/minecraft/class_9779";
    pub const DELTA_TRACKER_TIMER: &str = "net/minecraft/class_9779$class_9781";
    pub const DELTA_TRACKER_DEFAULT_VALUE: &str = "net/minecraft/class_9779$class_9780";
    pub const GAME_RENDERER: &str = "net/minecraft/class_757";
    pub const CAMERA: &str = "net/minecraft/class_4184";
    pub const VEC3: &str = "net/minecraft/class_243";
    pub const OPTIONS: &str = "net/minecraft/class_315";
    pub const OPTION_INSTANCE: &str = "net/minecraft/class_7172";
    pub const ENTITY: &str = "net/minecraft/class_1297";
    pub const LIVING_ENTITY: &str = "net/minecraft/class_1309";
    pub const ABILITIES: &str = "net/minecraft/class_1656";
    pub const LOCAL_PLAYER: &str = "net/minecraft/class_746";
    pub const PLAYER: &str = "net/minecraft/class_1657";
    pub const INVENTORY: &str = "net/minecraft/class_1661";
    pub const GAME_MODE: &str = "net/minecraft/class_636";
    pub const HIT_RESULT: &str = "net/minecraft/class_239";
    pub const BLOCK_HIT_RESULT: &str = "net/minecraft/class_3965";
    pub const ENTITY_HIT_RESULT: &str = "net/minecraft/class_3966";
    pub const COMPONENT: &str = "net/minecraft/class_2561";
    pub const ITEM_STACK: &str = "net/minecraft/class_1799";
    pub const ITEM: &str = "net/minecraft/class_1792";
    pub const AXE_ITEM: &str = "net/minecraft/class_1743";
    pub const CLIENT_PACKET_LISTENER: &str = "net/minecraft/class_634";
    pub const CLIENT_COMMON_PACKET_LISTENER_IMPL: &str = "net/minecraft/class_8673";
    pub const PACKET: &str = "net/minecraft/class_2596";
    pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET: &str = "net/minecraft/class_2868";
    pub const JAVA_LIST: &str = "java/util/List";
    pub const JAVA_NUMBER: &str = "java/lang/Number";
    pub const JAVA_DOUBLE: &str = "java/lang/Double";
}

/// Fabric 1.21 intermediary fields and methods for net.minecraft.class_310 (Minecraft).
pub mod fabric_minecraft {
    use super::MemberId;

    pub const INSTANCE: MemberId = MemberId {
        name: "field_1700",
        signature: "Lnet/minecraft/class_310;",
    };
    /// The current ClientWorld/level field.
    pub const LEVEL: MemberId = MemberId {
        name: "field_1687",
        signature: "Lnet/minecraft/class_638;",
    };
    /// The stored RenderTickCounter.Dynamic field (concrete subtype of class_9779).
    pub const DELTA_TRACKER: MemberId = MemberId {
        name: "field_52750",
        signature: "Lnet/minecraft/class_9779$class_9781;",
    };
    /// getRenderTickCounter() – returns the interface type class_9779.
    pub const GET_DELTA_TRACKER: MemberId = MemberId {
        name: "method_61966",
        signature: "()Lnet/minecraft/class_9779;",
    };
    pub const GAME_RENDERER: MemberId = MemberId {
        name: "field_1773",
        signature: "Lnet/minecraft/class_757;",
    };
    pub const OPTIONS: MemberId = MemberId {
        name: "field_1690",
        signature: "Lnet/minecraft/class_315;",
    };
    pub const PLAYER: MemberId = MemberId {
        name: "field_1724",
        signature: "Lnet/minecraft/class_746;",
    };
    /// Alias for PLAYER – same field, matches vanilla LOCAL_PLAYER naming.
    pub const LOCAL_PLAYER: MemberId = MemberId {
        name: "field_1724",
        signature: "Lnet/minecraft/class_746;",
    };
    pub const GUI: MemberId = MemberId {
        name: "field_1705",
        signature: "Lnet/minecraft/class_329;",
    };
    pub const GAME_MODE: MemberId = MemberId {
        name: "field_1761",
        signature: "Lnet/minecraft/class_636;",
    };
    pub const SCREEN: MemberId = MemberId {
        name: "field_1755",
        signature: "Lnet/minecraft/class_437;",
    };
    pub const HIT_RESULT: MemberId = MemberId {
        name: "field_1765",
        signature: "Lnet/minecraft/class_239;",
    };
    /// itemUseCooldown – the right-click use delay countdown int.
    pub const RIGHT_CLICK_DELAY: MemberId = MemberId {
        name: "field_1752",
        signature: "I",
    };
    pub const IS_PAUSED: MemberId = MemberId {
        name: "method_1493",
        signature: "()Z",
    };
    /// doAttack() – performs a left-click / attack action.
    pub const START_ATTACK: MemberId = MemberId {
        name: "method_1536",
        signature: "()Z",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_757 (GameRenderer).
pub mod fabric_game_renderer {
    use super::MemberId;

    pub const GET_MAIN_CAMERA: MemberId = MemberId {
        name: "method_19418",
        signature: "()Lnet/minecraft/class_4184;",
    };
    /// getFov(Camera, float, boolean) → double
    pub const GET_FOV: MemberId = MemberId {
        name: "method_3196",
        signature: "(Lnet/minecraft/class_4184;FZ)F",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_638 (ClientWorld).
pub mod fabric_client_level {
    use super::MemberId;

    /// getPlayers() – declared on EntityView (class_1924), inherited by ClientWorld.
    pub const PLAYERS: MemberId = MemberId {
        name: "method_18456",
        signature: "()Ljava/util/List;",
    };
}

/// Fabric 1.21 intermediary fields for net.minecraft.class_315 (GameOptions).
pub mod fabric_options {
    use super::MemberId;

    pub const GAMMA: MemberId = MemberId {
        name: "field_1840",
        signature: "Lnet/minecraft/class_7172;",
    };
    pub const FOV: MemberId = MemberId {
        name: "field_1826",
        signature: "Lnet/minecraft/class_7172;",
    };
}

/// Fabric 1.21 intermediary fields for net.minecraft.class_7172 (SimpleOption).
pub mod fabric_option_instance {
    use super::MemberId;

    /// The raw stored generic value (cast to Double for gamma, Integer for fov).
    pub const VALUE: MemberId = MemberId {
        name: "field_37868",
        signature: "Ljava/lang/Object;",
    };
}

/// Fabric 1.21 intermediary fields/methods for net.minecraft.class_746 (ClientPlayerEntity).
pub mod fabric_local_player {
    use super::MemberId;

    /// setSprinting(boolean) – declared on Entity (class_1297).
    pub const SET_SPRINTING: MemberId = MemberId {
        name: "method_5728",
        signature: "(Z)V",
    };
    /// getAttackCooldownProgress(float) – declared on PlayerEntity (class_1657).
    pub const ATTACK_STRENGTH_SCALE: MemberId = MemberId {
        name: "method_7261",
        signature: "(F)F",
    };
    /// setVelocity(double, double, double) – declared on Entity (class_1297).
    pub const SET_DELTA_MOVEMENT: MemberId = MemberId {
        name: "method_18800",
        signature: "(DDD)V",
    };
    /// hurtTime field – declared on LivingEntity (class_1309).
    pub const HURT_TIME: MemberId = MemberId {
        name: "field_6235",
        signature: "I",
    };
    /// networkHandler field (ClientPlayNetworkHandler / class_634).
    pub const CONNECTION: MemberId = MemberId {
        name: "field_3944",
        signature: "Lnet/minecraft/class_634;",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_1657 (PlayerEntity).
pub mod fabric_player {
    use super::MemberId;

    /// getHealth() – declared on LivingEntity (class_1309).
    pub const GET_HEALTH: MemberId = MemberId {
        name: "method_6032",
        signature: "()F",
    };
    pub const GET_INVENTORY: MemberId = MemberId {
        name: "method_31548",
        signature: "()Lnet/minecraft/class_1661;",
    };
    pub const GET_ABILITIES: MemberId = MemberId {
        name: "method_31549",
        signature: "()Lnet/minecraft/class_1656;",
    };
}

/// Fabric 1.21 intermediary fields for net.minecraft.class_1656 (PlayerAbilities).
pub mod fabric_abilities {
    use super::MemberId;

    pub const MAY_FLY: MemberId = MemberId {
        name: "field_7478",
        signature: "Z",
    };
    pub const FLYING: MemberId = MemberId {
        name: "field_7479",
        signature: "Z",
    };
    /// flySpeed is private – use getFlySpeed() / method_7252 if direct access fails.
    pub const FLY_SPEED: MemberId = MemberId {
        name: "field_7481",
        signature: "F",
    };
}

/// Fabric 1.21 intermediary fields/methods for net.minecraft.class_4184 (Camera).
pub mod fabric_camera {
    use super::MemberId;

    pub const GET_POSITION: MemberId = MemberId {
        name: "method_71156",
        signature: "()Lnet/minecraft/class_243;",
    };
    pub const GET_Y_ROT: MemberId = MemberId {
        name: "method_19330",
        signature: "()F",
    };
    pub const GET_X_ROT: MemberId = MemberId {
        name: "method_19329",
        signature: "()F",
    };
}

/// Fabric 1.21 intermediary fields for net.minecraft.class_243 (Vec3d).
pub mod fabric_vec3 {
    use super::MemberId;

    pub const X: MemberId = MemberId {
        name: "field_1352",
        signature: "D",
    };
    pub const Y: MemberId = MemberId {
        name: "field_1351",
        signature: "D",
    };
    pub const Z: MemberId = MemberId {
        name: "field_1350",
        signature: "D",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_9779 (RenderTickCounter interface).
pub mod fabric_delta_tracker {
    use super::MemberId;

    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "method_60637",
        signature: "(Z)F",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_9779$class_9781 (Dynamic impl).
pub mod fabric_delta_tracker_timer {
    use super::MemberId;

    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "method_60637",
        signature: "(Z)F",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_9779$class_9780 (Constant impl).
pub mod fabric_delta_tracker_default_value {
    use super::MemberId;

    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "method_60637",
        signature: "(Z)F",
    };
}

/// Fabric 1.21 intermediary fields/methods for net.minecraft.class_1661 (PlayerInventory).
pub mod fabric_inventory {
    use super::MemberId;

    /// getStack(int) – declared on Inventory interface (class_1263).
    pub const GET_ITEM: MemberId = MemberId {
        name: "method_5438",
        signature: "(I)Lnet/minecraft/class_1799;",
    };
    pub const SELECTED: MemberId = MemberId {
        name: "field_7545",
        signature: "I",
    };
}

/// Fabric 1.21 intermediary methods for net.minecraft.class_8673 (ClientCommonNetworkHandler).
pub mod fabric_client_common_packet_listener_impl {
    use super::MemberId;

    pub const SEND: MemberId = MemberId {
        name: "method_52787",
        signature: "(Lnet/minecraft/class_2596;)V",
    };
}

/// Fabric 1.21 intermediary constructor for net.minecraft.class_2868 (UpdateSelectedSlotC2SPacket).
pub mod fabric_serverbound_set_carried_item_packet {
    use super::MemberId;

    pub const CONSTRUCTOR: MemberId = MemberId {
        name: "<init>",
        signature: "(I)V",
    };
}

/// Fabric 1.21 intermediary fields and methods for net.minecraft.class_1297 (Entity).
pub mod fabric_entity {
    use super::MemberId;

    /// getId() – declared on EntityLike interface (class_5568).
    pub const GET_ID: MemberId = MemberId {
        name: "method_5628",
        signature: "()I",
    };
    pub const GET_NAME: MemberId = MemberId {
        name: "method_5477",
        signature: "()Lnet/minecraft/class_2561;",
    };
    pub const GET_TYPE_NAME: MemberId = MemberId {
        name: "method_23315",
        signature: "()Lnet/minecraft/class_2561;",
    };
    pub const IS_ALIVE: MemberId = MemberId {
        name: "method_5805",
        signature: "()Z",
    };
    pub const GET_X: MemberId = MemberId {
        name: "method_23317",
        signature: "()D",
    };
    pub const GET_Y: MemberId = MemberId {
        name: "method_23318",
        signature: "()D",
    };
    pub const GET_EYE_Y: MemberId = MemberId {
        name: "method_23320",
        signature: "()D",
    };
    pub const GET_Z: MemberId = MemberId {
        name: "method_23321",
        signature: "()D",
    };
    pub const GET_YAW: MemberId = MemberId {
        name: "method_36454",
        signature: "()F",
    };
    pub const GET_PITCH: MemberId = MemberId {
        name: "method_36455",
        signature: "()F",
    };
    /// prevX – previous-tick X position used for client-side interpolation.
    pub const OLD_X: MemberId = MemberId {
        name: "field_6014",
        signature: "D",
    };
    /// prevY – previous-tick Y position.
    pub const OLD_Y: MemberId = MemberId {
        name: "field_6036",
        signature: "D",
    };
    /// prevZ – previous-tick Z position.
    pub const OLD_Z: MemberId = MemberId {
        name: "field_5969",
        signature: "D",
    };
}

pub mod fabric_living_entity {
    use super::MemberId;

    pub const IS_BLOCKING: MemberId = MemberId {
        name: "method_6039",
        signature: "()Z",
    };
    pub const IS_USING_ITEM: MemberId = MemberId {
        name: "method_6115",
        signature: "()Z",
    };
    pub const GET_ITEM_IN_HAND: MemberId = MemberId {
        name: "method_24520",
        signature: "(Lnet/minecraft/class_1268;)Lnet/minecraft/class_1799;",
    };
}

pub mod fabric_component {
    use super::MemberId;

    pub const GET_STRING: MemberId = MemberId {
        name: "method_74062",
        signature: "()Ljava/lang/String;",
    };
}

pub mod fabric_hit_result {
    use super::MemberId;

    pub const GET_TYPE: MemberId = MemberId {
        name: "method_17783",
        signature: "()Lnet/minecraft/class_239$class_240;",
    };
}

pub mod fabric_entity_hit_result {
    use super::MemberId;

    pub const GET_ENTITY: MemberId = MemberId {
        name: "method_17782",
        signature: "()Lnet/minecraft/class_1297;",
    };
}

pub mod fabric_item_stack {
    use super::MemberId;

    pub const IS_EMPTY: MemberId = MemberId {
        name: "method_7960",
        signature: "()Z",
    };
    pub const GET_ITEM: MemberId = MemberId {
        name: "method_57385",
        signature: "()Lnet/minecraft/class_1792;",
    };
}

pub mod fabric_item {
    use super::MemberId;

    pub const GET_NAME: MemberId = MemberId {
        name: "method_65043",
        signature: "(Lnet/minecraft/class_1799;)Lnet/minecraft/class_2561;",
    };
}

pub mod fabric_game_mode {
    use super::MemberId;

    pub const GET_PLAYER_MODE: MemberId = MemberId {
        name: "method_2920",
        signature: "()Lnet/minecraft/class_1934;",
    };
    pub const IS_DESTROYING: MemberId = MemberId {
        name: "method_2923",
        signature: "()Z",
    };
    pub const GET_DESTROY_STAGE: MemberId = MemberId {
        name: "method_51888",
        signature: "()I",
    };
    /// blockBreakingCooldown field.
    pub const DESTROY_DELAY: MemberId = MemberId {
        name: "field_3716",
        signature: "I",
    };
    /// currentBreakingProgress field.
    pub const DESTROY_PROGRESS: MemberId = MemberId {
        name: "field_3715",
        signature: "F",
    };
}

/// Returns the correct Minecraft class name for the currently detected loader.
/// Tries JVMTI cache for Fabric first, falls back to vanilla name.
pub fn minecraft_class() -> &'static str {
    if crate::jni::class_cache::is_fabric() {
        fabric_classes::MINECRAFT
    } else {
        classes::MINECRAFT
    }
}

/// Returns the correct INSTANCE field MemberId for the currently detected loader.
pub fn minecraft_instance_field() -> MemberId {
    if crate::jni::class_cache::is_fabric() {
        fabric_minecraft::INSTANCE
    } else {
        minecraft::INSTANCE
    }
}

pub mod minecraft {
    use super::MemberId;
    pub const INSTANCE: MemberId = MemberId {
        name: "A",
        signature: "Lgfj;",
    };
    pub const LEVEL: MemberId = MemberId {
        name: "r",
        signature: "Lhif;",
    };
    pub const DELTA_TRACKER: MemberId = MemberId {
        name: "P",
        signature: "Lgez$b;",
    };
    pub const GET_DELTA_TRACKER: MemberId = MemberId {
        name: "aD",
        signature: "()Lgez;",
    };
    pub const GAME_RENDERER: MemberId = MemberId {
        name: "i",
        signature: "Lhob;",
    };
    pub const OPTIONS: MemberId = MemberId {
        name: "k",
        signature: "Lgfo;",
    };
    pub const LOCAL_PLAYER: MemberId = MemberId {
        name: "s",
        signature: "Lhnh;",
    };
    pub const HIT_RESULT: MemberId = MemberId {
        name: "u",
        signature: "Lftk;",
    };
    pub const GAME_MODE: MemberId = MemberId {
        name: "q",
        signature: "Lhio;",
    };
    pub const RIGHT_CLICK_DELAY: MemberId = MemberId {
        name: "aR",
        signature: "I",
    };
    pub const START_ATTACK: MemberId = MemberId {
        name: "bu",
        signature: "()Z",
    };
}

pub mod game_renderer {
    use super::MemberId;
    pub const GET_MAIN_CAMERA: MemberId = MemberId {
        name: "p",
        signature: "()Lger;",
    };
    pub const GET_FOV: MemberId = MemberId {
        name: "a",
        signature: "(Lger;FZ)F",
    };
}

pub mod client_level {
    use super::MemberId;
    pub const PLAYERS: MemberId = MemberId {
        name: "E",
        signature: "()Ljava/util/List;",
    };
}

pub mod options {
    use super::MemberId;
    pub const GAMMA: MemberId = MemberId {
        name: "di",
        signature: "Lgfn;",
    };
    pub const FOV: MemberId = MemberId {
        name: "cT",
        signature: "Lgfn;",
    };
}

pub mod option_instance {
    use super::MemberId;
    pub const VALUE: MemberId = MemberId {
        name: "k",
        signature: "Ljava/lang/Object;",
    };
}

pub mod local_player {
    use super::MemberId;
    pub const SET_SPRINTING: MemberId = MemberId {
        name: "i",
        signature: "(Z)V",
    };
    pub const ATTACK_STRENGTH_SCALE: MemberId = MemberId {
        name: "I",
        signature: "(F)F",
    };
    pub const SET_DELTA_MOVEMENT: MemberId = MemberId {
        name: "m",
        signature: "(DDD)V",
    };
    pub const HURT_TIME: MemberId = MemberId {
        name: "bu",
        signature: "I",
    };
    pub const CONNECTION: MemberId = MemberId {
        name: "b",
        signature: "Lhig;",
    };
}

pub mod game_mode {
    use super::MemberId;
    pub const DESTROY_DELAY: MemberId = MemberId {
        name: "h",
        signature: "I",
    };
    pub const DESTROY_PROGRESS: MemberId = MemberId {
        name: "f",
        signature: "F",
    };
}

pub mod player {
    use super::MemberId;
    pub const GET_HEALTH: MemberId = MemberId {
        name: "eZ",
        signature: "()F",
    };
    pub const GET_INVENTORY: MemberId = MemberId {
        name: "gK",
        signature: "()Lddl;",
    };
    pub const GET_ABILITIES: MemberId = MemberId {
        name: "cG",
        signature: "Lddi;",
    };
}

pub mod abilities {
    use super::MemberId;
    pub const MAY_FLY: MemberId = MemberId {
        name: "c",
        signature: "Z",
    };
    pub const FLYING: MemberId = MemberId {
        name: "b",
        signature: "Z",
    };
    pub const FLY_SPEED: MemberId = MemberId {
        name: "m",
        signature: "F",
    };
}

pub mod entity_hit_result {
    use super::MemberId;
    pub const GET_ENTITY: MemberId = MemberId {
        name: "a",
        signature: "()Lcgk;",
    };
}

pub mod entity {
    use super::MemberId;
    pub const GET_ID: MemberId = MemberId {
        name: "aA",
        signature: "()I",
    };
    pub const GET_NAME: MemberId = MemberId {
        name: "ap",
        signature: "()Lyh;",
    };
    pub const IS_ALIVE: MemberId = MemberId {
        name: "cb",
        signature: "()Z",
    };
    pub const GET_X: MemberId = MemberId {
        name: "dP",
        signature: "()D",
    };
    pub const GET_Y: MemberId = MemberId {
        name: "dR",
        signature: "()D",
    };
    pub const GET_EYE_Y: MemberId = MemberId {
        name: "dT",
        signature: "()D",
    };
    pub const GET_Z: MemberId = MemberId {
        name: "dV",
        signature: "()D",
    };
    pub const GET_YAW: MemberId = MemberId {
        name: "ec",
        signature: "()F",
    };
    pub const GET_PITCH: MemberId = MemberId {
        name: "ee",
        signature: "()F",
    };
    pub const OLD_X: MemberId = MemberId {
        name: "Y",
        signature: "D",
    };
    pub const OLD_Y: MemberId = MemberId {
        name: "Z",
        signature: "D",
    };
    pub const OLD_Z: MemberId = MemberId {
        name: "aa",
        signature: "D",
    };
}

pub mod living_entity {
    use super::MemberId;
    pub const IS_BLOCKING: MemberId = MemberId {
        name: "gg",
        signature: "()Z",
    };
}

pub mod camera {
    use super::MemberId;
    pub const GET_POSITION: MemberId = MemberId {
        name: "b",
        signature: "()Lftm;",
    };
    pub const GET_Y_ROT: MemberId = MemberId {
        name: "f",
        signature: "()F",
    };
    pub const GET_X_ROT: MemberId = MemberId {
        name: "e",
        signature: "()F",
    };
}

pub mod vec3 {
    use super::MemberId;
    pub const X: MemberId = MemberId {
        name: "g",
        signature: "D",
    };
    pub const Y: MemberId = MemberId {
        name: "h",
        signature: "D",
    };
    pub const Z: MemberId = MemberId {
        name: "i",
        signature: "D",
    };
}

pub mod delta_tracker {
    use super::MemberId;
    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "a",
        signature: "(Z)F",
    };
}

pub mod delta_tracker_timer {
    use super::MemberId;
    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "a",
        signature: "(Z)F",
    };
}

pub mod delta_tracker_default_value {
    use super::MemberId;
    pub const GET_GAME_TIME_DELTA_PARTIAL_TICK: MemberId = MemberId {
        name: "a",
        signature: "(Z)F",
    };
}

pub mod component {
    use super::MemberId;
    pub const GET_STRING: MemberId = MemberId {
        name: "getString",
        signature: "()Ljava/lang/String;",
    };
}

pub mod inventory {
    use super::MemberId;
    pub const GET_ITEM: MemberId = MemberId {
        name: "a",
        signature: "(I)Ldlt;",
    };
    pub const SELECTED: MemberId = MemberId {
        name: "m",
        signature: "I",
    };
}

pub mod item_stack {
    use super::MemberId;
    pub const IS_EMPTY: MemberId = MemberId {
        name: "f",
        signature: "()Z",
    };
    pub const GET_ITEM: MemberId = MemberId {
        name: "h",
        signature: "()Ldlp;",
    };
}

pub mod client_common_packet_listener_impl {
    use super::MemberId;
    pub const SEND: MemberId = MemberId {
        name: "b",
        signature: "(Laay;)V",
    };
}

pub mod serverbound_set_carried_item_packet {
    use super::MemberId;
    pub const CONSTRUCTOR: MemberId = MemberId {
        name: "<init>",
        signature: "(I)V",
    };
}

pub mod java_list {
    use super::MemberId;
    pub const SIZE: MemberId = MemberId {
        name: "size",
        signature: "()I",
    };
    pub const GET: MemberId = MemberId {
        name: "get",
        signature: "(I)Ljava/lang/Object;",
    };
}

pub mod java_number {
    use super::MemberId;
    pub const DOUBLE_VALUE: MemberId = MemberId {
        name: "doubleValue",
        signature: "()D",
    };
}

pub mod java_double {
    use super::MemberId;
    pub const CONSTRUCTOR: MemberId = MemberId {
        name: "<init>",
        signature: "(D)V",
    };
}

// ============================================================================
// JNI Lookups - Cached Java class references and method/field IDs
// ============================================================================

use jni::objects::{GlobalRef, JFieldID, JMethodID};

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
        use crate::core::client::Minecraft;
        use crate::jni::env::JniEnvironment;

        let mut env = JniEnvironment::attach("JNI ESP")?;

        let using_fabric = crate::jni::class_cache::is_fabric();

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

        // --- Minecraft class & level field ---
        let mc_class_global: Option<GlobalRef> = Minecraft::with(|mc| -> Option<GlobalRef> {
            let cls = mc.class_ref()?;
            let mut env = JniEnvironment::get_current_or_attach("ESP lookups")?;
            env.new_global_ref(cls).ok()
        })
        .flatten();

        let mc_class_global = mc_class_global?;

        let level_mapping = if using_fabric {
            &fabric_minecraft::LEVEL
        } else {
            &minecraft::LEVEL
        };

        // level field — actually store it this time
        lookups.level_field = env
            .env()
            .get_field_id(
                &mc_class_global,
                level_mapping.name,
                level_mapping.signature,
            )
            .ok();

        if lookups.level_field.is_none() {
            println!(
                "[ESP] CRITICAL: Minecraft.level field not found using {} mappings — ESP disabled",
                if using_fabric { "Fabric" } else { "Vanilla" }
            );
            return None;
        }

        let delta_tracker_mapping = if using_fabric {
            &fabric_minecraft::DELTA_TRACKER
        } else {
            &minecraft::DELTA_TRACKER
        };

        // delta tracker field
        lookups.delta_tracker_field = env
            .env()
            .get_field_id(
                &mc_class_global,
                delta_tracker_mapping.name,
                delta_tracker_mapping.signature,
            )
            .ok();

        let game_renderer_mapping = if using_fabric {
            &fabric_minecraft::GAME_RENDERER
        } else {
            &minecraft::GAME_RENDERER
        };

        // game renderer field
        lookups.game_renderer_field = env
            .env()
            .get_field_id(
                &mc_class_global,
                game_renderer_mapping.name,
                game_renderer_mapping.signature,
            )
            .ok();

        let client_level_class = if using_fabric {
            fabric_classes::CLIENT_LEVEL
        } else {
            classes::CLIENT_LEVEL
        };

        // --- Classes ---
        lookups.level_class = env
            .env()
            .find_class(client_level_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        lookups.list_class = env
            .env()
            .find_class(classes::JAVA_LIST)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let delta_tracker_class = if using_fabric {
            fabric_classes::DELTA_TRACKER
        } else {
            classes::DELTA_TRACKER
        };

        lookups.delta_tracker_timer_class = env
            .env()
            .find_class(delta_tracker_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let game_renderer_class = if using_fabric {
            fabric_classes::GAME_RENDERER
        } else {
            classes::GAME_RENDERER
        };

        lookups.game_renderer_class = env
            .env()
            .find_class(game_renderer_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let camera_class = if using_fabric {
            fabric_classes::CAMERA
        } else {
            classes::CAMERA
        };

        lookups.camera_class = env
            .env()
            .find_class(camera_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let vec3_class = if using_fabric {
            fabric_classes::VEC3
        } else {
            classes::VEC3
        };

        lookups.vec3_class = env
            .env()
            .find_class(vec3_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let living_entity_class = if using_fabric {
            fabric_classes::LIVING_ENTITY
        } else {
            classes::LIVING_ENTITY
        };

        lookups.living_entity_class = env
            .env()
            .find_class(living_entity_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let player_class = if using_fabric {
            fabric_classes::PLAYER
        } else {
            classes::PLAYER
        };

        lookups.player_class = env
            .env()
            .find_class(player_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let component_class = if using_fabric {
            fabric_classes::COMPONENT
        } else {
            classes::COMPONENT
        };

        lookups.component_class = env
            .env()
            .find_class(component_class)
            .ok()
            .and_then(|c| env.env().new_global_ref(c).ok());

        let entity_class_name = if using_fabric {
            fabric_classes::ENTITY
        } else {
            classes::ENTITY
        };

        // --- Entity class fields & methods ---
        let entity_class_local = env.env().find_class(entity_class_name).ok();
        if let Some(ref entity_cls) = entity_class_local {
            let get_x_mapping = if using_fabric {
                &fabric_entity::GET_X
            } else {
                &entity::GET_X
            };

            lookups.get_x_method = env
                .env()
                .get_method_id(entity_cls, get_x_mapping.name, get_x_mapping.signature)
                .ok();

            let get_y_mapping = if using_fabric {
                &fabric_entity::GET_Y
            } else {
                &entity::GET_Y
            };

            lookups.get_y_method = env
                .env()
                .get_method_id(entity_cls, get_y_mapping.name, get_y_mapping.signature)
                .ok();

            let get_eye_y_mapping = if using_fabric {
                &fabric_entity::GET_EYE_Y
            } else {
                &entity::GET_EYE_Y
            };

            lookups.get_eye_y_method = env
                .env()
                .get_method_id(
                    entity_cls,
                    get_eye_y_mapping.name,
                    get_eye_y_mapping.signature,
                )
                .ok();

            let get_z_mapping = if using_fabric {
                &fabric_entity::GET_Z
            } else {
                &entity::GET_Z
            };

            lookups.get_z_method = env
                .env()
                .get_method_id(entity_cls, get_z_mapping.name, get_z_mapping.signature)
                .ok();

            let get_id_mapping = if using_fabric {
                &fabric_entity::GET_ID
            } else {
                &entity::GET_ID
            };

            lookups.get_entity_id_method = env
                .env()
                .get_method_id(entity_cls, get_id_mapping.name, get_id_mapping.signature)
                .ok();

            let get_name_mapping = if using_fabric {
                &fabric_entity::GET_NAME
            } else {
                &entity::GET_NAME
            };

            lookups.get_name_method = env
                .env()
                .get_method_id(
                    entity_cls,
                    get_name_mapping.name,
                    get_name_mapping.signature,
                )
                .ok();

            let old_x_mapping = if using_fabric {
                &fabric_entity::OLD_X
            } else {
                &entity::OLD_X
            };

            lookups.old_x_field = env
                .env()
                .get_field_id(entity_cls, old_x_mapping.name, old_x_mapping.signature)
                .ok();

            let old_y_mapping = if using_fabric {
                &fabric_entity::OLD_Y
            } else {
                &entity::OLD_Y
            };

            lookups.old_y_field = env
                .env()
                .get_field_id(entity_cls, old_y_mapping.name, old_y_mapping.signature)
                .ok();

            let old_z_mapping = if using_fabric {
                &fabric_entity::OLD_Z
            } else {
                &entity::OLD_Z
            };

            lookups.old_z_field = env
                .env()
                .get_field_id(entity_cls, old_z_mapping.name, old_z_mapping.signature)
                .ok();
        }

        // --- LivingEntity health method ---
        if let Some(ref living_cls) = lookups.living_entity_class {
            let get_health_mapping = if using_fabric {
                &fabric_player::GET_HEALTH
            } else {
                &player::GET_HEALTH
            };

            lookups.get_health_method = env
                .env()
                .get_method_id(
                    living_cls,
                    get_health_mapping.name,
                    get_health_mapping.signature,
                )
                .ok();
        }

        // --- Component getString ---
        if let Some(ref component_cls) = lookups.component_class {
            let get_string_mapping = if using_fabric {
                &fabric_component::GET_STRING
            } else {
                &component::GET_STRING
            };

            lookups.get_string_method = env
                .env()
                .get_method_id(
                    component_cls,
                    get_string_mapping.name,
                    get_string_mapping.signature,
                )
                .ok();
        }

        // --- ClientLevel players method ---
        if let Some(ref level_cls) = lookups.level_class {
            let players_mapping = if using_fabric {
                &fabric_client_level::PLAYERS
            } else {
                &client_level::PLAYERS
            };

            lookups.get_players_method = env
                .env()
                .get_method_id(level_cls, players_mapping.name, players_mapping.signature)
                .ok();
        }

        // --- List methods ---
        if let Some(ref list_cls) = lookups.list_class {
            lookups.list_size_method = env
                .env()
                .get_method_id(list_cls, java_list::SIZE.name, java_list::SIZE.signature)
                .ok();

            lookups.list_get_method = env
                .env()
                .get_method_id(list_cls, java_list::GET.name, java_list::GET.signature)
                .ok();
        }

        // --- DeltaTracker.Timer getGameTimeDeltaPartialTick ---
        if let Some(ref dt_cls) = lookups.delta_tracker_timer_class {
            let delta_mapping = if using_fabric {
                &fabric_delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
            } else {
                &delta_tracker_timer::GET_GAME_TIME_DELTA_PARTIAL_TICK
            };

            lookups.get_game_time_delta_method = env
                .env()
                .get_method_id(dt_cls, delta_mapping.name, delta_mapping.signature)
                .ok();
        }

        // --- GameRenderer methods ---
        if let Some(ref gr_cls) = lookups.game_renderer_class {
            let get_main_camera_mapping = if using_fabric {
                &fabric_game_renderer::GET_MAIN_CAMERA
            } else {
                &game_renderer::GET_MAIN_CAMERA
            };

            lookups.get_main_camera_method = env
                .env()
                .get_method_id(
                    gr_cls,
                    get_main_camera_mapping.name,
                    get_main_camera_mapping.signature,
                )
                .ok();

            let get_fov_mapping = if using_fabric {
                &fabric_game_renderer::GET_FOV
            } else {
                &game_renderer::GET_FOV
            };

            lookups.get_render_fov_method = env
                .env()
                .get_method_id(gr_cls, get_fov_mapping.name, get_fov_mapping.signature)
                .ok();
        }

        // --- Camera methods ---
        if let Some(ref cam_cls) = lookups.camera_class {
            let get_position_mapping = if using_fabric {
                &fabric_camera::GET_POSITION
            } else {
                &camera::GET_POSITION
            };

            lookups.get_camera_position_method = env
                .env()
                .get_method_id(
                    cam_cls,
                    get_position_mapping.name,
                    get_position_mapping.signature,
                )
                .ok();

            let get_yaw_mapping = if using_fabric {
                &fabric_camera::GET_Y_ROT
            } else {
                &camera::GET_Y_ROT
            };

            lookups.get_camera_yaw_method = env
                .env()
                .get_method_id(cam_cls, get_yaw_mapping.name, get_yaw_mapping.signature)
                .ok();

            let get_pitch_mapping = if using_fabric {
                &fabric_camera::GET_X_ROT
            } else {
                &camera::GET_X_ROT
            };

            lookups.get_camera_pitch_method = env
                .env()
                .get_method_id(cam_cls, get_pitch_mapping.name, get_pitch_mapping.signature)
                .ok();
        }

        // --- Vec3 fields ---
        if let Some(ref v3_cls) = lookups.vec3_class {
            let x_mapping = if using_fabric {
                &fabric_vec3::X
            } else {
                &vec3::X
            };

            lookups.vec3_x_field = env
                .env()
                .get_field_id(v3_cls, x_mapping.name, x_mapping.signature)
                .ok();

            let y_mapping = if using_fabric {
                &fabric_vec3::Y
            } else {
                &vec3::Y
            };

            lookups.vec3_y_field = env
                .env()
                .get_field_id(v3_cls, y_mapping.name, y_mapping.signature)
                .ok();

            let z_mapping = if using_fabric {
                &fabric_vec3::Z
            } else {
                &vec3::Z
            };

            lookups.vec3_z_field = env
                .env()
                .get_field_id(v3_cls, z_mapping.name, z_mapping.signature)
                .ok();
        }

        // Report any missing required lookups
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
            println!(
                "[ESP] CRITICAL: Required entity/level lookups missing for {} — ESP disabled",
                if using_fabric { "Fabric" } else { "Vanilla" }
            );
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
            println!(
                "[ESP] CRITICAL: Required render-camera lookups missing for {} — ESP disabled",
                if using_fabric { "Fabric" } else { "Vanilla" }
            );
            return None;
        }

        if lookups.get_entity_id_method.is_none() {
            println!("[ESP] WARNING: Entity.getId not found — name cache disabled");
        }
        if lookups.get_health_method.is_none() {
            println!("[ESP] WARNING: LivingEntity.getHealth not found — health display disabled");
        }

        println!(
            "[ESP] All lookups initialized successfully for {}!",
            if using_fabric { "Fabric" } else { "Vanilla" }
        );
        Some(lookups)
    }

    pub fn is_ready(&self) -> bool {
        // Required for basic entity scanning
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
            // Required for camera
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

    /// True only if we have the optional lookups needed for name caching
    pub fn can_resolve_names(&self) -> bool {
        self.get_entity_id_method.is_some()
            && self.get_name_method.is_some()
            && self.get_string_method.is_some()
    }

    /// True only if health reading is available
    pub fn can_read_health(&self) -> bool {
        self.get_health_method.is_some()
    }

    /// True if frame-time interpolation is available
    pub fn can_read_frame_time(&self) -> bool {
        self.delta_tracker_field.is_some() && self.get_game_time_delta_method.is_some()
    }
}

impl Drop for JniLookups {
    fn drop(&mut self) {
        // GlobalRefs auto-clean on drop; primitive IDs don't need cleanup
    }
}
