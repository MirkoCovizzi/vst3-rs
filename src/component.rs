extern crate log;

use std::os::raw::c_void;
use std::ptr::{copy_nonoverlapping, null_mut};

use crate::wstrcpy;
use vst3_com::sys::GUID;
use vst3_sys::base::{
    kInvalidArgument, kResultFalse, kResultOk, tresult, FIDString, IPluginBase, TBool,
};
use vst3_sys::vst::{
    AudioBusBuffers, BusDirections, BusFlags, BusInfo, IAudioProcessor, IComponent,
    IEditController, MediaTypes, ParameterInfo, ProcessData, ProcessSetup, RoutingInfo, TChar,
};
use vst3_sys::IID;
use vst3_sys::VST3;

pub struct PluginInfo {
    pub cid: GUID,
    pub cardinality: i32,
    pub category: String,
    pub name: String,
    pub class_flags: u32,
    pub subcategories: String,
    pub vendor: String,
    pub version: String,
    pub sdk_version: String,
}

pub trait Plugin {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
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

    fn get_controller_id(&self) -> Option<GUID> {
        None
    }
}

#[VST3(implements(IComponent, IEditController, IAudioProcessor))]
pub struct PluginComponent {
    component: *mut c_void,
}

impl PluginComponent {
    pub(crate) fn new() -> Box<Self> {
        let component = null_mut();
        Self::allocate(component)
    }

    pub(crate) fn set_component(&mut self, component: *mut c_void) {
        self.component = component;
    }

    pub unsafe fn get_component(&self) -> &Box<dyn Plugin> {
        *(self.component as *mut &Box<dyn Plugin>)
    }

    pub unsafe fn get_component_mut(&mut self) -> &mut Box<dyn Plugin> {
        *(self.component as *mut &mut Box<dyn Plugin>)
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

impl IComponent for PluginComponent {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kResultFalse
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

impl IEditController for PluginComponent {
    unsafe fn set_component_state(&mut self, _state: *mut c_void) -> tresult {
        kResultOk
    }
    unsafe fn set_state(&mut self, _state: *mut c_void) -> tresult {
        kResultOk
    }
    unsafe fn get_state(&mut self, _state: *mut c_void) -> tresult {
        kResultOk
    }
    unsafe fn get_parameter_count(&self) -> i32 {
        0
    }
    unsafe fn get_parameter_info(&self, _: i32, _: *mut ParameterInfo) -> tresult {
        kResultFalse
    }
    unsafe fn get_param_string_by_value(
        &self,
        _id: u32,
        _value_normalized: f64,
        _string: *mut TChar,
    ) -> tresult {
        kResultFalse
    }
    unsafe fn get_param_value_by_string(
        &self,
        _id: u32,
        _string: *mut TChar,
        _value_normalized: *mut f64,
    ) -> tresult {
        kResultFalse
    }
    unsafe fn normalized_param_to_plain(&self, _id: u32, _value_normalized: f64) -> f64 {
        0.0
    }
    unsafe fn plain_param_to_normalized(&self, _id: u32, _plain_value: f64) -> f64 {
        0.0
    }
    unsafe fn get_param_normalized(&self, _id: u32) -> f64 {
        0.0
    }
    unsafe fn set_param_normalized(&mut self, _id: u32, _value: f64) -> tresult {
        kResultOk
    }
    unsafe fn set_component_handler(&mut self, _handler: *mut c_void) -> tresult {
        kResultOk
    }
    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        null_mut()
    }
}

impl IAudioProcessor for PluginComponent {
    unsafe fn set_bus_arrangements(
        &self,
        _inputs: *mut u64,
        _num_ins: i32,
        _outputs: *mut u64,
        _num_outs: i32,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn get_bus_arrangements(&self, _dir: i32, _index: i32, arr: *mut u64) -> i32 {
        let arr = &mut *arr;
        if (*arr == 0x0) || (*arr == 0x1) || (*arr == 0x3) {
            kResultOk
        } else {
            *arr = 0x03;
            kResultOk
        }
    }

    unsafe fn can_process_sample_size(&self, _symbolic_sample_size: i32) -> i32 {
        kResultOk
    }

    unsafe fn get_latency_sample(&self) -> u32 {
        0
    }
    unsafe fn setup_processing(&mut self, _setup: *mut ProcessSetup) -> tresult {
        kResultOk
    }
    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        kResultOk
    }
    unsafe fn process(&mut self, data: *mut ProcessData) -> tresult {
        let data = &*data;
        let num_samples = data.num_samples as usize;
        if data.inputs.is_null() || data.outputs.is_null() {
            return kResultOk;
        }
        let inputs: &mut AudioBusBuffers = &mut *data.inputs;
        let outputs: &mut AudioBusBuffers = &mut *data.outputs;
        let num_channels = inputs.num_channels as usize;
        let input_ptr = std::slice::from_raw_parts(inputs.buffers, num_channels);
        let output_ptr = std::slice::from_raw_parts(outputs.buffers, num_channels);
        let sample_size = if data.symbolic_sample_size == 0 { 4 } else { 8 };
        for (i, o) in input_ptr.iter().zip(output_ptr.iter()) {
            copy_nonoverlapping(*i, *o, num_samples * sample_size);
        }
        kResultOk
    }
    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }
}
