use std::os::raw::c_void;
use std::ptr::{copy_nonoverlapping, null_mut};

use log::*;

use crate::PluginBusDirection::{Input, Output};
use crate::PluginBusFlag::DefaultActive;
use crate::PluginBusType::Main;
use crate::PluginClassCardinality::ManyInstances;
use crate::PluginResult::{InvalidArgument, NotImplemented, ResultFalse, ResultOk, ResultTrue};
use crate::PluginSpeakerArrangement::Stereo;
use crate::{
    kStereo, strcpy, wstrcpy, AudioBusBuilder, BStream, Category, EventBusBuilder,
    EventList, Events, FxSubcategory, ParameterChanges, Parameters, PluginBusDirection,
    PluginBusFlag, PluginBusType, PluginClassCardinality, PluginProcessData, PluginResult,
    PluginSpeakerArrangement, ProcessMode, SymbolicSampleSize, IO, UID,
};
use num_traits::Float;
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};
use vst3_com::sys::RegSetValueExA;
use vst3_sys::base::{
    kInvalidArgument, kNotImplemented, kResultFalse, kResultOk, kResultTrue, tresult, FIDString,
    IPluginBase, TBool,
};
use vst3_sys::vst::BusDirections::{kInput, kOutput};
use vst3_sys::vst::BusFlags::kDefaultActive;
use vst3_sys::vst::BusTypes::{kAux, kMain};
use vst3_sys::vst::MediaTypes::{kAudio, kEvent};
use vst3_sys::vst::ProcessModes::kOffline;
use vst3_sys::vst::{
    AudioBusBuffers, BusDirection, BusDirections, BusFlags, BusInfo, BusType, Event,
    IAudioProcessor, IComponent, IEditController, MediaType, MediaTypes, ParameterInfo,
    ProcessData, ProcessSetup, RoutingInfo, SpeakerArrangement, TChar,
};
use vst3_sys::IID;
use vst3_sys::VST3;
use widestring::U16CString;

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

pub(crate) trait Bus {
    fn set_active(&mut self, state: bool);
    unsafe fn get_info(&self, info: *mut BusInfo);
}

#[derive(Clone)]
pub(crate) struct BaseBus {
    pub(crate) name: String,
    pub(crate) bus_type: BusType,
    pub(crate) flags: i32,
    pub(crate) active: bool,
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

pub struct AudioBus {
    pub(crate) inner: BaseBus,
    pub(crate) speaker_arr: SpeakerArrangement,
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

pub struct EventBus {
    pub(crate) inner: BaseBus,
    pub(crate) channel_count: i32,
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

pub(crate) struct BusVec {
    pub(crate) inner: Vec<Box<dyn Bus>>,
    type_: MediaType,
    direction: BusDirection,
}

impl BusVec {
    pub(crate) fn new(type_: MediaType, direction: BusDirection) -> Self {
        Self {
            inner: vec![],
            type_,
            direction,
        }
    }
}

pub struct Info {
    pub cid: UID,
    pub cardinality: PluginClassCardinality,
    pub category: Category,
    pub name: String,
    pub class_flags: u32,
    pub subcategories: FxSubcategory,
    pub vendor: String,
    pub version: String,
    pub sdk_version: String,
}

pub struct InfoBuilder {
    cid: UID,
    cardinality: PluginClassCardinality,
    category: Category,
    name: String,
    class_flags: u32,
    subcategories: FxSubcategory,
    vendor: String,
    version: String,
    sdk_version: String,
}

impl InfoBuilder {
    pub fn new(cid: UID) -> InfoBuilder {
        InfoBuilder {
            cid,
            name: "VST3".to_string(),
            cardinality: ManyInstances,
            category: Category::AudioEffect,
            class_flags: 0,
            subcategories: FxSubcategory::Fx,
            vendor: String::new(),
            version: "0.1.0".to_string(),
            sdk_version: "VST 3.6.14".to_string(),
        }
    }

    pub fn name(mut self, name: &str) -> InfoBuilder {
        self.name = name.to_string();
        self
    }

    pub fn cardinality(mut self, cardinality: PluginClassCardinality) -> InfoBuilder {
        self.cardinality = cardinality;
        self
    }

    pub fn category(mut self, category: Category) -> InfoBuilder {
        self.category = category;
        self
    }

    pub fn class_flags(mut self, class_flags: u32) -> InfoBuilder {
        self.class_flags = class_flags;
        self
    }

    pub fn subcategories(mut self, subcategories: FxSubcategory) -> InfoBuilder {
        self.subcategories = subcategories;
        self
    }

    pub fn vendor(mut self, vendor: &str) -> InfoBuilder {
        self.vendor = vendor.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> InfoBuilder {
        self.version = version.to_string();
        self
    }

    pub fn sdk_version(mut self, sdk_version: &str) -> InfoBuilder {
        self.sdk_version = sdk_version.to_string();
        self
    }

    pub fn build(&self) -> Info {
        Info {
            cid: self.cid.clone(),
            cardinality: self.cardinality.clone(),
            category: self.category.clone(),
            name: self.name.clone(),
            class_flags: self.class_flags,
            subcategories: self.subcategories.clone(),
            vendor: self.vendor.clone(),
            version: self.version.clone(),
            sdk_version: self.sdk_version.clone(),
        }
    }
}

pub trait Plugin {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn get_info(&self) -> Info {
        InfoBuilder::new(UID::new([0x0, 0x0, 0x0, 0x0])).build()
    }

