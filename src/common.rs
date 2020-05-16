use std::ffi::CStr;
use std::os::raw::{c_char, c_short, c_void};
use std::ptr::copy_nonoverlapping;

use widestring::U16CString;

use vst3_com::sys::GUID;
use vst3_sys::vst::{kFx, kVstAudioEffectClass, kVstComponentControllerClass};

use crate::{AudioProcessor, Component, EditController, HostApplication, ResultErr, ResultOk};

/// If the source &str is too long, it gets truncated to fit into the destination
pub(crate) unsafe fn strcpy(src: &str, dst: *mut c_char) {
    let mut src = src.to_string().into_bytes();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

/// If the source &str is too long, it gets truncated to fit into the destination
pub(crate) unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

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
    cid: &'static UID,
    cardinality: u32,
    category: &'static Category,
    name: &'static str,
    class_flags: u32,
    subcategories: &'static FxSubcategory,
    vendor: &'static str,
    version: &'static str,
    sdk_version: &'static str,
}

impl ClassInfo {
    pub fn get_cid(&self) -> &UID {
        &self.cid
    }

    pub fn get_info(&self) -> vst3_sys::base::PClassInfo {
        let mut info = vst3_sys::base::PClassInfo {
            cid: self.cid.to_guid(),
            cardinality: self.cardinality as i32,
            category: [0; 32],
            name: [0; 64],
        };

        unsafe {
            strcpy(&self.category.to_string(), info.category.as_mut_ptr());
            strcpy(self.name, info.name.as_mut_ptr());
        }

        info
    }

    pub fn get_info_2(&self) -> vst3_sys::base::PClassInfo2 {
        let mut info = vst3_sys::base::PClassInfo2 {
            cid: self.cid.to_guid(),
            cardinality: self.cardinality as i32,
            category: [0; 32],
            name: [0; 64],
            class_flags: 0,
            subcategories: [0; 128],
            vendor: [0; 64],
            version: [0; 64],
            sdk_version: [0; 64],
        };

        unsafe {
            strcpy(&self.category.to_string(), info.category.as_mut_ptr());
            strcpy(self.name, info.name.as_mut_ptr());
            strcpy(
                &self.subcategories.to_string(),
                info.subcategories.as_mut_ptr(),
            );
            strcpy(self.vendor, info.vendor.as_mut_ptr());
            strcpy(self.version, info.version.as_mut_ptr());
            strcpy(self.sdk_version, info.sdk_version.as_mut_ptr());
        }

        info
    }

    pub fn get_info_w(&self) -> vst3_sys::base::PClassInfoW {
        let mut info = vst3_sys::base::PClassInfoW {
            cid: self.cid.to_guid(),
            cardinality: self.cardinality as i32,
            category: [0; 32],
            name: [0; 64],
            class_flags: self.class_flags,
            subcategories: [0; 128],
            vendor: [0; 64],
            version: [0; 64],
            sdk_version: [0; 64],
        };

        unsafe {
            strcpy(&self.category.to_string(), info.category.as_mut_ptr());
            wstrcpy(self.name, info.name.as_mut_ptr());
            strcpy(
                &self.subcategories.to_string(),
                info.subcategories.as_mut_ptr(),
            );
            wstrcpy(self.vendor, info.vendor.as_mut_ptr());
            wstrcpy(self.version, info.version.as_mut_ptr());
            wstrcpy(self.sdk_version, info.sdk_version.as_mut_ptr());
        }

        info
    }
}

pub struct ClassInfoBuilder {
    cid: &'static UID,
    cardinality: u32,
    category: &'static Category,
    name: &'static str,
    class_flags: u32,
    subcategories: &'static FxSubcategory,
    vendor: &'static str,
    version: &'static str,
    sdk_version: &'static str,
}

impl ClassInfoBuilder {
    pub const fn new(cid: &'static UID) -> ClassInfoBuilder {
        ClassInfoBuilder {
            cid,
            name: "VST3",
            cardinality: MANY_INSTANCES,
            category: &Category::AudioEffect,
            class_flags: 0,
            subcategories: &FxSubcategory::NoSubcategory,
            vendor: "",
            version: "0.1.0",
            sdk_version: "VST 3.6.14",
        }
    }

    pub const fn name(mut self, name: &'static str) -> ClassInfoBuilder {
        self.name = name;
        self
    }

    pub const fn cardinality(mut self, cardinality: u32) -> ClassInfoBuilder {
        self.cardinality = cardinality;
        self
    }

    pub const fn category(mut self, category: &'static Category) -> ClassInfoBuilder {
        self.category = category;
        self
    }

    pub const fn class_flags(mut self, class_flags: u32) -> ClassInfoBuilder {
        self.class_flags = class_flags;
        self
    }

    pub const fn subcategories(
        mut self,
        subcategories: &'static FxSubcategory,
    ) -> ClassInfoBuilder {
        self.subcategories = subcategories;
        self
    }

    pub const fn vendor(mut self, vendor: &'static str) -> ClassInfoBuilder {
        self.vendor = vendor;
        self
    }

    pub const fn version(mut self, version: &'static str) -> ClassInfoBuilder {
        self.version = version;
        self
    }

    pub const fn sdk_version(mut self, sdk_version: &'static str) -> ClassInfoBuilder {
        self.sdk_version = sdk_version;
        self
    }

    pub const fn build(&self) -> ClassInfo {
        ClassInfo {
            cid: self.cid,
            cardinality: self.cardinality,
            category: self.category,
            name: self.name,
            class_flags: self.class_flags,
            subcategories: self.subcategories,
            vendor: self.vendor,
            version: self.version,
            sdk_version: self.sdk_version,
        }
    }
}
#[derive(Clone, Debug)]
pub struct UID([u32; 4]);
impl UID {
    pub const fn new(uid: [u32; 4]) -> Self {
        Self { 0: uid }
    }

    // todo: make it pub(crate)
    pub fn to_guid(&self) -> GUID {
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
