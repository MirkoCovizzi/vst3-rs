use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::ComPtr;
use vst3_sys::vst::{IHostApplication, IMessage};

use crate::ResultErr::ResultFalse;
use crate::ResultOk::ResOk;
use crate::{ResultErr, Unknown, UID};

pub struct HostApplication {
    inner: ComPtr<dyn IHostApplication>,
}

impl Unknown for HostApplication {
    const IID: UID = UID::new([0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IHostApplication> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}

impl HostApplication {
    pub fn get_name(&self) -> Result<String, ResultErr> {
        unsafe {
            let mut name = [0; 128];
            match self.inner.get_name(name.as_mut_ptr()) {
                r if r == ResOk.into() => {
                    Ok(String::from_utf16_lossy(&name[..]).replace("\u{0}", ""))
                }
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn create_instance<T: Unknown>(&self, class_id: UID) -> Result<T, ResultErr> {
        let cid = class_id.to_guid();
        let iid = T::IID.to_guid();
        let mut obj_ptr = null_mut();
        unsafe {
            match self
                .inner
                .create_instance(cid, iid, &mut obj_ptr as *mut *mut c_void)
            {
                r if r == ResOk.into() => match T::from_raw(obj_ptr) {
                    Some(obj) => Ok(*obj),
                    None => Err(ResultFalse),
                },
                r => Err(ResultErr::from(r)),
            }
        }
    }
}

pub struct Message {
    inner: ComPtr<dyn IMessage>,
}

impl Unknown for Message {
    const IID: UID = UID::new([0x936F033B, 0xC6C047DB, 0xBB0882F8, 0x13C1E613]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IMessage> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}
