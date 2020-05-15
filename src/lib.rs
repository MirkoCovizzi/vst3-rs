mod audio_processor;
mod basic_plugin;
mod bus;
mod common;
mod component;
mod edit_controller;
mod events;
mod factory;
mod host_application;
mod parameter_changes;
mod parameters;
mod plug_view;
mod plugin_base;
mod speaker;
mod stream;
mod unit;
mod unit_info;
mod unknown;

pub use audio_processor::*;
pub use basic_plugin::*;
pub use bus::*;
pub use common::*;
pub use component::*;
pub use edit_controller::*;
pub use events::*;
pub use factory::*;
pub use host_application::*;
pub use parameter_changes::*;
pub use parameters::*;
pub use plug_view::*;
pub use plugin_base::*;
pub use speaker::*;
pub use stream::*;
pub use unit::*;
pub use unit_info::*;
pub use unknown::*;

use std::os::raw::{c_char, c_short, c_void};
use std::ptr::copy_nonoverlapping;

use widestring::U16CString;

/// If the source &str is too long, it gets truncated to fit into the destination
unsafe fn strcpy(src: &str, dst: *mut c_char) {
    let mut src = src.to_string().into_bytes();
    src.push(0);
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
macro_rules! basic_plugin {
    (
        name: $name:expr,
        vendor: $vendor:expr,
        plugins: [$($plugin:ident), +]
    ) => {
        struct DefaultFactory {
            controllers: std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::PluginBase>>>,
            components: std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::PluginBase>>>,
        }

        impl std::default::Default for DefaultFactory {
            fn default() -> Self {
                DefaultFactory {
                    controllers: std::vec![$(std::sync::Mutex::new($crate::BasicPluginEditController::new($plugin::new(), $plugin::UID, $name, $vendor))), *],
                    components: std::vec![$(std::sync::Mutex::new($crate::BasicPluginComponent::new($plugin::new(), $plugin::UID, $name, $vendor))), *]
                }
            }
        }

        impl $crate::Factory for DefaultFactory {
            fn get_info(&self) -> $crate::FactoryInfo {
                $crate::FactoryInfo {
                    vendor: $vendor.to_string(),
                    url: "".to_string(),
                    email: "".to_string(),
                    flags: vst3_sys::vst::kDefaultFactoryFlags,
                }
            }

            fn get_edit_controllers(&self) -> &std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::PluginBase>>> {
                &self.controllers
            }

            fn get_components(&self) -> &std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::PluginBase>>> {
                &self.components
            }
        }

        $crate::factory_main!(DefaultFactory);
    };
}

#[macro_export]
macro_rules! plugin_main {
    (
        vendor: $vendor:expr,
        url: $url:expr,
        email: $email:expr,
        edit_controllers: [$($edit_controller:ident), +],
        components: [$($component:ident), +]
    ) => {
        struct DefaultFactory {
            edit_controllers: std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::EditController>>>,
            components: std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::Component>>>,
        }

        impl std::default::Default for DefaultFactory {
            fn default() -> Self {
                DefaultFactory {
                    edit_controllers: std::vec![$(std::sync::Mutex::new($edit_controller::new())), *],
                    components: std::vec![$(std::sync::Mutex::new($component::new())), *],
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

            fn get_edit_controllers(&self) -> &std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::EditController>>> {
                &self.edit_controllers
            }

            fn get_components(&self) -> &std::vec::Vec<std::sync::Mutex<std::boxed::Box<dyn $crate::Component>>> {
                &self.components
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
    let mut factory = VST3PluginFactory::new();
    factory.set_factory(f);
    Box::into_raw(factory) as *mut c_void
}
