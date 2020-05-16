use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::IID;
use vst3_sys::base::{
    IPluginFactory, IPluginFactory2, IPluginFactory3, PClassInfo, PClassInfo2, PClassInfoW,
    PFactoryInfo,
};
use vst3_sys::vst::kDefaultFactoryFlags;
use vst3_sys::VST3;

use crate::ResultErr::{InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    strcpy, wstrcpy, AudioProcessor, ClassInfo, Component, EditController, HostApplication,
    PluginBase, ResultErr, ResultOk, Unknown, VST3Component, VST3EditController, UID,
};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct FactoryInfo {
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
    pub flags: i32,
}

impl FactoryInfo {
    pub fn get_info(&self) -> vst3_sys::base::PFactoryInfo {
        let mut info = vst3_sys::base::PFactoryInfo {
            vendor: [0; 64],
            url: [0; 256],
            email: [0; 128],
            flags: self.flags,
        };

        unsafe {
            strcpy(self.vendor, info.vendor.as_mut_ptr());
            strcpy(self.url, info.url.as_mut_ptr());
            strcpy(self.email, info.email.as_mut_ptr());
        }

        info
    }
}

pub trait PluginFactory {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn get_factory_info(&self) -> Result<&FactoryInfo, ResultErr>;
    fn count_classes(&self) -> Result<u32, ResultErr>;
    fn get_class_info(&self, index: u32) -> Result<&ClassInfo, ResultErr>;
    // todo: change to &UID
    fn create_instance(&self, cid: *const IID) -> Result<Box<dyn PluginBase>, ResultErr>;
    fn set_host_context(&mut self, context: HostApplication) -> Result<ResultOk, ResultErr>;
}

struct DummyFactory {}

impl Default for DummyFactory {
    fn default() -> Self {
        Self {}
    }
}

impl PluginFactory for DummyFactory {
    fn get_factory_info(&self) -> Result<&FactoryInfo, ResultErr> {
        unimplemented!()
    }

    fn count_classes(&self) -> Result<u32, ResultErr> {
        unimplemented!()
    }

    fn get_class_info(&self, _index: u32) -> Result<&ClassInfo, ResultErr> {
        unimplemented!()
    }

    fn create_instance(&self, _cid: *const IID) -> Result<Box<dyn PluginBase>, ResultErr> {
        unimplemented!()
    }

    fn set_host_context(&mut self, _context: HostApplication) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }
}

#[VST3(implements(IPluginFactory, IPluginFactory2, IPluginFactory3))]
pub(crate) struct VST3PluginFactory {
    inner: Mutex<Box<dyn PluginFactory>>,
}

impl VST3PluginFactory {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(Mutex::new(DummyFactory::new()))
    }

    pub(crate) fn set_factory(&mut self, factory: Box<dyn PluginFactory>) {
        self.inner = Mutex::new(factory);
    }
}

impl IPluginFactory3 for VST3PluginFactory {
    unsafe fn get_class_info_unicode(&self, index: i32, info: *mut PClassInfoW) -> i32 {
        match self.inner.lock().unwrap().get_class_info(index as u32) {
            Ok(class_info) => {
                *info = class_info.get_info_w();
                ResOk.into()
            }
            Err(r) => r.into(),
        }
    }

    unsafe fn set_host_context(&self, context: *mut c_void) -> i32 {
        if let Some(context) = HostApplication::from_raw(context) {
            return match self.inner.lock().unwrap().set_host_context(*context) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        InvalidArgument.into()
    }
}

impl IPluginFactory2 for VST3PluginFactory {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> i32 {
        match self.inner.lock().unwrap().get_class_info(index as u32) {
            Ok(class_info) => {
                *info = class_info.get_info_2();
                ResOk.into()
            }
            Err(r) => r.into(),
        }
    }
}

impl IPluginFactory for VST3PluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> i32 {
        match self.inner.lock().unwrap().get_factory_info() {
            Ok(factory_info) => {
                *info = factory_info.get_info();
                ResOk.into()
            }
            Err(r) => r.into(),
        }
    }

    unsafe fn count_classes(&self) -> i32 {
        match self.inner.lock().unwrap().count_classes() {
            Ok(count) => count as i32,
            Err(r) => 0,
        }
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> i32 {
        match self.inner.lock().unwrap().get_class_info(index as u32) {
            Ok(class_info) => {
                *info = class_info.get_info();
                ResOk.into()
            }
            Err(r) => r.into(),
        }
    }

    unsafe fn create_instance(
        &self,
        cid: *const IID,
        _iid: *const IID,
        obj: *mut *mut c_void,
    ) -> i32 {
        // todo: convert to UID before sending to create_instance
        return match self.inner.lock().unwrap().create_instance(cid) {
            Ok(mut object) => {
                if object.as_edit_controller().is_some() {
                    let mut edit_controller = VST3EditController::new();
                    edit_controller.set_plugin_base(object);
                    *obj = Box::into_raw(edit_controller) as *mut c_void;
                    ResOk.into()
                } else if object.as_component().is_some() {
                    let mut component = VST3Component::new();
                    component.set_plugin_base(object);
                    *obj = Box::into_raw(component) as *mut c_void;
                    ResOk.into()
                } else {
                    ResultFalse.into()
                }
            }
            Err(r) => r.into(),
        };
    }
}
