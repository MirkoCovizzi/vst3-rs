extern crate log;
extern crate vst3_sys;

use std::os::raw::{c_char, c_void};
use std::ptr::{copy_nonoverlapping, null_mut};

use self::vst3_sys::vst::kDefaultFactoryFlags;
use vst3_sys::base::{
    kResultFalse, kResultOk, tresult, IPluginFactory, IPluginFactory2, IPluginFactory3, PClassInfo,
    PClassInfo2, PClassInfoW, PFactoryInfo,
};
use vst3_sys::IID;
use vst3_sys::VST3;

unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub struct FactoryInfo {
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub flags: i32,
}

pub trait Factory {
    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Default::default()
    }

    fn info(&self) -> FactoryInfo {
        FactoryInfo {
            vendor: "".to_string(),
            url: "".to_string(),
            email: "".to_string(),
            flags: kDefaultFactoryFlags,
        }
    }
}

#[VST3(implements(IPluginFactory3, IPluginFactory2, IPluginFactory))]
pub struct PluginFactory {
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

    pub unsafe fn get_factory(&self) -> &Box<dyn Factory> {
        &*(self.factory as *mut Box<dyn Factory>)
    }

    pub unsafe fn get_factory_mut(&mut self) -> &mut Box<dyn Factory> {
        &mut *(self.factory as *mut Box<dyn Factory>)
    }
}

impl IPluginFactory3 for PluginFactory {
    unsafe fn get_class_info_unicode(&self, _idx: i32, _info: *mut PClassInfoW) -> tresult {
        kResultFalse
    }

    unsafe fn set_host_context(&self, _context: *mut c_void) -> tresult {
        kResultFalse
    }
}

impl IPluginFactory2 for PluginFactory {
    unsafe fn get_class_info2(&self, _index: i32, _info: *mut PClassInfo2) -> tresult {
        kResultFalse
    }
}

impl IPluginFactory for PluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let factory_info = &self.get_factory().info();

        let len_src = factory_info.vendor.len();
        let len_dest = (*info).vendor.len();
        if len_src > len_dest {
            log::error!(
                "PluginFactory's `vendor` field is too long! {} > {}",
                len_src,
                len_dest
            );
            return kResultFalse;
        }

        let len_src = factory_info.url.len();
        let len_dest = (*info).url.len();
        if len_src > len_dest {
            log::error!(
                "PluginFactory's `url` field is too long! {} > {}",
                len_src,
                len_dest
            );
            return kResultFalse;
        }

        let len_src = factory_info.email.len();
        let len_dest = (*info).email.len();
        if len_src > len_dest {
            log::error!(
                "PluginFactory's `email` field is too long! {} > {}",
                len_src,
                len_dest
            );
            return kResultFalse;
        }

        let info = &mut *info;
        strcpy(&factory_info.vendor, info.vendor.as_mut_ptr());
        strcpy(&factory_info.url, info.url.as_mut_ptr());
        strcpy(&factory_info.email, info.email.as_mut_ptr());
        info.flags = factory_info.flags;

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        0
    }

    unsafe fn get_class_info(&self, _index: i32, _info: *mut PClassInfo) -> tresult {
        kResultFalse
    }

    unsafe fn create_instance(
        &self,
        _cid: *mut IID,
        _iid: *mut IID,
        _obj: *mut *mut c_void,
    ) -> tresult {
        kResultFalse
    }
}
