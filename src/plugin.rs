use vst3_com::sys::GUID;

use std::os::raw::{c_short, c_void};
use std::ptr::{copy_nonoverlapping, null_mut};

use vst3_sys::base::{kInvalidArgument, kResultFalse, kResultOk, tresult, IPluginBase, TBool};
use vst3_sys::vst::{BusDirections, BusFlags, BusInfo, IComponent, MediaTypes, RoutingInfo};
use vst3_sys::IID;
use vst3_sys::VST3;
use widestring::U16CString;

unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub struct ComponentInfo {
    pub cid: IID,
    pub cardinality: i32,
    pub category: String,
    pub name: String,
    pub class_flags: u32,
    pub subcategories: String,
    pub vendor: String,
    pub version: String,
    pub sdk_version: String,
}

pub trait Component {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn info(&self) -> ComponentInfo {
        ComponentInfo {
            cid: GUID { data: [0; 16] },
            cardinality: 0,
            category: "".to_string(),
            name: "".to_string(),
            class_flags: 0,
            subcategories: "".to_string(),
            vendor: "".to_string(),
            version: "".to_string(),
            sdk_version: "".to_string(),
        }
    }
}

#[VST3(implements(IComponent))]
pub struct PluginComponent {
    component: *mut c_void,
}

impl PluginComponent {
    pub(crate) fn new() -> Box<Self> {
        let factory = null_mut();
        Self::allocate(factory)
    }

    pub(crate) fn set_component(&mut self, component: *mut c_void) {
        self.component = component;
    }

    pub unsafe fn get_component(&self) -> &Box<dyn Component> {
        &*(self.component as *mut Box<dyn Component>)
    }

    pub unsafe fn get_component_mut(&mut self) -> &mut Box<dyn Component> {
        &mut *(self.component as *mut Box<dyn Component>)
    }
}

impl IComponent for PluginComponent {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, type_: i32, _dir: i32) -> i32 {
        if type_ == MediaTypes::kAudio as i32 {
            1
        } else {
            0
        }
    }

    unsafe fn get_bus_info(&self, type_: i32, dir: i32, _idx: i32, info: *mut BusInfo) -> tresult {
        if type_ == MediaTypes::kAudio as i32 {
            let info = &mut *info;
            if dir == BusDirections::kInput as i32 {
                info.direction = dir;
                info.bus_type = MediaTypes::kAudio as i32;
                info.channel_count = 2;
                info.flags = BusFlags::kDefaultActive as u32;
                wstrcpy("Audio Input", info.name.as_mut_ptr());
            } else {
                info.direction = dir;
                info.bus_type = MediaTypes::kAudio as i32;
                info.channel_count = 2;
                info.flags = BusFlags::kDefaultActive as u32;
                wstrcpy("Audio Output", info.name.as_mut_ptr());
            }
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn activate_bus(&mut self, _type_: i32, _dir: i32, _idx: i32, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&mut self, _state: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&mut self, _state: *mut c_void) -> tresult {
        kResultOk
    }
}

impl IPluginBase for PluginComponent {
    unsafe fn initialize(&mut self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }
    unsafe fn terminate(&mut self) -> tresult {
        kResultOk
    }
}
