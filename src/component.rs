use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;

use vst3_com::ProductionComInterface;
use vst3_com::IID;
use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{
    AudioBusBuffers, BusDirections, BusTypes, IAudioProcessor, IComponent, IoModes, MediaTypes,
};
use vst3_sys::VST3;

use crate::ResultErr::{InternalError, InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    register_panic_msg, wstrcpy, AudioProcessor, ClassInfo, HostApplication, PluginBase,
    ProcessData, ProcessMode, ProcessSetup, ResultErr, ResultOk, Stream, SymbolicSampleSize,
    Unknown, UID,
};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{Arc, Mutex};

pub enum IoMode {
    Simple,
    Advanced,
    OfflineProcessing,
}

impl IoMode {
    pub(crate) fn is_valid(mode: i32) -> bool {
        // todo: find better way to do this
        if mode != IoModes::kSimple as i32
            && mode != IoModes::kAdvanced as i32
            && mode != IoModes::kOfflineProcessing as i32
        {
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
        if t != MediaTypes::kAudio as i32
            && t != MediaTypes::kEvent as i32
            && t != MediaTypes::kNumMediaTypes as i32
        {
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
            _ => unreachable!(),
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
            channel: self.channel,
        }
    }
}

pub trait Component: PluginBase {
    fn get_controller_class_id(&self) -> Option<&UID>;
    fn set_io_mode(&self, mode: &IoMode) -> bool;
    fn get_bus_count(&self, media_type: &MediaType, dir: &BusDirection) -> usize;
    fn get_bus_info(
        &self,
        media_type: &MediaType,
        dir: &BusDirection,
        index: usize,
    ) -> Option<BusInfo>;
    fn get_routing_info(&self) -> Option<(&RoutingInfo, &RoutingInfo)>;
    fn activate_bus(
        &mut self,
        media_type: &MediaType,
        dir: &BusDirection,
        index: usize,
        state: bool,
    ) -> bool;
    fn set_active(&self, state: bool) -> bool;
    fn set_state(&mut self, state: &Stream) -> bool;
    fn get_state(&self, state: &Stream) -> bool;
}

struct DummyComponent {}

impl Default for DummyComponent {
    fn default() -> Self {
        Self {}
    }
}

impl PluginBase for DummyComponent {
    fn initialize(&mut self, _context: HostApplication) -> bool {
        unimplemented!()
    }

    fn terminate(&mut self) -> bool {
        unimplemented!()
    }
}

impl Component for DummyComponent {
    fn get_controller_class_id(&self) -> Option<&UID> {
        unimplemented!()
    }

    fn set_io_mode(&self, _mode: &IoMode) -> bool {
        unimplemented!()
    }

    fn get_bus_count(&self, _media_type: &MediaType, _dir: &BusDirection) -> usize {
        unimplemented!()
    }

    fn get_bus_info(
        &self,
        _media_type: &MediaType,
        _dir: &BusDirection,
        _index: usize,
    ) -> Option<BusInfo> {
        unimplemented!()
    }

    fn get_routing_info(&self) -> Option<(&RoutingInfo, &RoutingInfo)> {
        unimplemented!()
    }

    fn activate_bus(
        &mut self,
        _media_type: &MediaType,
        _dir: &BusDirection,
        _index: usize,
        _state: bool,
    ) -> bool {
        unimplemented!()
    }

    fn set_active(&self, _state: bool) -> bool {
        unimplemented!()
    }

    fn set_state(&mut self, _state: &Stream) -> bool {
        unimplemented!()
    }

