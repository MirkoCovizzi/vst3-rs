use crate::PluginResult::{ResultFalse, ResultOk};
use crate::{strcpy, wstrcpy, Parameter, PluginResult};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::num::ParseFloatError;
use std::sync::Arc;
use vst3_sys::vst::ParameterInfo;

// todo: Consider making the key usize?
//
// todo: Make parameter generic over real parameter field (example: I may want
// todo: a bool parameter or an Enum(String) parameter.
type ParamKey = u32;
pub struct Parameters {
    controller_params: RefCell<HashMap<ParamKey, Parameter>>,
    component_params: RefCell<HashMap<ParamKey, f64>>,
    order: RefCell<Vec<ParamKey>>,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            controller_params: RefCell::new(HashMap::new()),
            component_params: RefCell::new(HashMap::new()),
            order: RefCell::new(vec![]),
        }
    }

    pub fn remove_all(&self) {
        self.controller_params.borrow_mut().clear();
        self.order.borrow_mut().clear();
    }

    pub fn add_parameter(&self, parameter: Parameter) {
        let id = parameter.info.id;
        let val = parameter.info.default_normalized_value;
        let processor_only = parameter.processor_only;
        if !processor_only {
            self.controller_params.borrow_mut().insert(id, parameter);
        }
        self.component_params.borrow_mut().insert(id, val);
        self.order.borrow_mut().push(id);
    }

    pub fn get(&self, id: ParamKey) -> Option<f64> {
        match self.component_params.borrow().get(&id) {
            Some(val) => Some(*val),
            None => None,
        }
    }

    pub fn set(&self, id: ParamKey, value: f64) -> PluginResult {
        match self.component_params.borrow_mut().get_mut(&id) {
            Some(val) => {
                *val = value;
                ResultOk
            }
            None => ResultFalse,
        }
    }

    pub(crate) fn get_parameter_count(&self) -> usize {
        self.controller_params.borrow().len()
    }

    pub(crate) fn get_parameter_info(&self, index: u32) -> Result<ParameterInfo, PluginResult> {
        let key = self.order.borrow()[index as usize];
        if let Some(param_info) = &self.controller_params.borrow().get(&key) {
            let info = &param_info.info;
            let mut parameter_info = ParameterInfo {
                id: index,
                title: [0; 128],
                short_title: [0; 128],
                units: [0; 128],
                step_count: info.step_count,
                default_normalized_value: info.default_normalized_value,
                unit_id: info.unit_id,
                flags: info.flags,
            };
            unsafe { wstrcpy(&info.title, parameter_info.title.as_mut_ptr()) };
            if let Some(short_title) = &info.short_title {
                unsafe { wstrcpy(short_title, parameter_info.short_title.as_mut_ptr()) };
            }
            if let Some(units) = &info.units {
                unsafe { wstrcpy(units, parameter_info.units.as_mut_ptr()) };
            }
            return Ok(parameter_info);
        }
        Err(ResultFalse)
    }

    pub(crate) fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
    ) -> Result<String, PluginResult> {
        if let Some(param) = &self.controller_params.borrow().get(&id) {
            let get_param_string_by_value = param.get_param_string_by_value;
            return Ok(get_param_string_by_value(value_normalized));
        }
        Err(ResultFalse)
    }

    pub(crate) fn get_param_value_by_string(&self, id: ParamKey, string: String) -> Option<f64> {
        if let Some(param) = self.controller_params.borrow().get(&id) {
            let get_param_value_by_string = param.get_param_value_by_string;
            return match get_param_value_by_string(string) {
                Ok(val) => Some(val),
                Err(_) => None,
            };
        }
        None
    }

    pub(crate) fn get_param_normalized(&self, id: ParamKey) -> Option<f64> {
        if let Some(param) = self.controller_params.borrow().get(&id) {
            let get_param_normalized = param.get_param_normalized;
            return Some(get_param_normalized(param.value_normalized));
        }
        None
    }

    pub(crate) fn set_param_normalized(&self, id: ParamKey, value: f64) -> PluginResult {
        if let Some(param) = &mut self.controller_params.borrow_mut().get_mut(&id) {
            let set_param_normalized = param.set_param_normalized;
            param.value_normalized = set_param_normalized(value);
            return ResultOk;
        }
        ResultFalse
    }

    pub(crate) fn normalized_param_to_plain(
        &self,
        id: ParamKey,
        value_normalized: f64,
    ) -> Option<f64> {
        if let Some(param) = &self.controller_params.borrow().get(&id) {
            let normalized_param_to_plain = param.normalized_param_to_plain;
            return Some(normalized_param_to_plain(value_normalized));
        }
        None
    }

    pub(crate) fn plain_param_to_normalized(&self, id: ParamKey, plain_value: f64) -> Option<f64> {
        if let Some(param) = &self.controller_params.borrow().get(&id) {
            let plain_param_to_normalized = param.plain_param_to_normalized;
            return Some(plain_param_to_normalized(plain_value));
        }
        None
    }

    pub(crate) fn get_param_order(&self) -> &RefCell<Vec<ParamKey>> {
        &self.order
    }
}
