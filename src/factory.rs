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
    strcpy, wstrcpy, AudioProcessor, Component, EditController, PluginBase, ResultErr,
    VST3Component, VST3EditController,
};
use std::sync::Mutex;

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

    fn get_info(&self) -> FactoryInfo {
        FactoryInfo {
            vendor: "".to_string(),
            url: "".to_string(),
            email: "".to_string(),
            // todo: manage this in a nicer way (new type?)
            flags: kDefaultFactoryFlags,
        }
    }

    fn get_edit_controllers(&self) -> &Vec<Mutex<Box<dyn EditController>>>;
    fn get_components(&self) -> &Vec<Mutex<Box<dyn Component>>>;
}
/*
trait PluginFactory {
    fn get_factory_info(&self) -> Result<FactoryInfo, ResultErr>;
    fn count_classes(&self) -> i32;
    fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult;
    fn create_instance(
        &self,
        cid: *mut IID,
        _iid: *mut IID,
        obj: *mut *mut c_void,
    ) -> tresult;
}
*/

#[VST3(implements(IPluginFactory, IPluginFactory2, IPluginFactory3))]
pub(crate) struct VST3PluginFactory {
    inner: *mut c_void,
}

impl VST3PluginFactory {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(null_mut())
    }

    pub(crate) fn set_factory(&mut self, factory: *mut c_void) {
        self.inner = factory;
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_factory(&self) -> &Box<dyn Factory> {
        &*(self.inner as *mut Box<dyn Factory>)
    }
}

impl IPluginFactory3 for VST3PluginFactory {
    unsafe fn get_class_info_unicode(&self, index: i32, info: *mut PClassInfoW) -> i32 {
        let num_controllers = self.get_factory().get_edit_controllers().len();
        let num_components = self.get_factory().get_components().len();

        if index as usize >= num_controllers + num_components {
            return InvalidArgument.into();
        }

        let plugin_info;
        if index as usize >= num_controllers {
            let component = self
                .get_factory()
                .get_components()
                .get(index as usize - num_controllers)
                .unwrap();
            plugin_info = component.lock().unwrap().get_class_info();
        } else {
            let edit_controller = self
                .get_factory()
                .get_edit_controllers()
                .get(index as usize)
                .unwrap();
            plugin_info = edit_controller.lock().unwrap().get_class_info();
        }

        let info = &mut *info;
        info.cid = plugin_info.cid.to_guid();
        info.cardinality = plugin_info.cardinality as i32;
        strcpy(
            &plugin_info.category.to_string(),
            info.category.as_mut_ptr(),
        );
        wstrcpy(&plugin_info.name, info.name.as_mut_ptr());
        info.class_flags = plugin_info.class_flags;
        strcpy(
            &plugin_info.subcategories.to_string(),
            info.subcategories.as_mut_ptr(),
        );
        wstrcpy(&plugin_info.vendor, info.vendor.as_mut_ptr());
        wstrcpy(&plugin_info.version, info.version.as_mut_ptr());
        wstrcpy(&plugin_info.sdk_version, info.sdk_version.as_mut_ptr());

        ResOk.into()
    }

    unsafe fn set_host_context(&self, _context: *mut c_void) -> i32 {
        NotImplemented.into()
    }
}

impl IPluginFactory2 for VST3PluginFactory {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> i32 {
        let num_controllers = self.get_factory().get_edit_controllers().len();
        let num_components = self.get_factory().get_components().len();

        if index as usize >= num_controllers + num_components {
            return InvalidArgument.into();
        }

        let plugin_info;
        if index as usize >= num_controllers {
            let component = self
                .get_factory()
                .get_components()
                .get(index as usize - num_controllers)
                .unwrap();
            plugin_info = component.lock().unwrap().get_class_info();
        } else {
            let edit_controller = self
                .get_factory()
                .get_edit_controllers()
                .get(index as usize)
                .unwrap();
            plugin_info = edit_controller.lock().unwrap().get_class_info();
        }

        let info = &mut *info;
        info.cid = plugin_info.cid.to_guid();
        info.cardinality = plugin_info.cardinality as i32;
        strcpy(
            &plugin_info.category.to_string(),
            info.category.as_mut_ptr(),
        );
        strcpy(&plugin_info.name, info.name.as_mut_ptr());
        info.class_flags = plugin_info.class_flags;
        strcpy(
            &plugin_info.subcategories.to_string(),
            info.subcategories.as_mut_ptr(),
        );
        strcpy(&plugin_info.vendor, info.vendor.as_mut_ptr());
        strcpy(&plugin_info.version, info.version.as_mut_ptr());
        strcpy(&plugin_info.sdk_version, info.sdk_version.as_mut_ptr());

        ResOk.into()
    }
}

impl IPluginFactory for VST3PluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> i32 {
        let factory_info = self.get_factory().get_info();

        let info = &mut *info;
        strcpy(&factory_info.vendor, info.vendor.as_mut_ptr());
        strcpy(&factory_info.url, info.url.as_mut_ptr());
        strcpy(&factory_info.email, info.email.as_mut_ptr());
        info.flags = factory_info.flags as i32;

        ResOk.into()
    }

    unsafe fn count_classes(&self) -> i32 {
        let num_controllers = self.get_factory().get_edit_controllers().len() as i32;
        let num_components = self.get_factory().get_components().len() as i32;
        num_controllers + num_components
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> i32 {
        let num_controllers = self.get_factory().get_edit_controllers().len();
        let num_components = self.get_factory().get_components().len();

        if index as usize >= num_controllers + num_components {
            return InvalidArgument.into();
        }

        let plugin_info;
        if index as usize >= num_controllers {
            let component = self
                .get_factory()
                .get_components()
                .get(index as usize - num_controllers)
                .unwrap();
            plugin_info = component.lock().unwrap().get_class_info();
        } else {
            let edit_controller = self
                .get_factory()
                .get_edit_controllers()
                .get(index as usize)
                .unwrap();
            plugin_info = edit_controller.lock().unwrap().get_class_info();
        }

        let info = &mut *info;
        info.cid = plugin_info.cid.to_guid();
        info.cardinality = plugin_info.cardinality as i32;
        strcpy(
            &plugin_info.category.to_string(),
            info.category.as_mut_ptr(),
        );
        strcpy(&plugin_info.name, info.name.as_mut_ptr());

        ResOk.into()
    }

    unsafe fn create_instance(&self, cid: *mut IID, _iid: *mut IID, obj: *mut *mut c_void) -> i32 {
        let controllers = self.get_factory().get_edit_controllers();
        for c in controllers {
            if *cid == c.lock().unwrap().get_class_info().cid.to_guid() {
                let c = Box::into_raw(Box::new(c)) as *mut _;
                let mut edit_controller = VST3EditController::new();
                edit_controller.set_edit_controller(c);
                *obj = Box::into_raw(edit_controller) as *mut c_void;
                return ResOk.into();
            }
        }
        let components = self.get_factory().get_components();
        for c in components {
            if *cid == c.lock().unwrap().get_class_info().cid.to_guid() {
                let c = Box::into_raw(Box::new(c)) as *mut _;
                let mut component = VST3Component::new();
                component.set_component(c);
                *obj = Box::into_raw(component) as *mut c_void;
                return ResOk.into();
            }
        }
        ResultFalse.into()
    }
}
