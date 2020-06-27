mod audio_processor;
mod basic_plugin;
mod bus;
mod common;
mod component;
mod edit_controller;
mod events;
mod factory;
mod host_application;
mod logging;
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
pub use logging::*;
pub use parameter_changes::*;
pub use parameters::*;
pub use plug_view::*;
pub use plugin_base::*;
pub use speaker::*;
pub use stream::*;
pub use unit::*;
pub use unit_info::*;
pub use unknown::*;

use std::os::raw::c_void;

#[macro_export]
macro_rules! plugin_main {
    (
        vendor: $vendor:expr,
        url: $url:expr,
        email: $email:expr,
        classes: [$($class:ident), +]
    ) => {
        struct DefaultFactory {
            context: std::option::Option<$crate::HostApplication>,
            classes: std::vec::Vec<($crate::ClassInfo, fn() -> std::boxed::Box<dyn $crate::PluginBase>)>,
        }

        impl DefaultFactory {
            const INFO: FactoryInfo = FactoryInfo {
                vendor: $vendor,
                url: $url,
                email: $email,
                flags: vst3_sys::vst::kDefaultFactoryFlags,
            };
        }

        impl std::default::Default for DefaultFactory {
            fn default() -> Self {
                Self {
                    context: None,
                    classes: std::vec![$(($class::INFO, $class::new)), *],
                }
            }
        }

        impl $crate::PluginFactory for DefaultFactory {
            fn get_factory_info(&self) -> std::result::Result<&$crate::FactoryInfo, $crate::ResultErr> {
                std::result::Result::Ok(&Self::INFO)
            }

            fn count_classes(&self) -> std::result::Result<usize, $crate::ResultErr> {
                std::result::Result::Ok(self.classes.len())
            }

            fn get_class_info(&self, index: usize) -> std::result::Result<&$crate::ClassInfo, $crate::ResultErr> {
                if index as usize >= self.classes.len() {
                    return std::result::Result::Err($crate::ResultErr::InvalidArgument);
                }

                std::result::Result::Ok(&self.classes[index as usize].0)
            }

            fn create_instance(&self, cid: &$crate::UID) -> std::result::Result<std::boxed::Box<dyn $crate::PluginBase>, $crate::ResultErr> {
                for c in &self.classes {
                    if *cid == *c.0.get_cid() {
                        return std::result::Result::Ok(c.1());
                    }
                }
                std::result::Result::Err($crate::ResultErr::ResultFalse)
            }

            fn set_host_context(&mut self, context: $crate::HostApplication) -> std::result::Result<$crate::ResultOk, $crate::ResultErr> {
                if self.context.is_some() {
                    return std::result::Result::Err($crate::ResultErr::ResultFalse);
                }

                self.context = std::option::Option::Some(context);

                std::result::Result::Ok($crate::ResultOk::ResOk)
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

pub fn create_factory<T: 'static + PluginFactory + Default>() -> *mut c_void {
    let mut factory = VST3PluginFactory::new();
    factory.set_factory(T::new());
    Box::into_raw(factory) as *mut c_void
}
