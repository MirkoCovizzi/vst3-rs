use std::collections::HashMap;

use crate::ParameterFlag::CanAutomate;
use crate::ResultErr::InvalidArgument;
use crate::{wstrcpy, ResultErr, ROOT_UNIT_ID};

pub struct ParameterInfo {
    pub id: i32,
    pub title: String,
    pub short_title: Option<String>,
    pub units: Option<String>,
    pub step_count: i32,
    pub default_normalized_value: f64,
    pub unit_id: i32,
    pub flags: i32,
}

impl ParameterInfo {
    pub fn get_parameter_info(&self) -> vst3_sys::vst::ParameterInfo {
        let mut p_info = vst3_sys::vst::ParameterInfo {
            id: self.id,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: self.step_count,
            default_normalized_value: self.default_normalized_value,
            unit_id: self.unit_id,
            flags: self.flags,
        };

        unsafe {
            wstrcpy(&self.title, p_info.title.as_mut_ptr());
            if let Some(short_title) = &self.short_title {
                wstrcpy(short_title, p_info.short_title.as_mut_ptr());
            }
            if let Some(units) = &self.units {
                wstrcpy(units, p_info.units.as_mut_ptr());
            }
        }

        p_info
    }
}

pub enum ParameterFlag {
    NoFlags = vst3_sys::vst::ParameterFlags::kNoFlags as isize,
    CanAutomate = vst3_sys::vst::ParameterFlags::kCanAutomate as isize,
    IsReadOnly = vst3_sys::vst::ParameterFlags::kIsReadOnly as isize,
    IsWrapAround = vst3_sys::vst::ParameterFlags::kIsWrapAround as isize,
    IsList = vst3_sys::vst::ParameterFlags::kIsList as isize,
    IsProgramChange = vst3_sys::vst::ParameterFlags::kIsProgramChange as isize,
    IsBypass = vst3_sys::vst::ParameterFlags::kIsBypass as isize,
}

pub struct ParameterInfoBuilder {
    id: u32,
    title: String,
    short_title: Option<String>,
    units: Option<String>,
    step_count: i32,
    default_normalized_value: f64,
    unit_id: i32,
    flags: i32,
}

impl ParameterInfoBuilder {
    pub fn new(title: &str, id: u32) -> Self {
        Self {
            id,
            title: title.to_string(),
            short_title: None,
            units: None,
            step_count: 0,
            default_normalized_value: 0.0,
            unit_id: ROOT_UNIT_ID,
            flags: CanAutomate as i32,
        }
    }

    pub fn short_title(mut self, short_title: &str) -> Self {
        self.short_title = Some(short_title.to_string());
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

    pub fn default_normalized_value(mut self, val: f64) -> Self {
        self.default_normalized_value = val;
        self
    }

    pub fn unit_id(mut self, unit_id: i32) -> Self {
        self.unit_id = unit_id;
        self
    }

    pub fn flags(mut self, flags: i32) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(&self) -> ParameterInfo {
        let mut info = ParameterInfo {
            id: self.id as i32,
            title: self.title.clone(),
            short_title: self.short_title.clone(),
            units: self.units.clone(),
            step_count: self.step_count,
            default_normalized_value: self.default_normalized_value,
            unit_id: self.unit_id,
            flags: self.flags,
        };

        info
    }
}

pub trait Parameter {
    fn get_info(&self) -> &ParameterInfo;
    fn get_info_mut(&mut self) -> &mut ParameterInfo;
    fn set_unit_id(&mut self, id: i32);
    fn get_unit_id(&self) -> i32;
    fn set_normalized(&mut self, v: f64);
    fn get_normalized(&self) -> f64;
    fn set_precision(&mut self, val: usize);
    fn get_precision(&self) -> usize;

    fn to_string(&self, value_normalized: f64) -> String {
        return if self.get_info().step_count == 1 {
            if value_normalized > 0.5 {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        } else {
            format!("{:.*}", self.get_precision(), value_normalized)
        };
    }

    fn from_string(&self, string: String) -> Result<f64, ResultErr> {
        match string.parse::<f64>() {
            Ok(val) => Ok(val),
            Err(_) => Err(InvalidArgument),
        }
    }

    fn to_plain(&self, value_normalized: f64) -> f64 {
        value_normalized
    }

    fn to_normalized(&self, plain_value: f64) -> f64 {
        plain_value
    }
}

pub struct BaseParameter {
    info: ParameterInfo,
    value_normalized: f64,
    precision: usize,
}

impl BaseParameter {
    pub fn new(info: ParameterInfo) -> Box<Self> {
        Box::new(Self {
            value_normalized: info.default_normalized_value.clone(),
            info,
            precision: 4,
        })
    }
}

impl Parameter for BaseParameter {
    fn get_info(&self) -> &ParameterInfo {
        &self.info
    }

    fn get_info_mut(&mut self) -> &mut ParameterInfo {
        &mut self.info
    }

    fn set_unit_id(&mut self, id: i32) {
        self.info.id = id;
    }

    fn get_unit_id(&self) -> i32 {
        self.info.id
    }

    fn set_normalized(&mut self, norm_value: f64) {
        let mut norm_value = norm_value;
        if norm_value > 1.0 {
            norm_value = 1.0
        } else if norm_value < 0.0 {
            norm_value = 0.0
        }

        self.value_normalized = norm_value
    }

    fn get_normalized(&self) -> f64 {
        self.value_normalized
    }

    fn set_precision(&mut self, val: usize) {
        self.precision = val;
    }

    fn get_precision(&self) -> usize {
        self.precision
    }
}

pub struct ParameterContainer {
    params: Vec<Box<dyn Parameter>>,
    id_to_index: HashMap<u32, usize>,
}

impl ParameterContainer {
    pub fn new() -> Self {
        Self {
            params: vec![],
            id_to_index: HashMap::new(),
        }
    }

    pub fn add_parameter(&mut self, p: Box<dyn Parameter>) {
        self.id_to_index
            .insert(p.get_info().id as u32, self.params.len());
        self.params.push(p);
    }

    pub fn get_parameter_count(&self) -> usize {
        self.params.len()
    }

    pub fn get_parameter_by_index(&self, index: usize) -> Option<&Box<dyn Parameter>> {
        self.params.get(index)
    }

    pub fn remove_all(&mut self) {
        self.params.clear();
        self.id_to_index.clear();
    }

    pub fn get_parameter(&self, tag: u32) -> Option<&Box<dyn Parameter>> {
        match self.id_to_index.get(&tag) {
            Some(index) => self.params.get(*index),
            None => None,
        }
    }

    pub fn get_parameter_mut(&mut self, tag: u32) -> Option<&mut Box<dyn Parameter>> {
        match self.id_to_index.get(&tag) {
            Some(index) => self.params.get_mut(*index),
            None => None,
        }
    }
}