    fn get_state(&self, _state: &Stream) -> bool {
        unimplemented!()
    }
}

#[repr(C)]
pub(crate) struct VST3Component {
    __icomponentvptr: *const <dyn IComponent as vst3_com::ComInterface>::VTable,
    __iaudioprocessorvptr: *const <dyn IAudioProcessor as vst3_com::ComInterface>::VTable,
    __refcnt: std::cell::Cell<u32>,
    inner: Mutex<Box<dyn PluginBase>>,
}

impl VST3Component {
    fn allocate(inner: Mutex<Box<dyn PluginBase>>) -> Box<VST3Component> {
        let icomponent_vtable = <dyn IComponent as ::vst3_com::ProductionComInterface<
            VST3Component,
        >>::vtable::<vst3_com::Offset0>();
        let __icomponentvptr = Box::into_raw(Box::new(icomponent_vtable));
        let iaudioprocessor_vtable = <dyn IAudioProcessor as ::vst3_com::ProductionComInterface<
            VST3Component,
        >>::vtable::<vst3_com::Offset1>();
        let __iaudioprocessorvptr = Box::into_raw(Box::new(iaudioprocessor_vtable));
        let out = VST3Component {
            __icomponentvptr,
            __iaudioprocessorvptr,
            __refcnt: std::cell::Cell::new(1),
            inner,
        };
        Box::new(out)
    }
}

unsafe impl vst3_com::CoClass for VST3Component {}

impl vst3_com::interfaces::IUnknown for VST3Component {
    unsafe fn query_interface(
        &self,
        riid: *const vst3_com::sys::IID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> vst3_com::sys::HRESULT {
        let riid = &*riid;
        if riid == &vst3_com::interfaces::iunknown::IID_IUNKNOWN {
            *ppv = &self.__icomponentvptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IComponent as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid) {
            *ppv = &self.__icomponentvptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IAudioProcessor as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid)
        {
            *ppv = &self.__iaudioprocessorvptr as *const _ as *mut std::ffi::c_void;
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
                self.__icomponentvptr as *mut <dyn IComponent as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(
                self.__iaudioprocessorvptr
                    as *mut <dyn IAudioProcessor as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(self as *const _ as *mut VST3Component);
        }
        __refcnt
    }
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
        let mutex_plugin_base = self.get_plugin_base();
        // Creating a return value wrapped in a Mutex for the catch_unwind closure
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            // Checking to see if plugin_base has been poisoned or not
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                // Checking to see if context is valid or not (i.e. it's a null pointer)
                if let Some(context) = HostApplication::from_raw(context) {
                    // If initialize returns true, return kOk to Host, else return kResultFalse
                    return if plugin_base.initialize(*context) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                // Return kInvalidArgument to inform the Host that the pointer to context is null
                return *ret.lock().unwrap() = InvalidArgument.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                register_panic_msg("VST3Component: initialize: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn terminate(&self) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                return if plugin_base.terminate() {
                    *ret.lock().unwrap() = ResOk.into()
                } else {
                    *ret.lock().unwrap() = ResultFalse.into()
                };
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: terminate: panic");
                *ret.lock().unwrap()
            }
        }
    }
}

impl IComponent for VST3Component {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    return match component.get_controller_class_id() {
                        Some(controller_class_id) => {
                            *tuid = controller_class_id.to_guid();
                            *ret.lock().unwrap() = ResOk.into()
                        }
                        None => *ret.lock().unwrap() = ResultFalse.into(),
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_controller_class_id: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn set_io_mode(&self, mode: i32) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if !IoMode::is_valid(mode) {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return if component.set_io_mode(&IoMode::from(mode)) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: set_io_mode: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(0);
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if !MediaType::is_valid(type_) || !BusDirection::is_valid(dir) {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    let count =
                        component.get_bus_count(&MediaType::from(type_), &BusDirection::from(dir));
                    if count > i32::MAX as usize {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "VST3Component: get_bus_count: returned value is too big! \
                                    {}usize > {}i32",
                            count,
                            i32::MAX
                        );
                        return;
                    } else {
                        return *ret.lock().unwrap() = count as i32;
                    }
                }
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_bus_count: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_bus_info(
        &self,
        type_: i32,
        dir: i32,
        index: i32,
        info: *mut vst3_sys::vst::BusInfo,
    ) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if !MediaType::is_valid(type_) || !BusDirection::is_valid(dir) || index < 0 {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return match component.get_bus_info(
                        &MediaType::from(type_),
                        &BusDirection::from(dir),
                        index as usize,
                    ) {
                        Some(bus_info) => {
                            *info = bus_info.get_info();
                            *ret.lock().unwrap() = ResOk.into()
                        }
                        None => *ret.lock().unwrap() = ResultFalse.into(),
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_bus_info: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_routing_info(
        &self,
        in_info: *mut vst3_sys::vst::RoutingInfo,
        out_info: *mut vst3_sys::vst::RoutingInfo,
    ) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    return match component.get_routing_info() {
                        Some(routing_info) => {
                            *in_info = routing_info.0.get_info();
                            *out_info = routing_info.1.get_info();
                            *ret.lock().unwrap() = ResOk.into()
                        }
                        None => *ret.lock().unwrap() = ResultFalse.into(),
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_routing_info: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn activate_bus(&self, type_: i32, dir: i32, index: i32, state: u8) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if !MediaType::is_valid(type_) || !BusDirection::is_valid(dir) || index < 0 {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    let state = if state != 0 { true } else { false };
                    return if component.activate_bus(
                        &MediaType::from(type_),
                        &BusDirection::from(dir),
                        index as usize,
                        state,
                    ) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: activate_bus: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn set_active(&self, state: u8) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    let state = if state != 0 { true } else { false };
                    return if component.set_active(state) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: set_active: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn set_state(&self, state: *mut c_void) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if let Some(state) = Stream::from_raw(state) {
                        return if component.set_state(&*state) {
                            *ret.lock().unwrap() = ResOk.into()
                        } else {
                            *ret.lock().unwrap() = ResultFalse.into()
                        };
                    }
                    return *ret.lock().unwrap() = InvalidArgument.into();
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: set_state: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_state(&self, state: *mut c_void) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(component) = plugin_base.as_component() {
                    if let Some(state) = Stream::from_raw(state) {
                        return if component.get_state(&*state) {
                            *ret.lock().unwrap() = ResOk.into()
                        } else {
                            *ret.lock().unwrap() = ResultFalse.into()
                        };
                    }
                    return *ret.lock().unwrap() = InvalidArgument.into();
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_state: panic");
                *ret.lock().unwrap()
            }
        }
    }
}
