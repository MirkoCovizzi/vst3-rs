extern crate log;

use std::os::raw::c_void;
use std::ptr::{copy_nonoverlapping, null_mut};

use crate::{kStereo, strcpy, wstrcpy, UID};
use vst3_sys::base::{
    kInvalidArgument, kNotImplemented, kResultFalse, kResultOk, kResultTrue, tresult, FIDString,
    IPluginBase, TBool,
};
use vst3_sys::vst::BusDirections::{kInput, kOutput};
use vst3_sys::vst::BusFlags::kDefaultActive;
use vst3_sys::vst::BusTypes::kMain;
use vst3_sys::vst::MediaTypes::{kAudio, kEvent};
use vst3_sys::vst::{
    AudioBusBuffers, BusDirection, BusDirections, BusFlags, BusInfo, BusType, IAudioProcessor,
    IComponent, IEditController, MediaType, MediaTypes, ParameterInfo, ProcessData, ProcessSetup,
    RoutingInfo, SpeakerArrangement, TChar,
};
use vst3_sys::IID;
use vst3_sys::VST3;

fn get_channel_count(arr: SpeakerArrangement) -> i32 {
    let mut arr = arr;
    let mut count = 0;
    while arr != 0 {
        if (arr & 1) == 1 {
            count += 1;
        }
        arr >>= 1;
    }
    count
}

trait Bus {
    fn set_active(&mut self, state: bool);
    unsafe fn get_info(&self, info: *mut BusInfo);
}

struct BaseBus {
    name: String,
    bus_type: BusType,
    flags: i32,
    active: bool,
}
impl BaseBus {
    fn new(name: &str, bus_type: BusType, flags: i32) -> Self {
        Self {
            name: name.to_string(),
            bus_type,
            flags,
            active: false,
        }
    }
}

impl Bus for BaseBus {
    fn set_active(&mut self, state: bool) {
        self.active = state
    }

    unsafe fn get_info(&self, info: *mut BusInfo) {
        wstrcpy(&self.name, (*info).name.as_mut_ptr());
        (*info).bus_type = self.bus_type;
        (*info).flags = self.flags as u32;
    }
}

struct AudioBus {
    inner: BaseBus,
    speaker_arr: SpeakerArrangement,
}
impl AudioBus {
    fn new(name: &str, bus_type: BusType, flags: i32, speaker_arr: SpeakerArrangement) -> Self {
        Self {
            inner: BaseBus {
                name: name.to_string(),
                bus_type,
                flags,
                active: false,
            },
            speaker_arr,
        }
    }
}

impl Bus for AudioBus {
    fn set_active(&mut self, state: bool) {
        self.inner.active = state
    }

    unsafe fn get_info(&self, info: *mut BusInfo) {
        (*info).channel_count = get_channel_count(self.speaker_arr);
        wstrcpy(&self.inner.name, (*info).name.as_mut_ptr());
        (*info).bus_type = self.inner.bus_type;
        (*info).flags = self.inner.flags as u32;
    }
}

struct EventBus {
    inner: BaseBus,
    channel_count: i32,
}
impl EventBus {
    fn new(name: &str, bus_type: BusType, flags: i32, channel_count: i32) -> Self {
        Self {
            inner: BaseBus {
                name: name.to_string(),
                bus_type,
                flags,
                active: false,
            },
            channel_count,
        }
    }
}

impl Bus for EventBus {
    fn set_active(&mut self, state: bool) {
        self.inner.active = state
    }

    unsafe fn get_info(&self, info: *mut BusInfo) {
        (*info).channel_count = self.channel_count;
        wstrcpy(&self.inner.name, (*info).name.as_mut_ptr());
        (*info).bus_type = self.inner.bus_type;
        (*info).flags = self.inner.flags as u32;
    }
}

struct BusVec {
    inner: Vec<Box<dyn Bus>>,
    type_: MediaType,
    direction: BusDirection,
}
impl BusVec {
    fn new(type_: MediaType, direction: BusDirection) -> Self {
        Self {
            inner: vec![],
            type_,
            direction,
        }
    }
}

pub struct PluginInfo {
    pub cid: UID,
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
            cid: [0; 4],
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

#[VST3(implements(IComponent, IEditController, IAudioProcessor))]
pub(crate) struct PluginComponent {
    component: *mut c_void,
    audio_inputs: BusVec,
    audio_outputs: BusVec,
    event_inputs: BusVec,
    event_outputs: BusVec,
}

impl PluginComponent {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(
            null_mut(),
            BusVec::new(kAudio as MediaType, kInput as BusDirection),
            BusVec::new(kAudio as MediaType, kOutput as BusDirection),
            BusVec::new(kEvent as MediaType, kInput as BusDirection),
            BusVec::new(kEvent as MediaType, kOutput as BusDirection),
        )
    }

