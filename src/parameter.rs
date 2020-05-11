use crate::AtomicFloat;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use vst3_sys::vst::ParameterFlags::{kCanAutomate, kIsBypass};
use vst3_sys::vst::{kRootUnitId, ParameterInfo};

pub(crate) struct ParamInfo {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) short_title: Option<String>,
    pub(crate) units: Option<String>,
    pub(crate) step_count: i32,
    pub(crate) default_normalized_value: f64,
    pub(crate) unit_id: i32,
    pub(crate) flags: i32,
}

pub struct Parameter {
    pub(crate) info: ParamInfo,
    pub(crate) value_normalized: AtomicFloat,
    pub(crate) get_param_string_by_value: fn(f64) -> String,
    pub(crate) get_param_value_by_string: fn(String) -> Result<f64, ParseFloatError>,
    pub(crate) normalized_param_to_plain: fn(f64) -> f64,
    pub(crate) plain_param_to_normalized: fn(f64) -> f64,
    pub(crate) get_param_normalized: fn(f64) -> f64,
    pub(crate) set_param_normalized: fn(f64) -> f64,
    pub(crate) processor_only: bool,
}

pub struct ParameterBuilder {
    id: u32,
    title: String,
    short_title: Option<String>,
    units: Option<String>,
    step_count: i32,
    default_normalized_value: f64,
    unit_id: i32,
    flags: i32,
    get_param_string_by_value: fn(f64) -> String,
    get_param_value_by_string: fn(String) -> Result<f64, ParseFloatError>,
    normalized_param_to_plain: fn(f64) -> f64,
    plain_param_to_normalized: fn(f64) -> f64,
    get_param_normalized: fn(f64) -> f64,
    set_param_normalized: fn(f64) -> f64,
    processor_only: bool,
}
impl ParameterBuilder {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            title: "".to_string(),
            short_title: None,
            units: None,
            step_count: 0,
            default_normalized_value: 0.0,
            unit_id: kRootUnitId,
            flags: kCanAutomate as i32,
            get_param_string_by_value: |v| format!("{:.2}", v),
            get_param_value_by_string: |s| s.parse::<f64>(),
            normalized_param_to_plain: |v| v,
            plain_param_to_normalized: |v| v,
            get_param_normalized: |v| v,
            set_param_normalized: |v| v,
            processor_only: false,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn units(mut self, units: &str) -> Self {
        self.units = Some(units.to_string());
        self
    }

    pub fn step_count(mut self, step_count: i32) -> Self {
        self.step_count = step_count;
        self
    }

    pub fn default_value_normalized(mut self, value: f64) -> Self {
        self.default_normalized_value = value;
        self
    }

    pub fn can_automate(mut self) -> Self {
        self.flags |= kCanAutomate as i32;
        self
    }

    pub fn is_bypass(mut self) -> Self {
        self.flags |= kIsBypass as i32;
        self
    }

    pub fn unit_id(mut self, unit_id: i32) -> Self {
        self.unit_id = unit_id;
        self
    }

    pub fn short_title(mut self, short_title: &str) -> Self {
        self.short_title = Some(short_title.to_string());
        self
    }

    pub fn get_param_string_by_value(
        mut self,
        get_param_string_by_value: fn(f64) -> String,
    ) -> Self {
        self.get_param_string_by_value = get_param_string_by_value;
        self
    }

    pub fn get_param_value_by_string(
        mut self,
        get_param_value_by_string: fn(String) -> Result<f64, ParseFloatError>,
    ) -> Self {
        self.get_param_value_by_string = get_param_value_by_string;
        self
    }

    pub fn normalized_param_to_plain(mut self, normalized_param_to_plain: fn(f64) -> f64) -> Self {
        self.normalized_param_to_plain = normalized_param_to_plain;
        self
    }

    pub fn plain_param_to_normalized(mut self, plain_param_to_normalized: fn(f64) -> f64) -> Self {
        self.plain_param_to_normalized = plain_param_to_normalized;
        self
    }

    pub fn get_param_normalized(mut self, get_param_normalized: fn(f64) -> f64) -> Self {
        self.get_param_normalized = get_param_normalized;
        self
    }

    pub fn set_param_normalized(mut self, set_param_normalized: fn(f64) -> f64) -> Self {
        self.set_param_normalized = set_param_normalized;
        self
    }

    pub fn processor_only(mut self) -> Self {
        self.processor_only = true;
        self
    }

    pub fn build(&self) -> Parameter {
        let param_info = ParamInfo {
            id: self.id,
            title: self.title.clone(),
            short_title: self.short_title.clone(),
            units: self.units.clone(),
            step_count: self.step_count,
            default_normalized_value: self.default_normalized_value,
            unit_id: self.unit_id,
            flags: self.flags,
        };
        Parameter {
            info: param_info,
            value_normalized: AtomicFloat::new(self.default_normalized_value),
            get_param_string_by_value: self.get_param_string_by_value,
            get_param_value_by_string: self.get_param_value_by_string,
            normalized_param_to_plain: self.normalized_param_to_plain,
            plain_param_to_normalized: self.plain_param_to_normalized,
            get_param_normalized: self.get_param_normalized,
            set_param_normalized: self.set_param_normalized,
            processor_only: self.processor_only,
        }
    }
}
