extern crate log;
extern crate vst3_sys;

use std::os::raw::c_void;
use std::ptr::null_mut;

use self::vst3_sys::vst::kDefaultFactoryFlags;
use vst3_sys::base::{
    kResultFalse, kResultOk, tresult, IPluginFactory, IPluginFactory2, IPluginFactory3, PClassInfo,
    PClassInfo2, PClassInfoW, PFactoryInfo,
};
use vst3_sys::IID;
use vst3_sys::VST3;

use self::vst3_sys::base::kNotImplemented;
use crate::{guid, strcpy, wstrcpy, Plugin, PluginComponent};

pub struct FactoryInfo {
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub flags: i32,
}

pub trait Factory {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn info(&self) -> FactoryInfo {
        FactoryInfo {
            vendor: "".to_string(),
            url: "".to_string(),
            email: "".to_string(),
            flags: kDefaultFactoryFlags,
        }
    }

    fn get_plugins(&self) -> &Vec<Box<dyn Plugin>>;
}

#[VST3(implements(IPluginFactory, IPluginFactory2, IPluginFactory3))]
pub(crate) struct PluginFactory {
    factory: *mut c_void,
}
impl PluginFactory {
    pub(crate) fn new() -> Box<Self> {
        let factory = null_mut();
        Self::allocate(factory)
    }

    pub(crate) fn set_factory(&mut self, factory: *mut c_void) {
        self.factory = factory;
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_factory(&self) -> &Box<dyn Factory> {
        &*(self.factory as *mut Box<dyn Factory>)
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_factory_mut(&mut self) -> &mut Box<dyn Factory> {
        &mut *(self.factory as *mut Box<dyn Factory>)
    }
}

impl IPluginFactory3 for PluginFactory {
    unsafe fn get_class_info_unicode(&self, index: i32, info: *mut PClassInfoW) -> tresult {
        if index as usize >= self.get_factory().get_plugins().len() {
            return kResultFalse;
        }

        let plugins = self.get_factory().get_plugins();
        let plugin = plugins.get(index as usize).unwrap();
        let plugin_info = plugin.info();

        let info = &mut *info;
        info.cid = guid(plugin_info.cid);
        info.cardinality = plugin_info.cardinality;
        strcpy(&plugin_info.category, info.category.as_mut_ptr());
        wstrcpy(&plugin_info.name, info.name.as_mut_ptr());
        info.class_flags = plugin_info.class_flags;
        strcpy(&plugin_info.subcategories, info.subcategories.as_mut_ptr());
        wstrcpy(&plugin_info.vendor, info.vendor.as_mut_ptr());
        wstrcpy(&plugin_info.version, info.version.as_mut_ptr());
        wstrcpy(&plugin_info.sdk_version, info.sdk_version.as_mut_ptr());

        kResultOk
    }

    unsafe fn set_host_context(&self, _context: *mut c_void) -> tresult {
        kNotImplemented
    }
}

impl IPluginFactory2 for PluginFactory {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        if index as usize >= self.get_factory().get_plugins().len() {
            return kResultFalse;
        }

        let plugins = self.get_factory().get_plugins();
        let plugin = plugins.get(index as usize).unwrap();
        let plugin_info = plugin.info();

        let info = &mut *info;
        info.cid = guid(plugin_info.cid);
        info.cardinality = plugin_info.cardinality;
        strcpy(&plugin_info.category, info.category.as_mut_ptr());
        strcpy(&plugin_info.name, info.name.as_mut_ptr());
        info.class_flags = plugin_info.class_flags;
        strcpy(&plugin_info.subcategories, info.subcategories.as_mut_ptr());
        strcpy(&plugin_info.vendor, info.vendor.as_mut_ptr());
        strcpy(&plugin_info.version, info.version.as_mut_ptr());
        strcpy(&plugin_info.sdk_version, info.sdk_version.as_mut_ptr());

        kResultOk
    }
}

impl IPluginFactory for PluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let factory_info = self.get_factory().info();

        let info = &mut *info;
        strcpy(&factory_info.vendor, info.vendor.as_mut_ptr());
        strcpy(&factory_info.url, info.url.as_mut_ptr());
        strcpy(&factory_info.email, info.email.as_mut_ptr());
        info.flags = factory_info.flags;

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        self.get_factory().get_plugins().len() as i32
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult {
        if index as usize >= self.get_factory().get_plugins().len() {
            return kResultFalse;
        }

        let plugins = self.get_factory().get_plugins();
        let plugin = plugins.get(index as usize).unwrap();
        let plugin_info = plugin.info();

        let info = &mut *info;
        info.cid = guid(plugin_info.cid);
        info.cardinality = plugin_info.cardinality;
        strcpy(&plugin_info.category, info.category.as_mut_ptr());
        strcpy(&plugin_info.name, info.name.as_mut_ptr());

        kResultOk
    }

    unsafe fn create_instance(
        &self,
        cid: *mut IID,
        _iid: *mut IID,
        obj: *mut *mut c_void,
    ) -> tresult {
        let plugins = self.get_factory().get_plugins();
        for p in plugins {
            if *cid == guid(p.info().cid) {
                let p = Box::into_raw(Box::new(p)) as *mut _;
                let mut component = PluginComponent::new();
                component.set_component(p);
                *obj = Box::into_raw(component) as *mut c_void;
                return kResultOk;
            }
        }
        kResultFalse
    }
}
