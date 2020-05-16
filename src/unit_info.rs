use std::os::raw::c_void;
use vst3_sys::vst::{IUnitInfo, ProgramListInfo};

use crate::ResultErr::{InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    wstrcpy, EditController, ProgramList, ResultErr, ResultOk, Stream, Unit, Unknown,
    VST3EditController,
};
use std::ffi::CStr;

pub trait UnitInfo: EditController {
    fn get_unit_count(&self) -> Result<i32, ResultErr>;
    fn get_unit_info(&self, unit_index: i32) -> Result<Unit, ResultErr>;
    fn get_program_list_count(&self) -> Result<i32, ResultErr>;
    fn get_program_list_info(&self, list_index: i32) -> Result<ProgramList, ResultErr>;
    fn get_program_name(&self, list_id: i32, program_index: i32) -> Result<String, ResultErr>;
    fn get_program_info(
        &self,
        list_id: i32,
        program_index: i32,
        attribute_id: String,
    ) -> Result<String, ResultErr>;
    fn has_program_pitch_names(&self, id: i32, index: i32) -> Result<ResultOk, ResultErr>;
    fn get_program_pitch_name(&self, id: i32, index: i32, pitch: i16) -> Result<String, ResultErr>;
    fn get_selected_unit(&self) -> Result<i32, ResultErr>;
    fn select_unit(&self, id: i32) -> Result<ResultOk, ResultErr>;
    fn get_unit_by_bus(
        &self,
        type_: i32,
        dir: i32,
        bus_index: i32,
        channel: i32,
    ) -> Result<i32, ResultErr>;
    fn set_unit_program_data(
        &self,
        list_or_unit: i32,
        program_index: i32,
        data: Stream,
    ) -> Result<ResultOk, ResultErr>;
}

impl IUnitInfo for VST3EditController {
    unsafe fn get_unit_count(&self) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_unit_count() {
                Ok(unit_count) => unit_count,
                Err(_) => 0,
            };
        }
        0
    }

    unsafe fn get_unit_info(&self, unit_index: i32, info: *mut vst3_sys::vst::UnitInfo) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_unit_info(unit_index) {
                Ok(unit) => {
                    *info = unit.get_info();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_program_list_count() {
                Ok(program_list_count) => program_list_count,
                Err(_) => 0,
            };
        }
        0
    }

    unsafe fn get_program_list_info(
        &self,
        list_index: i32,
        info: *mut vst3_sys::vst::ProgramListInfo,
    ) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_program_list_info(list_index) {
                Ok(program_list) => {
                    *info = program_list.get_info();
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_program_name(&self, list_id: i32, program_index: i32, name: *mut u16) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_program_name(list_id, program_index) {
                Ok(program_name) => {
                    wstrcpy(&program_name, name as *mut i16);
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_program_info(
        &self,
        list_id: i32,
        program_index: i32,
        attribute_id: *const u8,
        attribute_value: *mut u16,
    ) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            if attribute_id.is_null() {
                return InvalidArgument.into();
            }
            let attribute_id = CStr::from_ptr(attribute_id as *const i8)
                .to_string_lossy()
                .to_string();
            return match unit_info.get_program_info(list_id, program_index, attribute_id) {
                Ok(attr_value) => {
                    wstrcpy(&attr_value, attribute_value as *mut i16);
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn has_program_pitch_names(&self, id: i32, index: i32) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.has_program_pitch_names(id, index) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_program_pitch_name(
        &self,
        id: i32,
        index: i32,
        pitch: i16,
        name: *mut u16,
    ) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_program_pitch_name(id, index, pitch) {
                Ok(pitch_name) => {
                    wstrcpy(&pitch_name, name as *mut i16);
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_selected_unit() {
                Ok(num) => num,
                Err(r) => 0,
            };
        }
        0
    }

    unsafe fn select_unit(&self, id: i32) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.select_unit(id) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_unit_by_bus(
        &self,
        type_: i32,
        dir: i32,
        bus_index: i32,
        channel: i32,
        unit_id: *mut i32,
    ) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            return match unit_info.get_unit_by_bus(type_, dir, bus_index, channel) {
                Ok(id) => {
                    *unit_id = id;
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_unit_program_data(
        &self,
        list_or_unit: i32,
        program_index: i32,
        data: *mut c_void,
    ) -> i32 {
        if let Some(unit_info) = self.get_plugin_base().lock().unwrap().as_unit_info() {
            if let Some(data) = Stream::from_raw(data) {
                return match unit_info.set_unit_program_data(list_or_unit, program_index, *data) {
                    Ok(r) => r.into(),
                    Err(r) => r.into(),
                };
            }
            return InvalidArgument.into();
        }
        NotImplemented.into()
    }
}
