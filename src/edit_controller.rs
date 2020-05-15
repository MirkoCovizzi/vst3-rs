use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;
use std::sync::Mutex;
use std::ffi::{CStr, CString};
use std::slice;
use widestring::U16CString;

use vst3_com::ComPtr;
use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{IComponentHandler, IEditController, IUnitInfo, IMidiMapping};
use vst3_sys::VST3;

use crate::plug_view::{PlugView, VST3PlugView};
use crate::unknown::ResultErr::ResultFalse;
use crate::unknown::{ResultErr, Unknown};
use crate::ResultErr::{InvalidArgument, NotImplemented};
use crate::ResultOk::ResOk;
use crate::{
    wstrcpy, ClassInfo, ClassInfoBuilder, HostApplication, ParameterInfo, PluginBase, ResultOk,
    Stream, UnitInfo, UID,
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
    fn as_unit_info(&mut self) -> Option<&mut dyn UnitInfo> {
        None
    }

    fn as_midi_mapping(&mut self) -> Option<&mut dyn MidiMapping> {
        None
    }

    fn set_component_state(&mut self, state: Stream) -> Result<ResultOk, ResultErr>;
    fn set_state(&self, state: Stream) -> Result<ResultOk, ResultErr>;
    fn get_state(&self, state: Stream) -> Result<ResultOk, ResultErr>;
    fn get_parameter_count(&self) -> Result<i32, ResultErr>;
    fn get_parameter_info(&self, param_index: i32) -> Result<&ParameterInfo, ResultErr>;
    fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
    ) -> Result<String, ResultErr>;
    fn get_param_value_by_string(&self, id: u32, string: String) -> Result<f64, ResultErr>;
    fn normalized_param_to_plain(&self, id: u32, value: f64) -> Result<f64, ResultErr>;
    fn plain_param_to_normalized(&self, id: u32, plain: f64) -> Result<f64, ResultErr>;
    fn get_param_normalized(&self, id: u32) -> Result<f64, ResultErr>;
    fn set_param_normalized(&mut self, id: u32, value: f64) -> Result<ResultOk, ResultErr>;
    fn set_component_handler(&self, handler: ComponentHandler) -> Result<ResultOk, ResultErr>;
    fn create_view(&self, name: String) -> Result<Box<dyn PlugView>, ResultErr>;
}

#[VST3(implements(IEditController, IUnitInfo, IMidiMapping))]
pub(crate) struct VST3EditController {
    inner: *mut c_void,
}

impl VST3EditController {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(null_mut())
    }

    pub(crate) fn set_edit_controller(&mut self, edit_controller: *mut c_void) {
        self.inner = edit_controller
    }

    #[allow(clippy::borrowed_box)]
    pub(crate) unsafe fn get_edit_controller(&self) -> &Mutex<Box<dyn EditController>> {
        *(self.inner as *mut &Mutex<Box<dyn EditController>>)
    }
}

impl IPluginBase for VST3EditController {
    unsafe fn initialize(&self, context: *mut c_void) -> i32 {
        if let Some(context) = HostApplication::from_raw(context) {
            return match self
                .get_edit_controller()
                .lock()
                .unwrap()
                .initialize(*context)
            {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        InvalidArgument.into()
    }

    unsafe fn terminate(&self) -> i32 {
        match self.get_edit_controller().lock().unwrap().terminate() {
            Ok(r) => r.into(),
            Err(r) => r.into(),
        }
    }
}

impl IEditController for VST3EditController {
    unsafe fn set_component_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = Stream::from_raw(state) {
            return match self
                .get_edit_controller()
                .lock()
                .unwrap()
                .set_component_state(*state)
            {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        return InvalidArgument.into();
    }

    unsafe fn set_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = Stream::from_raw(state) {
            return match self.get_edit_controller().lock().unwrap().set_state(*state) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        return InvalidArgument.into();
    }

    unsafe fn get_state(&self, state: *mut c_void) -> i32 {
        if let Some(state) = Stream::from_raw(state) {
            return match self.get_edit_controller().lock().unwrap().get_state(*state) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        return InvalidArgument.into();
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .get_parameter_count()
        {
            Ok(count) => count,
            Err(_) => 0,
        };
    }

    unsafe fn get_parameter_info(
        &self,
        param_index: i32,
        info: *mut vst3_sys::vst::ParameterInfo,
    ) -> i32 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .get_parameter_info(param_index)
        {
            Ok(param_info) => {
                *info = param_info.get_parameter_info();
                ResOk.into()
            }
            Err(r) => r.into(),
        };
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut i16,
    ) -> i32 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .get_param_string_by_value(id, value_normalized)
        {
            Ok(param_string) => {
                wstrcpy(&param_string, string);
                ResOk.into()
            }
            Err(r) => r.into(),
        };
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const i16,
        value_normalized: *mut f64,
    ) -> i32 {
        let string = U16CString::from_ptr_str(string as *const u16).to_string_lossy();
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .get_param_value_by_string(id, string)
        {
            Ok(value) => {
                *value_normalized = value;
                ResOk.into()
            }
            Err(r) => r.into(),
        };
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .normalized_param_to_plain(id, value_normalized)
        {
            Ok(plain) => plain,
            Err(_) => 0.0,
        };
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .plain_param_to_normalized(id, plain_value)
        {
            Ok(normalized) => normalized,
            Err(_) => 0.0,
        };
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .get_param_normalized(id)
        {
            Ok(param_normalized) => param_normalized,
            Err(_) => 0.0,
        };
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> i32 {
        return match self
            .get_edit_controller()
            .lock()
            .unwrap()
            .set_param_normalized(id, value)
        {
            Ok(r) => r.into(),
            Err(r) => r.into(),
        };
    }

    unsafe fn set_component_handler(&self, handler: *mut c_void) -> i32 {
        if let Some(handler) = ComponentHandler::from_raw(handler) {
            return match self
                .get_edit_controller()
                .lock()
                .unwrap()
                .set_component_handler(*handler)
            {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        return InvalidArgument.into();
    }

    unsafe fn create_view(&self, name: *const i8) -> *mut c_void {
        if name.is_null() {
            return null_mut();
        }

        let name = CStr::from_ptr(name).to_string_lossy().to_string();
        return match self.get_edit_controller().lock().unwrap().create_view(name) {
            Ok(plug_view) => {
                let plug_view = Box::into_raw(Box::new(plug_view)) as *mut _;
                let mut view = VST3PlugView::new();
                view.set_plug_view(plug_view);
                Box::into_raw(view) as *mut c_void
            }
            Err(r) => null_mut(),
        };
    }
}

pub trait MidiMapping: EditController {
    fn get_midi_controller_assignment(&self, bus_index: i32, channel: i16, midi_controller_number: i16) -> Result<u32, ResultErr>;
}

impl IMidiMapping for VST3EditController {
    unsafe fn get_midi_controller_assignment(&self, bus_index: i32, channel: i16, midi_controller_number: i16, id: *mut u32) -> i32 {
        if let Some(midi_controller) = self.get_edit_controller().lock().unwrap().as_midi_mapping() {
            return match midi_controller.get_midi_controller_assignment(bus_index, channel, midi_controller_number) {
                Ok(assignment_id) => {
                    *id = assignment_id;
                    ResOk.into()
                },
                Err(r) => r.into()
            }
        }

        NotImplemented.into()
    }
}
