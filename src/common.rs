use std::ffi::CStr;

use crate::{AudioProcessor, Component, EditController, HostApplication, ResultErr, ResultOk};
use std::ops::Add;
use vst3_com::sys::GUID;
use vst3_sys::vst::{kFx, kVstAudioEffectClass, kVstComponentControllerClass};

#[derive(Clone)]
pub enum Category {
    AudioEffect,
    ComponentController,
}
impl ToString for Category {
    fn to_string(&self) -> String {
        match self {
            Category::AudioEffect => unsafe {
                CStr::from_ptr(kVstAudioEffectClass)
                    .to_string_lossy()
                    .to_string()
            },
            Category::ComponentController => unsafe {
                CStr::from_ptr(kVstComponentControllerClass)
                    .to_string_lossy()
                    .to_string()
            },
        }
    }
}

#[derive(Clone)]
pub enum FxSubcategory {
    NoSubcategory,
    Fx,
}
impl ToString for FxSubcategory {
    fn to_string(&self) -> String {
        match self {
            FxSubcategory::NoSubcategory => "".to_string(),
            FxSubcategory::Fx => unsafe { CStr::from_ptr(kFx).to_string_lossy().to_string() },
        }
    }
}

const MANY_INSTANCES: u32 = 0x7FFFFFFF;

pub struct ClassInfo {
    pub cid: UID,
    pub cardinality: u32,
    pub category: Category,
    pub name: String,
    pub class_flags: u32,
    pub subcategories: FxSubcategory,
    pub vendor: String,
    pub version: String,
    pub sdk_version: String,
}

pub struct ClassInfoBuilder {
    cid: UID,
    cardinality: u32,
    category: Category,
    name: String,
    class_flags: u32,
    subcategories: FxSubcategory,
    vendor: String,
    version: String,
    sdk_version: String,
}

impl ClassInfoBuilder {
    pub fn new(cid: UID) -> ClassInfoBuilder {
        ClassInfoBuilder {
            cid,
            name: "VST3".to_string(),
            cardinality: MANY_INSTANCES,
            category: Category::AudioEffect,
            class_flags: 0,
            subcategories: FxSubcategory::NoSubcategory,
            vendor: String::new(),
            version: "0.1.0".to_string(),
            sdk_version: "VST 3.6.14".to_string(),
        }
    }

    pub fn name(mut self, name: &str) -> ClassInfoBuilder {
        self.name = name.to_string();
        self
    }

    pub fn cardinality(mut self, cardinality: u32) -> ClassInfoBuilder {
        self.cardinality = cardinality;
        self
    }

    pub fn category(mut self, category: Category) -> ClassInfoBuilder {
        self.category = category;
        self
    }

    pub fn class_flags(mut self, class_flags: u32) -> ClassInfoBuilder {
        self.class_flags = class_flags;
        self
    }

    pub fn subcategories(mut self, subcategories: FxSubcategory) -> ClassInfoBuilder {
        self.subcategories = subcategories;
        self
    }

    pub fn vendor(mut self, vendor: &str) -> ClassInfoBuilder {
        self.vendor = vendor.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> ClassInfoBuilder {
        self.version = version.to_string();
        self
    }

    pub fn sdk_version(mut self, sdk_version: &str) -> ClassInfoBuilder {
        self.sdk_version = sdk_version.to_string();
        self
    }

    pub fn build(&self) -> ClassInfo {
        ClassInfo {
            cid: self.cid.clone(),
            cardinality: self.cardinality,
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
#[derive(Clone, Debug)]
pub struct UID([u32; 4]);
impl UID {
    pub const fn new(uid: [u32; 4]) -> Self {
        Self { 0: uid }
    }
    pub(crate) fn to_guid(&self) -> GUID {
        let mut tuid: [u8; 16] = [0; 16];
        for i in 0..4 {
            let big_e = self.0[i].to_be_bytes();
            for k in 0..4 {
                tuid[i * 4 + k] = big_e[k];
            }
        }

        #[cfg(target_os = "windows")]
        {
            tuid.swap(0, 3);
            tuid.swap(1, 2);
            tuid.swap(4, 5);
            tuid.swap(6, 7);
        }

        GUID { data: tuid }
    }
    pub(crate) fn auto_inc(mut self) -> Self {
        if self.0[3] == u32::MAX {
            self.0[3] = 0;
        } else {
            self.0[3] += 1;
        }
        self
    }

    pub(crate) fn auto_dec(mut self) -> Self {
        if self.0[3] == 0 {
            self.0[3] = u32::MAX;
        } else {
            self.0[3] -= 1;
        }
        self
    }
}
