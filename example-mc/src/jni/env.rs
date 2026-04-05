// jni_env.rs
use jni::sys::{JNIEnv as JNIEnvRaw, JavaVM, JNI_OK, JNI_VERSION_1_8};
use jni::JNIEnv;
use once_cell::sync::OnceCell;
use std::ffi::CString;
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

#[repr(transparent)]
#[derive(Clone, Copy)]
struct VmPtr(*mut JavaVM);
unsafe impl Send for VmPtr {}
unsafe impl Sync for VmPtr {}

impl VmPtr {
    fn new(ptr: *mut JavaVM) -> Self {
        Self(ptr)
    }
    fn get(&self) -> *mut JavaVM {
        self.0
    }
}

static ACTIVE_VM: OnceCell<VmPtr> = OnceCell::new();

/// Returns the raw JavaVM pointer as *mut c_void, or None if the VM hasn't been discovered yet.
/// Used by class_cache to obtain a JVMTI environment.
pub fn get_raw_vm() -> Option<*mut std::ffi::c_void> {
    ACTIVE_VM.get().map(|p| p.get() as *mut std::ffi::c_void)
}

pub struct JniEnvironment {
    env: JNIEnv<'static>,
    attached: bool,
}

impl JniEnvironment {
    pub fn attach(thread_name: &str) -> Option<Self> {
        unsafe {
            let h_jvm = GetModuleHandleW(windows::core::w!("jvm.dll")).ok()?;
            let proc_addr = GetProcAddress(h_jvm, PCSTR("JNI_GetCreatedJavaVMs\0".as_ptr()))?;

            type GetCreatedJavaVMsFn = unsafe extern "system" fn(
                *mut *mut JavaVM,
                jni::sys::jsize,
                *mut jni::sys::jsize,
            ) -> jni::sys::jint;

            let get_created_vms: GetCreatedJavaVMsFn = std::mem::transmute(proc_addr);

            let mut vm: *mut JavaVM = std::ptr::null_mut();
            let mut count: jni::sys::jsize = 0;

            if get_created_vms(&mut vm, 1, &mut count) != JNI_OK || vm.is_null() {
                return None;
            }

            // First successful attach stores the VM; later attach() calls (e.g. after Minecraft::init)
            // must not fail because OnceCell is already occupied.
            let _ = ACTIVE_VM.set(VmPtr::new(vm));

            let name = CString::new(thread_name).ok()?;
            let mut env_ptr: *mut JNIEnvRaw = std::ptr::null_mut();

            let attach_args = jni::sys::JavaVMAttachArgs {
                version: JNI_VERSION_1_8,
                name: name.as_ptr() as *mut _,
                group: std::ptr::null_mut(),
            };

            let vm_raw = ACTIVE_VM.get()?.get();
            let iface = *(*vm_raw);
            let attach_fn = iface.AttachCurrentThread?;

            // Cast to *mut *mut c_void as required by JNI spec
            let result = attach_fn(
                vm_raw,
                &mut env_ptr as *mut *mut JNIEnvRaw as *mut *mut _,
                &attach_args as *const _ as *mut _,
            );

            if result != JNI_OK {
                return None;
            }

            let env = JNIEnv::from_raw(env_ptr).ok()?;
            Some(Self {
                env,
                attached: true,
            })
        }
    }

    pub fn get_current() -> Option<JNIEnv<'static>> {
        unsafe {
            let vm_ptr = ACTIVE_VM.get()?.get();
            let iface = *(*vm_ptr);
            let get_env_fn = iface.GetEnv?;

            let mut env_ptr: *mut JNIEnvRaw = std::ptr::null_mut();
            let result = get_env_fn(
                vm_ptr,
                &mut env_ptr as *mut *mut JNIEnvRaw as *mut *mut _,
                JNI_VERSION_1_8,
            );

            if result == JNI_OK && !env_ptr.is_null() {
                JNIEnv::from_raw(env_ptr).ok()
            } else {
                None
            }
        }
    }

    /// Like [`get_current`], but attaches the **current OS thread** to the JVM if needed.
    /// Call this from native callbacks that are not the thread that ran `attach` (e.g. `wglSwapBuffers`).
    pub fn get_current_or_attach(thread_name: &str) -> Option<JNIEnv<'static>> {
        if let Some(env) = Self::get_current() {
            return Some(env);
        }

        unsafe {
            let vm_raw = ACTIVE_VM.get()?.get();
            let iface = *(*vm_raw);
            let attach_fn = iface.AttachCurrentThread?;

            let name = CString::new(thread_name).ok()?;
            let attach_args = jni::sys::JavaVMAttachArgs {
                version: JNI_VERSION_1_8,
                name: name.as_ptr() as *mut _,
                group: std::ptr::null_mut(),
            };

            let mut env_ptr: *mut JNIEnvRaw = std::ptr::null_mut();
            let result = attach_fn(
                vm_raw,
                &mut env_ptr as *mut *mut JNIEnvRaw as *mut *mut _,
                &attach_args as *const _ as *mut _,
            );

            if result != JNI_OK || env_ptr.is_null() {
                return None;
            }

            JNIEnv::from_raw(env_ptr).ok()
        }
    }

    pub fn detach(&mut self) {
        if self.attached {
            unsafe {
                if let Some(vm_ptr) = ACTIVE_VM.get() {
                    let vm_raw = vm_ptr.get();
                    let iface = *(*vm_raw);
                    if let Some(detach_fn) = iface.DetachCurrentThread {
                        let _ = detach_fn(vm_raw);
                    }
                }
            }
            self.attached = false;
        }
    }

    pub fn env(&mut self) -> &mut JNIEnv<'static> {
        &mut self.env
    }
}

impl Drop for JniEnvironment {
    fn drop(&mut self) {
        self.detach();
    }
}