    fn initialize(&self) {
        self.get_io()
            .add_audio_input(AudioBusBuilder::new("Stereo In", Stereo).build());
        self.get_io()
            .add_audio_output(AudioBusBuilder::new("Stereo Out", Stereo).build());
        self.get_io()
            .add_event_input(EventBusBuilder::new("Event In", 1).build());
    }

    fn terminate(&self) {
        self.get_parameters().remove_all();

        self.get_io().clear_audio_inputs();
        self.get_io().clear_audio_outputs();
        self.get_io().clear_event_inputs();
        self.get_io().clear_event_outputs();
    }

    fn read_input_param_changes(&self, param_changes: &ParameterChanges) {
        let num_params_changed = param_changes.get_parameter_count();
        for i in 0..num_params_changed {
            if let Some(param_queue) = param_changes.get_parameter_data(i) {
                let num_points = param_queue.get_point_count();
                if let Some(_) = self.get_parameters().get(param_queue.get_parameter_id()) {
                    if let Some(point) = param_queue.get_point(num_points - 1) {
                        self.get_parameters()
                            .set(param_queue.get_parameter_id(), point.value);
                    }
                }
            }
        }
    }

    fn read_input_events(&self, event_list: &EventList) {
        let num_events = event_list.get_event_count();
        for i in 0..num_events {
            if let Some(event) = event_list.get_event(i) {
                self.get_events()
                    .call_event_action(event, self.get_parameters());
            }
        }
    }

    fn process(&self, _data: PluginProcessData<f32>) {}
    fn process_f64(&self, _data: PluginProcessData<f64>) {}

    fn set_state(&self, state: BStream) -> PluginResult {
        let param_order = self.get_parameters().get_param_order();
        for key in param_order.borrow().iter() {
            if let Some(val) = state.read::<f64>() {
                match self.get_parameters().set(*key, val) {
                    ResultOk => continue,
                    ResultTrue => continue,
                    result => return result,
                }
            }
        }
        ResultFalse
    }
    fn get_state(&self, state: BStream) -> PluginResult {
        let param_order = self.get_parameters().get_param_order();
        for key in param_order.borrow().iter() {
            if let Some(val) = self.get_parameters().get(*key) {
                match state.write::<f64>(val) {
                    ResultOk => continue,
                    ResultTrue => continue,
                    result => return result,
                }
            }
        }
        ResultFalse
    }

    fn get_parameters(&self) -> &Parameters;
    fn get_events(&self) -> &Events;
    fn get_io(&self) -> &IO;
}

struct ContextPtr(*mut c_void);

#[VST3(implements(IComponent, IEditController, IAudioProcessor))]
pub(crate) struct PluginComponent {
    inner: *mut c_void,
    host_context: RefCell<ContextPtr>,
}

impl PluginComponent {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(null_mut(), RefCell::new(ContextPtr(null_mut())))
    }

    pub(crate) fn set_component(&mut self, component: *mut c_void) {
        self.inner = component
    }

    #[allow(clippy::borrowed_box)]
    unsafe fn get_component(&self) -> &Box<dyn Plugin> {
        *(self.inner as *mut &Box<dyn Plugin>)
    }
}

impl IPluginBase for PluginComponent {
    unsafe fn initialize(&self, context: *mut c_void) -> i32 {
        if !self.host_context.borrow().0.is_null() {
            return ResultFalse.into();
        }
        self.host_context.borrow_mut().0 = context;

        self.get_component().initialize();

        ResultOk.into()
    }

    unsafe fn terminate(&self) -> i32 {
        self.get_component().terminate();

        self.host_context.borrow_mut().0 = null_mut();

        ResultOk.into()
    }
}

impl IComponent for PluginComponent {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> i32 {
        ResultFalse.into()
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> i32 {
        NotImplemented.into()
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        // todo: Refactor?
        if let Some(bus_vec) = self
            .get_component()
            .get_io()
            .borrow()
            .get_bus_vec(type_, dir)
        {
            return bus_vec.borrow().inner.len() as i32;
        }
        0
    }

    unsafe fn get_bus_info(&self, type_: i32, dir: i32, index: i32, info: *mut BusInfo) -> i32 {
        // todo: Refactor?
        return if let Some(bus_vec) = self.get_component().get_io().get_bus_vec(type_, dir) {
            if index >= bus_vec.borrow().inner.len() as i32 {
                return InvalidArgument.into();
            }
            let bus = &bus_vec.borrow_mut().inner[index as usize];
            (*info).media_type = type_;
            (*info).direction = dir;
            bus.get_info(info);
            ResultTrue.into()
        } else {
            InvalidArgument.into()
        };
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> i32 {
        ResultFalse.into()
    }

    unsafe fn activate_bus(&self, type_: i32, dir: i32, index: i32, state: TBool) -> i32 {
        self.get_component()
            .get_io()
            .activate_bus(type_, dir, index, state)
            .into()
    }

    unsafe fn set_active(&self, _state: TBool) -> i32 {
        ResultOk.into()
    }

    unsafe fn set_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = BStream::from_raw(state) {
            self.get_component().set_state(state);
            return ResultOk.into();
        }
        ResultFalse.into()
    }

