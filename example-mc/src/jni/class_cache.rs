//! JVMTI-based class cache.
//!
//! The problem with Fabric (and other modloaders): JNI's `FindClass` only searches
//! classloaders reachable from the calling native thread — the system/bootstrap
//! classloader. Fabric loads all Minecraft classes through its own "Knot"
//! classloader, which `FindClass` cannot see.
//!
//! This module calls JVMTI's `GetLoadedClasses` instead, which returns *every*
//! class the JVM has loaded, regardless of which classloader loaded it.
//! The result is stored in a static `HashMap<String, GlobalRef>` and reused for
//! all subsequent lookups.
//!
//! This is the direct Rust equivalent of the C++ `Lunar::GetLoadedClasses` /
//! `Lunar::GetClass` approach.

use std::collections::HashMap;
use std::sync::Arc;

use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::JNIEnv;
use once_cell::sync::OnceCell;

use jni_simple::{JVMTIEnv, JVMTI_VERSION_1_2};

// ---------------------------------------------------------------------------
// Static cache
// ---------------------------------------------------------------------------

/// A cheap-to-clone, thread-safe class reference.
pub type SharedClassRef = Arc<GlobalRef>;

static CLASS_CACHE: OnceCell<HashMap<String, SharedClassRef>> = OnceCell::new();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns `true` if the cache has already been built.
pub fn is_ready() -> bool {
    CLASS_CACHE.get().is_some()
}

/// Build the JVMTI class cache.
///
/// Attaches the current OS thread to the JVM, calls `GetLoadedClasses` via
/// JVMTI, and stores every class under its JNI slash-separated name
/// (e.g. `"net/minecraft/class_310"`).
///
/// This must be called once at startup **before** any `find_class` lookups.
/// Returns `true` on success.
pub fn init() -> bool {
    if CLASS_CACHE.get().is_some() {
        return true;
    }

    let mut env_wrapper = match crate::jni::env::JniEnvironment::attach("ClassCache-JVMTI") {
        Some(e) => e,
        None => {
            crate::esp_log("[ClassCache] Failed to attach JNI environment\n");
            return false;
        }
    };

    build(env_wrapper.env())
}

/// Look up a class by its JNI slash-separated name and return a fresh
/// `JClass` that is valid for the current call frame.
///
/// Accepts both dot-separated (`"net.minecraft.class_310"`) and
/// slash-separated (`"net/minecraft/class_310"`) names.
///
/// Returns `None` if the cache has not been built or the class is absent.
///
/// # Safety
/// The returned `JClass` borrows from a static `GlobalRef`, which lives
/// as long as the JVM.  Treat it as you would any other `JClass` obtained
/// from `FindClass`.
pub fn find_class<'env>(name: &str) -> Option<JClass<'env>> {
    let key = to_slash(name);
    let arc = CLASS_CACHE.get()?.get(&key)?;

    // Re-interpret the global ref's raw pointer as a local JClass.
    // Safe because:
    //  • Global refs are valid on any attached thread for the life of the JVM.
    //  • The GlobalRef (inside the static OnceCell) is never freed.
    //  • JNI accepts global-ref pointers wherever jclass / jobject is expected.
    let raw = arc.as_obj().as_raw();
    let obj = unsafe { JObject::from_raw(raw) };
    Some(unsafe { std::mem::transmute::<JObject<'env>, JClass<'env>>(obj) })
}

/// Returns `true` when the cache contains the Fabric intermediary Minecraft
/// class (`net/minecraft/class_310`), i.e. we are running under Fabric.
pub fn is_fabric() -> bool {
    CLASS_CACHE
        .get()
        .map_or(false, |c| c.contains_key("net/minecraft/class_310"))
}

