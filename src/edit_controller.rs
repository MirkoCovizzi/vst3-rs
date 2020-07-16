use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;
use std::slice;
use std::sync::Mutex;
use widestring::{U16CStr, U16CString};

use vst3_com::ComPtr;
use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{IComponentHandler, IEditController, IMidiMapping, IUnitInfo};
use vst3_sys::VST3;

use crate::plug_view::{PlugView, VST3PlugView};
use crate::unknown::ResultErr::ResultFalse;
use crate::unknown::{ResultErr, Unknown};
use crate::ResultErr::{InternalError, InvalidArgument, NotImplemented};
use crate::ResultOk::ResOk;
use crate::{
    wstrcpy, ClassInfo, ClassInfoBuilder, HostApplication, Offset0, Offset1, Offset2,
    ParameterInfo, PluginBase, ResultOk, Stream, UnitInfo, UID,
};

pub struct ComponentHandler {
    inner: ComPtr<dyn IComponentHandler>,
}

impl Unknown for ComponentHandler {
    const IID: UID = UID::new([0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IComponentHandler> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}

impl ComponentHandler {
    pub fn begin_edit(&self, id: u32) -> Result<ResultOk, ResultErr> {
        unsafe {
            match self.inner.begin_edit(id) {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn perform_edit(&self, id: u32, value_normalized: f64) -> Result<ResultOk, ResultErr> {
        unsafe {
            match self.inner.perform_edit(id, value_normalized) {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn end_edit(&self, id: u32) -> Result<ResultOk, ResultErr> {
        unsafe {
            match self.inner.end_edit(id) {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn restart_component(&self, flags: i32) -> Result<ResultOk, ResultErr> {
        unsafe {
            match self.inner.restart_component(flags) {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }
}

pub trait EditController: PluginBase {
    fn set_component_state(&mut self, state: &Stream) -> Result<ResultOk, ResultErr>;
    fn set_state(&mut self, state: &Stream) -> Result<ResultOk, ResultErr>;
    fn get_state(&self, state: &Stream) -> Result<ResultOk, ResultErr>;
    fn get_parameter_count(&self) -> Result<usize, ResultErr>;
    fn get_parameter_info(&self, index: usize) -> Result<&ParameterInfo, ResultErr>;
    fn get_param_string_by_value(&self, id: usize, value: f64) -> Result<String, ResultErr>;
    fn get_param_value_by_string(&self, id: usize, string: &str) -> Result<f64, ResultErr>;
    fn normalized_param_to_plain(&self, id: usize, value: f64) -> Result<f64, ResultErr>;
    fn plain_param_to_normalized(&self, id: usize, value: f64) -> Result<f64, ResultErr>;
    fn get_param_normalized(&self, id: usize) -> Result<f64, ResultErr>;
    fn set_param_normalized(&mut self, id: usize, value: f64) -> Result<ResultOk, ResultErr>;
    fn set_component_handler(&self, handler: ComponentHandler) -> Result<ResultOk, ResultErr>;
    fn create_view(&mut self) -> Option<&mut Box<dyn PlugView>>;
}

struct DummyEditController {}

impl Default for DummyEditController {
    fn default() -> Self {
        Self {}
    }
}

impl PluginBase for DummyEditController {
    fn initialize(&mut self, _context: HostApplication) -> bool {
        unimplemented!()
    }

    fn terminate(&mut self) -> bool {
        unimplemented!()
    }
}

impl EditController for DummyEditController {
    fn set_component_state(&mut self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_state(&mut self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_state(&self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn get_parameter_count(&self) -> Result<usize, ResultErr> {
        unimplemented!()
    }

    fn get_parameter_info(&self, _param_index: usize) -> Result<&ParameterInfo, ResultErr> {
        unimplemented!()
    }

    fn get_param_string_by_value(
        &self,
        _id: usize,
        _value_normalized: f64,
    ) -> Result<String, ResultErr> {
        unimplemented!()
    }

    fn get_param_value_by_string(&self, _id: usize, _string: &str) -> Result<f64, ResultErr> {
        unimplemented!()
    }

    fn normalized_param_to_plain(&self, _id: usize, _value: f64) -> Result<f64, ResultErr> {
        unimplemented!()
    }

    fn plain_param_to_normalized(&self, _id: usize, _plain: f64) -> Result<f64, ResultErr> {
        unimplemented!()
    }

    fn get_param_normalized(&self, _id: usize) -> Result<f64, ResultErr> {
        unimplemented!()
    }

    fn set_param_normalized(&mut self, _id: usize, _value: f64) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn set_component_handler(&self, _handler: ComponentHandler) -> Result<ResultOk, ResultErr> {
        unimplemented!()
    }

    fn create_view(&mut self) -> Option<&mut Box<dyn PlugView>> {
        unimplemented!()
    }
}

#[repr(C)]
pub(crate) struct VST3EditController {
    __ieditcontrollervptr: *const <dyn IEditController as vst3_com::ComInterface>::VTable,
    __iunitinfovptr: *const <dyn IUnitInfo as vst3_com::ComInterface>::VTable,
    __imidimappingvptr: *const <dyn IMidiMapping as vst3_com::ComInterface>::VTable,
    __refcnt: std::cell::Cell<u32>,
    inner: Mutex<Box<dyn PluginBase>>,
}

impl VST3EditController {
    fn allocate(inner: Mutex<Box<dyn PluginBase>>) -> Box<VST3EditController> {
        let ieditcontroller_vtable = <dyn IEditController as ::vst3_com::ProductionComInterface<
            VST3EditController,
        >>::vtable::<Offset0>();
        let __ieditcontrollervptr = Box::into_raw(Box::new(ieditcontroller_vtable));
        let iunitinfo_vtable = <dyn IUnitInfo as ::vst3_com::ProductionComInterface<
            VST3EditController,
        >>::vtable::<Offset1>();
        let __iunitinfovptr = Box::into_raw(Box::new(iunitinfo_vtable));
        let imidimapping_vtable = <dyn IMidiMapping as ::vst3_com::ProductionComInterface<
            VST3EditController,
        >>::vtable::<Offset2>();
        let __imidimappingvptr = Box::into_raw(Box::new(imidimapping_vtable));
        let out = VST3EditController {
            __ieditcontrollervptr,
            __iunitinfovptr,
            __imidimappingvptr,
            __refcnt: std::cell::Cell::new(1),
            inner,
        };
        Box::new(out)
    }
}

unsafe impl vst3_com::CoClass for VST3EditController {}

impl vst3_com::interfaces::IUnknown for VST3EditController {
    unsafe fn query_interface(
        &self,
        riid: *const vst3_com::sys::IID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> vst3_com::sys::HRESULT {
        let riid = &*riid;
        if riid == &vst3_com::interfaces::iunknown::IID_IUNKNOWN {
            *ppv = &self.__ieditcontrollervptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IEditController as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid)
        {
            *ppv = &self.__ieditcontrollervptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IUnitInfo as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid) {
            *ppv = &self.__iunitinfovptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IMidiMapping as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid) {
            *ppv = &self.__imidimappingvptr as *const _ as *mut std::ffi::c_void;
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
                self.__ieditcontrollervptr
                    as *mut <dyn IEditController as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(
                self.__iunitinfovptr as *mut <dyn IUnitInfo as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(
                self.__imidimappingvptr
                    as *mut <dyn IMidiMapping as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(self as *const _ as *mut VST3EditController);
        }
        __refcnt
    }
}

impl VST3EditController {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(Mutex::new(DummyEditController::new()))
    }

    pub(crate) fn set_plugin_base(&mut self, plugin_base: Box<dyn PluginBase>) {
        self.inner = Mutex::new(plugin_base)
    }

    pub(crate) fn get_plugin_base(&self) -> &Mutex<Box<dyn PluginBase>> {
        &self.inner
    }
}

impl IPluginBase for VST3EditController {
    unsafe fn initialize(&self, context: *mut c_void) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(context) = HostApplication::from_raw(context) {
                    return if plugin_base.initialize(*context) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = InvalidArgument.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3EditController: initialize: panic");
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
                log::error!("VST3EditController: terminate: panic");
                *ret.lock().unwrap()
            }
        }
    }
}

impl IEditController for VST3EditController {
    unsafe fn set_component_state(&self, state: *mut c_void) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if let Some(state) = Stream::from_raw(state) {
                return match edit_controller.set_component_state(&*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }

    unsafe fn set_state(&self, state: *mut c_void) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if let Some(state) = Stream::from_raw(state) {
                return match edit_controller.set_state(&*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }

    unsafe fn get_state(&self, state: *mut c_void) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if let Some(state) = Stream::from_raw(state) {
                return match edit_controller.get_state(&*state) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.get_parameter_count() {
                Ok(count) => {
                    if count > i32::MAX as usize {
                        log::trace!(
                            "get_parameter_count(): returned value is too big! {}usize > {}i32",
                            count,
                            i32::MAX
                        );
                        return InternalError.into();
                    }
                    count as i32
                }
                Err(_) => InternalError.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_parameter_info(
        &self,
        param_index: i32,
        info: *mut vst3_sys::vst::ParameterInfo,
    ) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if param_index < 0 {
                return InvalidArgument.into();
            }
            return match edit_controller.get_parameter_info(param_index as usize) {
                Ok(param_info) => {
                    *info = param_info.get_info();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut i16,
    ) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.get_param_string_by_value(id as usize, value_normalized) {
                Ok(param_string) => {
                    wstrcpy(&param_string, string);
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const i16,
        value_normalized: *mut f64,
    ) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if string.is_null() {
                return InvalidArgument.into();
            }
            let string = U16CStr::from_ptr_str(string as *const u16).to_string_lossy();
            return match edit_controller.get_param_value_by_string(id as usize, &string) {
                Ok(value) => {
                    *value_normalized = value;
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.normalized_param_to_plain(id as usize, value_normalized) {
                Ok(plain) => plain,
                Err(_) => 0.0,
            };
        }
        0.0
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.plain_param_to_normalized(id as usize, plain_value) {
                Ok(normalized) => normalized,
                Err(_) => 0.0,
            };
        }
        0.0
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.get_param_normalized(id as usize) {
                Ok(param_normalized) => param_normalized,
                Err(_) => 0.0,
            };
        }
        0.0
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            return match edit_controller.set_param_normalized(id as usize, value) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_component_handler(&self, handler: *mut c_void) -> i32 {
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if let Some(handler) = ComponentHandler::from_raw(handler) {
                return match edit_controller.set_component_handler(*handler) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }

    unsafe fn create_view(&self, _: *const i8) -> *mut c_void {
        /*
        if let Some(edit_controller) = self.get_plugin_base().lock().unwrap().as_edit_controller() {
            if name.is_null() {
                return null_mut();
            }

            // For now the only name is "editor" so we can remove this until the API supports other
            // types.
            // let name = CStr::from_ptr(name).to_string_lossy().to_string();
            return match edit_controller.create_view() {
                Some(plug_view) => {
                    let mut view = VST3PlugView::new();
                    view.set_plug_view(plug_view);
                    Box::into_raw(view) as *mut c_void
                }
                None => null_mut(),
            };
        }
        */
        null_mut()
    }
}

pub trait MidiMapping: EditController {
    fn get_midi_controller_assignment(
        &self,
        bus_index: i32,
        channel: i16,
        midi_controller_number: i16,
    ) -> Result<u32, ResultErr>;
}

impl IMidiMapping for VST3EditController {
    unsafe fn get_midi_controller_assignment(
        &self,
        bus_index: i32,
        channel: i16,
        midi_controller_number: i16,
        id: *mut u32,
    ) -> i32 {
        if let Some(midi_controller) = self.get_plugin_base().lock().unwrap().as_midi_mapping() {
            return match midi_controller.get_midi_controller_assignment(
                bus_index,
                channel,
                midi_controller_number,
            ) {
                Ok(assignment_id) => {
                    *id = assignment_id;
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }

        NotImplemented.into()
    }
}
