use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;

use vst3_com::IID;
use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{
    AudioBusBuffers, BusDirections, BusTypes, IAudioProcessor, IComponent, IoModes, MediaTypes,
};
use vst3_sys::VST3;

use crate::ResultErr::{InvalidArgument, NotImplemented, ResultFalse, InternalError};
use crate::ResultOk::ResOk;
use crate::{
    wstrcpy, AudioProcessor, ClassInfo, HostApplication, PluginBase, ProcessData, ProcessMode,
    ProcessSetup, ResultErr, ResultOk, Stream, SymbolicSampleSize, Unknown, UID,
};
use std::sync::Mutex;

pub enum IoMode {
    Simple,
    Advanced,
    OfflineProcessing,
}

impl IoMode {
    pub(crate) fn is_valid(mode: i32) -> bool {
        // todo: find better way to do this
        if mode != IoModes::kSimple as i32 && mode != IoModes::kAdvanced as i32 && mode != IoModes::kOfflineProcessing as i32 {
            false
        } else {
            true
        }
    }
}

impl From<i32> for IoMode {
    fn from(io_mode: i32) -> Self {
        match io_mode {
            m if m == IoModes::kSimple as i32 => IoMode::Simple,
            m if m == IoModes::kAdvanced as i32 => IoMode::Advanced,
            m if m == IoModes::kOfflineProcessing as i32 => IoMode::OfflineProcessing,
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

impl MediaType {
    pub(crate) fn is_valid(t: i32) -> bool {
        // todo: find better way to do this
        if t != MediaTypes::kAudio as i32 && t != MediaTypes::kEvent as i32 && t != MediaTypes::kNumMediaTypes as i32 {
            false
        } else {
            true
        }
    }
}

impl From<i32> for MediaType {
    fn from(media_type: i32) -> Self {
        match media_type {
            t if t == MediaTypes::kAudio as i32 => MediaType::Audio,
            t if t == MediaTypes::kEvent as i32 => MediaType::Event,
            t if t == MediaTypes::kNumMediaTypes as i32 => MediaType::NumMediaTypes,
            _ => unreachable!(),
        }
    }
}

impl From<MediaType> for i32 {
    fn from(media_type: MediaType) -> Self {
        match media_type {
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

impl BusDirection {
    pub(crate) fn is_valid(dir: i32) -> bool {
        // todo: find better way to do this
        if dir != BusDirections::kInput as i32 && dir != BusDirections::kOutput as i32 {
            false
        } else {
            true
        }
    }
}

impl From<i32> for BusDirection {
    fn from(dir: i32) -> Self {
        match dir {
            d if d == BusDirections::kInput as i32 => BusDirection::Input,
            d if d == BusDirections::kOutput as i32 => BusDirection::Output,
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

impl From<i32> for BusType {
    fn from(bus_type: i32) -> Self {
        match bus_type {
            t if t == BusTypes::kMain as i32 => BusType::Main,
            t if t == BusTypes::kAux as i32 => BusType::Aux,
            _ => unreachable!()
        }
    }
}

impl From<BusType> for i32 {
    fn from(bus_type: BusType) -> Self {
        match bus_type {
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

impl BusInfo {
    pub fn get_info(&self) -> vst3_sys::vst::BusInfo {
        let mut info = vst3_sys::vst::BusInfo {
            media_type: self.media_type.clone().into(),
            direction: self.direction.clone().into(),
            channel_count: self.channel_count,
            name: [0; 128],
            bus_type: self.bus_type.clone().into(),
            flags: self.flags,
        };

        unsafe {
            wstrcpy(&self.name, info.name.as_mut_ptr());
        }

        info
    }
}

pub struct RoutingInfo {
    pub media_type: MediaType,
    pub bus_index: i32,
    pub channel: i32,
}

impl RoutingInfo {
    pub fn get_info(&self) -> vst3_sys::vst::RoutingInfo {
        vst3_sys::vst::RoutingInfo {
            media_type: self.media_type.clone().into(),
            bus_index: self.bus_index,
            channel: self.channel
        }
    }
}

pub trait Component: PluginBase {
    fn get_controller_class_id(&self) -> Result<&UID, ResultErr>;
    fn set_io_mode(&self, mode: &IoMode) -> Result<ResultOk, ResultErr>;
    fn get_bus_count(&self, media_type: &MediaType, dir: &BusDirection) -> Result<usize, ResultErr>;
    fn get_bus_info(
        &self,
        media_type: &MediaType,
        dir: &BusDirection,
        index: usize,
    ) -> Result<BusInfo, ResultErr>;
    fn get_routing_info(&self) -> Result<(&RoutingInfo, &RoutingInfo), ResultErr>;
    fn activate_bus(
        &mut self,
        media_type: &MediaType,
        dir: &BusDirection,
        index: usize,
        state: bool,
    ) -> Result<ResultOk, ResultErr>;
    fn set_active(&self, state: bool) -> Result<ResultOk, ResultErr>;
    fn set_state(&mut self, state: &Stream) -> Result<ResultOk, ResultErr>;
    fn get_state(&self, state: &Stream) -> Result<ResultOk, ResultErr>;
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
    fn get_controller_class_id(&self) -> Result<&UID, ResultErr> {
        unimplemented!()
    }

    fn set_io_mode(&self, _mode: &IoMode) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_bus_count(
        &self,
        _media_type: &MediaType,
        _dir: &BusDirection,
    ) -> Result<usize, ResultErr> {
        unimplemented!()
    }

    fn get_bus_info(
        &self,
        _media_type: &MediaType,
        _dir: &BusDirection,
        _index: usize,
    ) -> Result<BusInfo, ResultErr> {
        unimplemented!()
    }

    fn get_routing_info(&self) -> Result<(&RoutingInfo, &RoutingInfo), ResultErr> {
        unimplemented!()
    }

    fn activate_bus(
        &mut self,
        _media_type: &MediaType,
        _dir: &BusDirection,
        _index: usize,
        _state: bool,
    ) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_active(&self, _state: bool) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_state(&mut self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_state(&self, _state: &Stream) -> Result<ResultOk, ResultErr> {
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
            if !IoMode::is_valid(mode) {
                return InvalidArgument.into()
            }
            return match component.set_io_mode(&IoMode::from(mode)) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            if !MediaType::is_valid(type_) {
                return InvalidArgument.into()
            }
            if !BusDirection::is_valid(dir) {
                return InvalidArgument.into()
            }
            return match component.get_bus_count(&MediaType::from(type_), &BusDirection::from(dir))
            {
                Ok(bus_count) => {
                    if bus_count > i32::MAX as usize {
                        InternalError.into()
                    } else {
                        bus_count as i32
                    }
                },
                Err(_) => 0,
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_bus_info(
        &self,
        type_: i32,
        dir: i32,
        index: i32,
        info: *mut vst3_sys::vst::BusInfo,
    ) -> i32 {
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            if !MediaType::is_valid(type_) {
                return InvalidArgument.into()
            }
            if !BusDirection::is_valid(dir) {
                return InvalidArgument.into()
            }
            if index < 0 {
                return InvalidArgument.into()
            }
            return match component.get_bus_info(
                &MediaType::from(type_),
                &BusDirection::from(dir),
                index as usize,
            ) {
                Ok(bus_info) => {
                    *info = bus_info.get_info();
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
                    *in_info = routing_info.0.get_info();
                    *out_info = routing_info.1.get_info();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn activate_bus(&self, type_: i32, dir: i32, index: i32, state: u8) -> i32 {
        let state = if state != 0 { true } else { false };
        if let Some(component) = self.get_plugin_base().lock().unwrap().as_component() {
            if !MediaType::is_valid(type_) {
                return InvalidArgument.into()
            }
            if !BusDirection::is_valid(dir) {
                return InvalidArgument.into()
            }
            if index < 0 {
                return InvalidArgument.into()
            }
            return match component.activate_bus(
                &MediaType::from(type_),
                &BusDirection::from(dir),
                index as usize,
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
                return match component.set_state(&*state) {
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
                return match component.get_state(&*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }
}
