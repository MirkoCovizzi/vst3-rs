use std::os::raw::{c_char, c_short, c_void};

pub mod buffer;
pub mod component;
pub mod events;
pub mod factory;
pub mod io;
pub mod parameter;
pub mod parameters;
pub mod types;
pub mod util;

pub use buffer::*;
pub use component::*;
pub use events::*;
pub use factory::*;
pub use io::*;
pub use parameter::*;
pub use parameters::*;
pub use types::*;
pub use util::*;

use vst3_com::sys::*;
use vst3_sys::base::*;
use vst3_sys::vst::*;

use std::ffi::CStr;
use std::ptr::copy_nonoverlapping;
use vst3_sys::base::ClassCardinality::kManyInstances;
use vst3_sys::base::FactoryFlags::*;
use vst3_sys::vst::BusFlags::kDefaultActive;
use vst3_sys::vst::BusTypes::{kAux, kMain};
use vst3_sys::vst::ParameterFlags::kCanAutomate;
use widestring::U16CString;

pub enum PluginSpeaker {
    L,
    R,
}

/// todo: replace with the correct ones from vst3-sys!!!
pub enum PluginSpeakerArrangement {
    Empty = 0,
    Mono = 524288,
    Stereo = kStereo as isize,
}

pub enum PluginBusType {
    Main = kMain as isize,
    Aux = kAux as isize,
}

pub enum PluginBusFlag {
    DefaultActive = kDefaultActive as isize,
}

pub enum PluginBusDirection {
    Input,
    Output,
}

#[derive(Clone)]
pub enum PluginClassCardinality {
    ManyInstances = kManyInstances as isize,
}

/// If the source &str is too long, it gets truncated to fit into the destination
unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

/// If the source &str is too long, it gets truncated to fit into the destination
unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

#[macro_export]
macro_rules! plugin_main {
    (
        vendor: $vendor:expr,
        url: $url:expr,
        email: $email:expr,
        plugins: [$($plugin:ident), +]
    ) => {
        struct DefaultFactory {
            plugins: std::vec::Vec<std::boxed::Box<dyn $crate::Plugin>>,
        }

        impl std::default::Default for DefaultFactory {
            fn default() -> Self {
                DefaultFactory {
                    plugins: std::vec![$($plugin::new()), *]
                }
            }
        }

        impl $crate::Factory for DefaultFactory {
            fn get_info(&self) -> $crate::FactoryInfo {
                $crate::FactoryInfo {
                    vendor: $vendor.to_string(),
                    url: $url.to_string(),
                    email: $email.to_string(),
                    flags: vst3_sys::vst::kDefaultFactoryFlags,
                }
            }

            fn get_plugins(&self) -> &std::vec::Vec<std::boxed::Box<dyn $crate::Plugin>> {
                &self.plugins
            }
        }

        $crate::factory_main!(DefaultFactory);
    };
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
        pub extern "system" fn ModuleEntry(_: *mut std::os::raw::c_void) -> bool {
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
        pub extern "system" fn bundleEntry(_: *mut std::os::raw::c_void) -> bool {
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