    fn get_bus_vec(&self, type_: MediaType, dir: BusDirection) -> Option<&BusVec> {
        if type_ == kAudio as MediaType {
            return if dir == kInput as BusDirection {
                Some(&self.audio_inputs)
            } else {
                Some(&self.audio_outputs)
            };
        } else if type_ == kEvent as MediaType {
            return if dir == kInput as BusDirection {
                Some(&self.event_inputs)
            } else {
                Some(&self.event_outputs)
            };
        }
        None
    }

    fn get_bus_vec_mut(&mut self, type_: MediaType, dir: BusDirection) -> Option<&mut BusVec> {
        if type_ == kAudio as MediaType {
            return if dir == kInput as BusDirection {
                Some(&mut self.audio_inputs)
            } else {
                Some(&mut self.audio_outputs)
            };
        } else if type_ == kEvent as MediaType {
            return if dir == kInput as BusDirection {
                Some(&mut self.event_inputs)
            } else {
                Some(&mut self.event_outputs)
            };
        }
        None
    }

    fn add_audio_input(
        &mut self,
        name: &str,
        arr: SpeakerArrangement,
        bus_type: BusType,
        flags: i32,
    ) {
        let new_bus = AudioBus::new(name, bus_type, flags, arr);
        self.audio_inputs.inner.push(Box::new(new_bus))
    }

    fn add_audio_output(
        &mut self,
        name: &str,
        arr: SpeakerArrangement,
        bus_type: BusType,
        flags: i32,
    ) {
        let new_bus = AudioBus::new(name, bus_type, flags, arr);
        self.audio_outputs.inner.push(Box::new(new_bus))
    }

    fn add_event_input(&mut self, name: &str, channels: i32, bus_type: BusType, flags: i32) {
        let new_bus = EventBus::new(name, bus_type, flags, channels);
        self.event_inputs.inner.push(Box::new(new_bus))
    }

    fn add_event_output(&mut self, name: &str, channels: i32, bus_type: BusType, flags: i32) {
        let new_bus = EventBus::new(name, bus_type, flags, channels);
        self.event_outputs.inner.push(Box::new(new_bus))
    }

    pub(crate) fn set_component(&mut self, component: *mut c_void) {
        self.component = component
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_component(&self) -> &Box<dyn Plugin> {
        *(self.component as *mut &Box<dyn Plugin>)
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_component_mut(&mut self) -> &mut Box<dyn Plugin> {
        *(self.component as *mut &mut Box<dyn Plugin>)
    }
}

impl IPluginBase for PluginComponent {
    unsafe fn initialize(&mut self, _host_context: *mut c_void) -> tresult {
        self.add_audio_input(
            "Stereo In",
            kStereo,
            kMain as BusDirection,
            kDefaultActive as i32,
        );
        self.add_audio_input(
            "Stereo Out",
            kStereo,
            kMain as BusDirection,
            kDefaultActive as i32,
        );
        self.add_event_input("Event In", 1, kMain as BusDirection, kDefaultActive as i32);
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
        kNotImplemented
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        if let Some(bus_vec) = self.get_bus_vec(type_, dir) {
            return bus_vec.inner.len() as i32;
        }
        0
    }

    unsafe fn get_bus_info(&self, type_: i32, dir: i32, index: i32, info: *mut BusInfo) -> tresult {
        return if let Some(bus_vec) = self.get_bus_vec(type_, dir) {
            if index >= bus_vec.inner.len() as i32 {
                return kInvalidArgument;
            }
            let bus = &bus_vec.inner[index as usize];
            (*info).media_type = type_;
            (*info).direction = dir;
            bus.get_info(info);
            kResultTrue
        } else {
            kInvalidArgument
        };
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn activate_bus(&mut self, type_: i32, dir: i32, index: i32, state: TBool) -> tresult {
        if let Some(bus_vec) = self.get_bus_vec_mut(type_, dir) {
            if index >= bus_vec.inner.len() as i32 {
                return kInvalidArgument;
            }
            let mut bus = &mut bus_vec.inner[index as usize];
            bus.set_active(state != 0);
            kResultTrue
        } else {
            kInvalidArgument
        }
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