    unsafe fn get_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = BStream::from_raw(state) {
            self.get_component().get_state(state);
            return ResultOk.into();
        }
        ResultFalse.into()
    }
}

impl IEditController for PluginComponent {
    unsafe fn set_component_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = BStream::from_raw(state) {
            let param_order = self.get_component().get_parameters().get_param_order();
            for key in param_order.borrow().iter() {
                if let Some(val) = state.read::<f64>() {
                    self.get_component()
                        .get_parameters()
                        .set_param_normalized(*key, val);
                } else {
                    return ResultFalse.into();
                }
            }
        }
        ResultFalse.into()
    }

    unsafe fn set_state(&self, _state: *mut c_void) -> i32 {
        ResultOk.into()
    }

    unsafe fn get_state(&self, _state: *mut c_void) -> i32 {
        ResultOk.into()
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        return self.get_component().get_parameters().get_parameter_count() as i32;
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> i32 {
        return match self
            .get_component()
            .get_parameters()
            .get_parameter_info(param_index as u32)
        {
            Ok(parameter_info) => {
                *info = parameter_info;
                kResultOk
            }
            Err(result) => result.into(),
        };
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut TChar,
    ) -> i32 {
        return match self
            .get_component()
            .get_parameters()
            .get_param_string_by_value(id, value_normalized)
        {
            Ok(param_string) => {
                wstrcpy(&param_string, string);
                ResultOk.into()
            }
            Err(result) => result.into(),
        };
    }

    unsafe fn get_param_value_by_string(
        &self,
        _id: u32,
        _string: *mut TChar,
        _value_normalized: *mut f64,
    ) -> i32 {
        ResultFalse.into()
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        return match self
            .get_component()
            .get_parameters()
            .normalized_param_to_plain(id, value_normalized)
        {
            Some(plain) => plain,
            None => 0.0,
        };
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        return match self
            .get_component()
            .get_parameters()
            .plain_param_to_normalized(id, plain_value)
        {
            Some(normalized) => normalized,
            None => 0.0,
        };
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        return match self
            .get_component()
            .get_parameters()
            .get_param_normalized(id)
        {
            Some(param_normalized) => param_normalized,
            None => 0.0,
        };
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> i32 {
        self.get_component()
            .get_parameters()
            .set_param_normalized(id, value)
            .into()
    }

    unsafe fn set_component_handler(&self, _handler: *mut c_void) -> i32 {
        ResultOk.into()
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
        ResultFalse.into()
    }

    unsafe fn get_bus_arrangements(&self, _dir: i32, _index: i32, arr: *mut u64) -> i32 {
        let arr = &mut *arr;
        if (*arr == 0x0) || (*arr == 0x1) || (*arr == 0x3) {
            ResultOk.into()
        } else {
            *arr = 0x03;
            ResultOk.into()
        }
    }

    unsafe fn can_process_sample_size(&self, _symbolic_sample_size: i32) -> i32 {
        ResultOk.into()
    }

    unsafe fn get_latency_sample(&self) -> u32 {
        0
    }

    unsafe fn setup_processing(&self, _setup: *mut ProcessSetup) -> tresult {
        ResultOk.into()
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        ResultOk.into()
    }

    unsafe fn process(&self, data: *mut ProcessData) -> i32 {
        if data.is_null() {
            return ResultOk.into();
        }

        match SymbolicSampleSize::from((*data).symbolic_sample_size) {
            SymbolicSampleSize::Sample32 => {
                let process_data = PluginProcessData::<f32>::from_raw(
                    (*data).num_inputs as usize,
                    (*data).num_outputs as usize,
                    (*data).inputs as *const AudioBusBuffers,
                    (*data).outputs,
                    ProcessMode::from((*data).process_mode),
                    (*data).num_samples as usize,
                    (*data).input_parameter_changes as *mut c_void,
                    (*data).output_parameter_changes as *mut c_void,
                    (*data).input_events as *mut c_void,
                    (*data).output_events as *mut c_void,
                );

                self.get_component().process(process_data);
            }
            SymbolicSampleSize::Sample64 => {
                let process_data = PluginProcessData::<f64>::from_raw(
                    (*data).num_inputs as usize,
                    (*data).num_outputs as usize,
                    (*data).inputs as *const AudioBusBuffers,
                    (*data).outputs,
                    ProcessMode::from((*data).process_mode),
                    (*data).num_samples as usize,
                    (*data).input_parameter_changes as *mut c_void,
                    (*data).output_parameter_changes as *mut c_void,
                    (*data).input_events as *mut c_void,
                    (*data).output_events as *mut c_void,
                );

                self.get_component().process_f64(process_data);
            }
        }

        ResultOk.into()
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }
}
