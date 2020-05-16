use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;

use vst3_com::IID;
use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{
    AudioBusBuffers, BusDirections, BusTypes, IAudioProcessor, IComponent, IoModes, MediaTypes,
};
use vst3_sys::VST3;

use crate::ResultErr::{InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    wstrcpy, AudioProcessor, ClassInfo, HostApplication, PluginBase, ProcessData, ProcessMode,
    ProcessSetup, ResultErr, ResultOk, Stream, SymbolicSampleSize, Unknown, UID,
};
use std::cell::RefCell;
use std::sync::Mutex;

pub enum IoMode {
    Simple,
    Advanced,
    OfflineProcessing,
}

impl From<i32> for IoMode {
    fn from(io_mode: i32) -> Self {
        match io_mode {
            io_mode if io_mode == IoModes::kSimple as i32 => IoMode::Simple,
            io_mode if io_mode == IoModes::kAdvanced as i32 => IoMode::Advanced,
            io_mode if io_mode == IoModes::kOfflineProcessing as i32 => IoMode::OfflineProcessing,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub enum MediaType {
    Audio,
    Event,
    NumMediaTypes,
}

impl From<i32> for MediaType {
    fn from(type_: i32) -> Self {
        match type_ {
            type_ if type_ == MediaTypes::kAudio as i32 => MediaType::Audio,
            type_ if type_ == MediaTypes::kEvent as i32 => MediaType::Event,
            type_ if type_ == MediaTypes::kNumMediaTypes as i32 => MediaType::NumMediaTypes,
            _ => unreachable!(),
        }
    }
}

impl From<MediaType> for i32 {
    fn from(type_: MediaType) -> Self {
        match type_ {
            MediaType::Audio => MediaTypes::kAudio as i32,
            MediaType::Event => MediaTypes::kEvent as i32,
            MediaType::NumMediaTypes => MediaTypes::kNumMediaTypes as i32,
        }
    }
}

#[derive(Clone)]
pub enum BusDirection {
    Input,
    Output,
}

impl From<i32> for BusDirection {
    fn from(dir: i32) -> Self {
        match dir {
            dir if dir == BusDirections::kInput as i32 => BusDirection::Input,
            dir if dir == BusDirections::kOutput as i32 => BusDirection::Output,
            _ => unreachable!(),
        }
    }
}

impl From<BusDirection> for i32 {
    fn from(dir: BusDirection) -> Self {
        match dir {
            BusDirection::Input => BusDirections::kInput as i32,
            BusDirection::Output => BusDirections::kOutput as i32,
        }
    }
}

#[derive(Clone)]
pub enum BusType {
    Main,
    Aux,
}

impl From<BusType> for i32 {
    fn from(type_: BusType) -> Self {
        match type_ {
            BusType::Main => BusTypes::kMain as i32,
            BusType::Aux => BusTypes::kAux as i32,
        }
    }
}

pub struct BusInfo {
    pub media_type: MediaType,
    pub direction: BusDirection,
    pub channel_count: i32,
    pub name: String,
    pub bus_type: BusType,
    pub flags: u32,
}

impl From<BusInfo> for vst3_sys::vst::BusInfo {
    fn from(bus_info: BusInfo) -> Self {
        let mut info = vst3_sys::vst::BusInfo {
            media_type: bus_info.media_type.into(),
            direction: bus_info.direction.into(),
            channel_count: bus_info.channel_count,
            name: [0; 128],
            bus_type: bus_info.bus_type.into(),
            flags: bus_info.flags,
        };
        unsafe {
            wstrcpy(&bus_info.name, info.name.as_mut_ptr());
        }
        info
    }
}

pub struct RoutingInfo {
    pub media_type: MediaType,
    pub bus_index: i32,
    pub channel: i32,
}

impl From<RoutingInfo> for vst3_sys::vst::RoutingInfo {
    fn from(routing_info: RoutingInfo) -> Self {
        vst3_sys::vst::RoutingInfo {
            media_type: routing_info.media_type.into(),
            bus_index: routing_info.bus_index,
            channel: routing_info.channel,
        }
    }
}

pub trait Component: PluginBase {
    fn get_controller_class_id(&self) -> Result<UID, ResultErr>;
    fn set_io_mode(&self, mode: IoMode) -> Result<ResultOk, ResultErr>;
    fn get_bus_count(&self, type_: &MediaType, dir: &BusDirection) -> Result<i32, ResultErr>;
    fn get_bus_info(
        &self,
        type_: &MediaType,
        dir: &BusDirection,
        index: i32,
    ) -> Result<BusInfo, ResultErr>;
    fn get_routing_info(&self) -> Result<(RoutingInfo, RoutingInfo), ResultErr>;
    fn activate_bus(
        &mut self,
        type_: &MediaType,
        dir: &BusDirection,
        index: i32,
        state: bool,
    ) -> Result<ResultOk, ResultErr>;
    fn set_active(&self, state: bool) -> Result<ResultOk, ResultErr>;
    fn set_state(&mut self, state: Stream) -> Result<ResultOk, ResultErr>;
    fn get_state(&self, state: Stream) -> Result<ResultOk, ResultErr>;
}

struct DummyComponent {}

impl Default for DummyComponent {
    fn default() -> Self {
        Self {}
    }
}

impl PluginBase for DummyComponent {
    fn initialize(&mut self, _context: HostApplication) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn terminate(&mut self) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }
}

impl Component for DummyComponent {
    fn get_controller_class_id(&self) -> Result<UID, ResultErr> {
        unimplemented!()
    }

    fn set_io_mode(&self, _mode: IoMode) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_bus_count(&self, _type_: &MediaType, _dir: &BusDirection) -> Result<i32, ResultErr> {
        unimplemented!()
    }

    fn get_bus_info(
        &self,
        _type_: &MediaType,
        _dir: &BusDirection,
        _index: i32,
    ) -> Result<BusInfo, ResultErr> {
        unimplemented!()
    }

    fn get_routing_info(&self) -> Result<(RoutingInfo, RoutingInfo), ResultErr> {
        unimplemented!()
    }

    fn activate_bus(
        &mut self,
        _type_: &MediaType,
        _dir: &BusDirection,
        _index: i32,
        _state: bool,
    ) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_active(&self, _state: bool) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_state(&mut self, _state: Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_state(&self, _state: Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }
}

#[VST3(implements(IComponent, IAudioProcessor))]
pub(crate) struct VST3Component {
    inner: Mutex<Box<dyn PluginBase>>,
}

impl VST3Component {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(Mutex::new(DummyComponent::new()))
    }

    pub(crate) fn set_plugin_base(&mut self, plugin_base: Box<dyn PluginBase>) {
        self.inner = Mutex::new(plugin_base)
    }

    pub(crate) fn get_plugin_base(&self) -> &Mutex<Box<dyn PluginBase>> {
        &self.inner
    }
}

impl IPluginBase for VST3Component {
    unsafe fn initialize(&self, context: *mut c_void) -> i32 {
        if let Some(context) = HostApplication::from_raw(context) {
            return match self.get_plugin_base().lock().unwrap().initialize(*context) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        InvalidArgument.into()
    }

    unsafe fn terminate(&self) -> i32 {
        match self.get_plugin_base().lock().unwrap().terminate() {
            Ok(r) => r.into(),
            Err(r) => r.into(),
        }
    }
}

impl IComponent for VST3Component {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            return match component.get_controller_class_id() {
                Ok(controller_class_id) => {
                    *tuid = controller_class_id.to_guid();
                    ResOk.into()
                }
                Err(result) => result.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_io_mode(&self, mode: i32) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            return match component.set_io_mode(IoMode::from(mode)) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            return match component.get_bus_count(&MediaType::from(type_), &BusDirection::from(dir))
            {
                Ok(bus_count) => bus_count,
                Err(_) => 0,
            };
        }
        0
    }

    unsafe fn get_bus_info(
        &self,
        type_: i32,
        dir: i32,
        index: i32,
        info: *mut vst3_sys::vst::BusInfo,
    ) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            return match component.get_bus_info(
                &MediaType::from(type_),
                &BusDirection::from(dir),
                index,
            ) {
                Ok(bus_info) => {
                    *info = bus_info.into();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_routing_info(
        &self,
        in_info: *mut vst3_sys::vst::RoutingInfo,
        out_info: *mut vst3_sys::vst::RoutingInfo,
    ) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            return match component.get_routing_info() {
                Ok(routing_info) => {
                    *in_info = routing_info.0.into();
                    *out_info = routing_info.1.into();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn activate_bus(&self, type_: i32, dir: i32, index: i32, state: u8) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            let state = if state != 0 { true } else { false };
            return match component.activate_bus(
                &MediaType::from(type_),
                &BusDirection::from(dir),
                index,
                state,
            ) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_active(&self, state: u8) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            let state = if state != 0 { true } else { false };
            return match component.set_active(state) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_state(&self, state: *mut c_void) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            if let Some(state) = Stream::from_raw(state) {
                return match component.set_state(*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }

    unsafe fn get_state(&self, state: *mut c_void) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            if let Some(state) = Stream::from_raw(state) {
                return match component.get_state(*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }
}
