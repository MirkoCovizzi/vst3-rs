use std::fmt::{Debug, Formatter};
use std::os::raw::c_void;

use vst3_com::ComPtr;
use vst3_sys::base::kResultOk;
use vst3_sys::vst::{IParamValueQueue, IParameterChanges};

use crate::ResultOk::ResOk;
use crate::{ResultErr, ResultOk, Unknown, UID};

pub struct ParameterChanges {
    inner: ComPtr<dyn IParameterChanges>,
}

impl Unknown for ParameterChanges {
    const IID: UID = UID::new([0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IParameterChanges> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}

impl ParameterChanges {
    pub fn get_parameter_count(&self) -> usize {
        unsafe { self.inner.get_parameter_count() as usize }
    }

    pub fn get_parameter_data(&self, index: usize) -> Option<ParamValueQueue> {
        let mut ptr;
        unsafe {
            ptr = self.inner.get_parameter_data(index as i32);
        }
        ParamValueQueue::from_raw(ptr)
    }

    pub fn add_parameter_data(&self, id: &usize, index: &mut usize) -> Option<ParamValueQueue> {
        if *id > u32::MAX as usize {
            log::trace!(
                "ParameterChanges::add_parameter_data(): value is too big! {}usize > {}u32",
                *id,
                u32::MAX
            );
            return None;
        }
        let mut ptr;
        unsafe {
            ptr = self.inner.add_parameter_data(
                id as *const usize as *const u32,
                index as *mut usize as *mut i32,
            );
        }
        ParamValueQueue::from_raw(ptr)
    }
}

impl Debug for ParameterChanges {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParameterChanges").finish()
    }
}

pub struct ParamValueQueue {
    inner: ComPtr<dyn IParamValueQueue>,
}

impl ParamValueQueue {
    pub fn from_raw(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IParamValueQueue> = ComPtr::new(ptr);
            Some(Self { inner: ptr })
        }
    }

    pub fn get_parameter_id(&self) -> usize {
        unsafe { self.inner.get_parameter_id() as usize }
    }

    pub fn get_point_count(&self) -> i32 {
        unsafe { self.inner.get_point_count() as i32 }
    }

    // todo: fix function signature to be more flexible
    pub fn get_point(&self, index: i32) -> Result<ParamValuePoint, ResultErr> {
        let mut value = 0.0;
        let mut sample_offset = 0i32;
        unsafe {
            match self
                .inner
                .get_point(index, &mut sample_offset as *mut _, &mut value as *mut _)
            {
                r if r == ResOk.into() => Ok(ParamValuePoint {
                    value,
                    sample_offset,
                }),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    // todo: fix function signature to be more flexible
    pub fn add_point(
        &self,
        sample_offset: i32,
        value: f64,
        index: &mut i32,
    ) -> Result<ResultOk, ResultErr> {
        unsafe {
            match self
                .inner
                .add_point(sample_offset, value, index as *mut i32)
            {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }
}

impl Debug for ParamValueQueue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamValueQueue").finish()
    }
}

pub struct ParamValuePoint {
    pub sample_offset: i32,
    pub value: f64,
}
