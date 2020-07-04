use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::sys::GUID;
use vst3_com::IID;
use vst3_sys::base::{
    IPluginFactory, IPluginFactory2, IPluginFactory3, PClassInfo, PClassInfo2, PClassInfoW,
    PFactoryInfo,
};
use vst3_sys::vst::kDefaultFactoryFlags;
use vst3_sys::VST3;

use crate::ResultErr::{InternalError, InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    strcpy, wstrcpy, AudioProcessor, ClassInfo, Component, EditController, HostApplication,
    Offset0, Offset1, Offset2, PluginBase, ResultErr, ResultOk, Unknown, VST3Component,
    VST3EditController, UID,
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
    fn count_classes(&self) -> Result<usize, ResultErr>;
    fn get_class_info(&self, index: usize) -> Result<&ClassInfo, ResultErr>;
    fn create_instance(&self, cid: &UID) -> Result<Box<dyn PluginBase>, ResultErr>;
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

    fn count_classes(&self) -> Result<usize, ResultErr> {
        unimplemented!()
    }

    fn get_class_info(&self, _index: usize) -> Result<&ClassInfo, ResultErr> {
        unimplemented!()
    }

    fn create_instance(&self, _cid: &UID) -> Result<Box<dyn PluginBase>, ResultErr> {
        unimplemented!()
    }

    fn set_host_context(&mut self, _context: HostApplication) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }
}

#[repr(C)]
pub(crate) struct VST3PluginFactory {
    __ipluginfactoryvptr: *const <dyn IPluginFactory as vst3_com::ComInterface>::VTable,
    __ipluginfactory2vptr: *const <dyn IPluginFactory2 as vst3_com::ComInterface>::VTable,
    __ipluginfactory3vptr: *const <dyn IPluginFactory3 as vst3_com::ComInterface>::VTable,
    __refcnt: std::cell::Cell<u32>,
    inner: Mutex<Box<dyn PluginFactory>>,
}

impl VST3PluginFactory {
    fn allocate(inner: Mutex<Box<dyn PluginFactory>>) -> Box<VST3PluginFactory> {
        let ipluginfactory_vtable = <dyn IPluginFactory as ::vst3_com::ProductionComInterface<
            VST3PluginFactory,
        >>::vtable::<Offset0>();
        let __ipluginfactoryvptr = Box::into_raw(Box::new(ipluginfactory_vtable));
        let ipluginfactory2_vtable = <dyn IPluginFactory2 as ::vst3_com::ProductionComInterface<
            VST3PluginFactory,
        >>::vtable::<Offset1>();
        let __ipluginfactory2vptr = Box::into_raw(Box::new(ipluginfactory2_vtable));
        let ipluginfactory3_vtable = <dyn IPluginFactory3 as ::vst3_com::ProductionComInterface<
            VST3PluginFactory,
        >>::vtable::<Offset2>();
        let __ipluginfactory3vptr = Box::into_raw(Box::new(ipluginfactory3_vtable));
        let out = VST3PluginFactory {
            __ipluginfactoryvptr,
            __ipluginfactory2vptr,
            __ipluginfactory3vptr,
            __refcnt: std::cell::Cell::new(1),
            inner,
        };
        Box::new(out)
    }
}

unsafe impl vst3_com::CoClass for VST3PluginFactory {}

impl vst3_com::interfaces::IUnknown for VST3PluginFactory {
    unsafe fn query_interface(
        &self,
        riid: *const vst3_com::sys::IID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> vst3_com::sys::HRESULT {
        let riid = &*riid;
        if riid == &vst3_com::interfaces::iunknown::IID_IUNKNOWN {
            *ppv = &self.__ipluginfactoryvptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IPluginFactory as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid)
        {
            *ppv = &self.__ipluginfactoryvptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IPluginFactory2 as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid)
        {
            *ppv = &self.__ipluginfactory2vptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IPluginFactory3 as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid)
        {
            *ppv = &self.__ipluginfactory3vptr as *const _ as *mut std::ffi::c_void;
        } else {
            *ppv = std::ptr::null_mut::<std::ffi::c_void>();
            return vst3_com::sys::E_NOINTERFACE;
        }
        self.add_ref();
        vst3_com::sys::NOERROR
    }

    unsafe fn add_ref(&self) -> u32 {
        let value = self
            .__refcnt
            .get()
            .checked_add(1)
            .expect("Overflow of reference count");
        self.__refcnt.set(value);
        value
    }

    unsafe fn release(&self) -> u32 {
        let value = self
            .__refcnt
            .get()
            .checked_sub(1)
            .expect("Underflow of reference count");
        self.__refcnt.set(value);
        let __refcnt = self.__refcnt.get();
        if __refcnt == 0 {
            Box::from_raw(
                self.__ipluginfactoryvptr
                    as *mut <dyn IPluginFactory as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(
                self.__ipluginfactory2vptr
                    as *mut <dyn IPluginFactory2 as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(
                self.__ipluginfactory3vptr
                    as *mut <dyn IPluginFactory3 as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(self as *const _ as *mut VST3PluginFactory);
        }
        __refcnt
    }
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
        if index < 0 {
            return InvalidArgument.into();
        }
        match self.inner.lock().unwrap().get_class_info(index as usize) {
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
        if index < 0 {
            return InvalidArgument.into();
        }
        match self.inner.lock().unwrap().get_class_info(index as usize) {
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
            Ok(count) => {
                if count > i32::MAX as usize {
                    log::trace!(
                        "count_classes(): returned value is too big! {}usize > {}i32",
                        count,
                        i32::MAX
                    );
                    InternalError.into()
                } else {
                    count as i32
                }
            }
            Err(r) => 0,
        }
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> i32 {
        if index < 0 {
            return InvalidArgument.into();
        }
        match self.inner.lock().unwrap().get_class_info(index as usize) {
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
        iid: *const IID,
        obj: *mut *mut c_void,
    ) -> i32 {
        if cid.is_null() || iid.is_null() {
            return InvalidArgument.into();
        }
        let uid = UID::from_guid(&*cid as &GUID);
        return match self.inner.lock().unwrap().create_instance(&uid) {
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
