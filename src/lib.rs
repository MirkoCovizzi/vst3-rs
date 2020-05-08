extern crate log;

use std::os::raw::c_void;

pub mod factory;
pub mod plugin;

pub use factory::*;
pub use plugin::*;

pub use vst3_com::sys::*;
pub use vst3_sys::base::*;
pub use vst3_sys::vst::*;

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
    let f = T::new();
    let f = Box::into_raw(Box::new(f as Box<dyn Factory>)) as *mut _;
    let mut factory = PluginFactory::new();
    factory.set_factory(f);
    Box::into_raw(factory) as *mut c_void
}