/// Returns `true` when the cache contains the vanilla / ProGuard Minecraft
/// class (`gfj`), i.e. we are running on vanilla.
pub fn is_vanilla() -> bool {
    CLASS_CACHE.get().map_or(false, |c| c.contains_key("gfj"))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Convert dots to slashes (normalise to JNI internal name format).
fn to_slash(name: &str) -> String {
    name.replace('.', "/")
}

fn build(env: &mut JNIEnv<'static>) -> bool {
    // ---- 1. Get the raw JavaVM pointer stored by env.rs ------------------
    let vm_raw_cv = match crate::jni::env::get_raw_vm() {
        Some(p) => p,
        None => {
            crate::esp_log("[ClassCache] No JavaVM pointer available\n");
            return false;
        }
    };

    // ---- 2. Obtain a JVMTI environment -----------------------------------
    //
    // We need to call JavaVM::GetEnv with the JVMTI version.  The raw
    // vm_raw_cv pointer is `*mut c_void` that represents `JavaVM*` in C.
    // jni::sys::JavaVM_ has the same vtable layout, so we can use the
    // jni-crate's sys types to reach GetEnv.
    //
    // Once we have the raw jvmtiEnv* we wrap it with jni-simple's JVMTIEnv
    // which gives us a clean, type-safe JVMTI API.
    let jvmti: JVMTIEnv = unsafe {
        let vm_sys = vm_raw_cv as *mut jni::sys::JavaVM;
        let iface = *(*vm_sys);
        let get_env = match iface.GetEnv {
            Some(f) => f,
            None => {
                crate::esp_log("[ClassCache] JavaVM::GetEnv is null\n");
                return false;
            }
        };

        let mut jvmti_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let rc = get_env(
            vm_sys,
            &mut jvmti_ptr as *mut *mut std::ffi::c_void as *mut *mut _,
            JVMTI_VERSION_1_2,
        );

        if rc != 0 || jvmti_ptr.is_null() {
            crate::esp_log(&format!(
                "[ClassCache] GetEnv(JVMTI_VERSION_1_2) failed: {rc}\n"
            ));
            return false;
        }

        // Construct jni-simple JVMTIEnv from the raw pointer — this is safe
        // because the pointer is a valid jvmtiEnv* returned by the JVM.
        JVMTIEnv::from(jvmti_ptr)
    };

    // ---- 3. Enumerate every loaded class ---------------------------------
    let mut class_count: jni_simple::jint = 0;
    let mut classes_raw: *mut jni_simple::jclass = std::ptr::null_mut();

    let err = unsafe { jvmti.GetLoadedClasses(&mut class_count, &mut classes_raw) };
    if err != jni_simple::JVMTI_ERROR_NONE || classes_raw.is_null() {
        crate::esp_log(&format!("[ClassCache] GetLoadedClasses failed: {err:?}\n"));
        return false;
    }

    crate::esp_log(&format!(
        "[ClassCache] JVMTI reports {class_count} loaded classes\n"
    ));

    // ---- 4. Resolve Class.getName() once ---------------------------------
    let class_of_class = match env.find_class("java/lang/Class") {
        Ok(c) => c,
        Err(_) => {
            crate::esp_log("[ClassCache] Cannot find java/lang/Class\n");
            unsafe { jvmti.Deallocate(classes_raw) };
            return false;
        }
    };

    let get_name_mid = match env.get_method_id(&class_of_class, "getName", "()Ljava/lang/String;") {
        Ok(m) => m,
        Err(_) => {
            crate::esp_log("[ClassCache] Cannot resolve Class.getName()\n");
            unsafe { jvmti.Deallocate(classes_raw) };
            return false;
        }
    };

    // ---- 5. Build the HashMap -------------------------------------------
    let mut cache: HashMap<String, SharedClassRef> = HashMap::with_capacity(class_count as usize);

    for i in 0..class_count as usize {
        let raw_cls: jni_simple::jclass = unsafe { *classes_raw.add(i) };

        // Cast jni_simple::jclass (*mut _jobject) → jni::sys::jobject (*mut _jobject).
        // Both are the same C type; the cast goes through *mut c_void to avoid
        // any Rust-level type mismatch between different crate definitions.
        let jni_cls_raw = raw_cls as *mut std::ffi::c_void as jni::sys::jobject;
        let cls_obj = unsafe { JObject::from_raw(jni_cls_raw) };

        // Call cls.getName() → "net.minecraft.class_310" (dot-separated)
        let name_result = unsafe {
            env.call_method_unchecked(
                &cls_obj,
                get_name_mid,
                jni::signature::ReturnType::Object,
                &[],
            )
        };

        if let Ok(jval) = name_result {
            if let Ok(name_obj) = jval.l() {
                // Scope the JString borrow so `cow` (and its lifetime tied to
                // `name_jstr`) is fully consumed into an owned String before
                // `name_jstr` is dropped at the end of this inner block.
                let dot_name_opt: Option<String> = {
                    let name_jstr = JString::from(name_obj);
                    env.get_string(&name_jstr)
                        .map(|cow| cow.to_string_lossy().into_owned())
                        .ok()
                    // name_jstr (and cow's borrow) dropped here
                };
                if let Some(dot_name) = dot_name_opt {
                    let slash_name = to_slash(&dot_name);
                    if let Ok(global) = env.new_global_ref(&cls_obj) {
                        cache.insert(slash_name, Arc::new(global));
                    }
                }
            }
        }

        // Clear any exception before continuing to the next class.
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }

        // Release the local ref immediately to keep the local-ref table from
        // overflowing while iterating over potentially tens of thousands of classes.
        let _ = env.delete_local_ref(cls_obj);
    }

    // ---- 6. Free the JVMTI-allocated array ------------------------------
    unsafe { jvmti.Deallocate(classes_raw) };

    crate::esp_log(&format!(
        "[ClassCache] Cached {} classes  (Fabric={}, Vanilla={})\n",
        cache.len(),
        cache.contains_key("net/minecraft/class_310"),
        cache.contains_key("gfj"),
    ));

    let _ = CLASS_CACHE.set(cache);
    true
}
