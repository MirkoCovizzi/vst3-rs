use std::os::raw::{c_char, c_short, c_void};

pub mod component;
pub mod factory;

pub use component::*;
pub use factory::*;

pub use vst3_com::sys::*;
pub use vst3_sys::base::*;
pub use vst3_sys::vst::*;

use std::ptr::copy_nonoverlapping;
use widestring::U16CString;

pub type UID = [u32; 4];

pub(crate) unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub(crate) unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub fn guid(uid: UID) -> GUID {
    let mut tuid: [u8; 16] = [0; 16];
    for i in 0..4 {
        let big_e = uid[i].to_be_bytes();
        for k in 0..4 {
            tuid[i * 4 + k] = unsafe { std::mem::transmute(big_e[k]) };
        }
    }
    GUID { data: tuid }
}

#[macro_export]
macro_rules! factory_main {
    ($t:ty) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn InitDll() -> bool {
            true
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn ExitDll() -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn ModuleEntry(_: *mut c_void) -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn ModuleExit() -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleEntry(_: *mut c_void) -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleExit() -> bool {
            true
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn GetPluginFactory() -> *mut std::os::raw::c_void {
            $crate::create_factory::<$t>()
        }
    };
}

pub fn create_factory<T: Factory + Default>() -> *mut c_void {
    let f = Box::into_raw(Box::new(T::new() as Box<dyn Factory>)) as *mut _;
    let mut factory = PluginFactory::new();
    factory.set_factory(f);
    Box::into_raw(factory) as *mut c_void
}
