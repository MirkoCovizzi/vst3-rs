use std::os::raw::c_void;

use crate::UID;

pub trait Unknown {
    const IID: UID;
    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>>;
}

pub enum ResultOk {
    ResOk,
    ResTrue,
}

impl From<ResultOk> for i32 {
    fn from(result: ResultOk) -> Self {
        match result {
            ResultOk::ResOk => vst3_sys::base::kResultOk,
            ResultOk::ResTrue => vst3_sys::base::kResultTrue,
        }
    }
}

impl From<i32> for ResultOk {
    fn from(result: i32) -> Self {
        match result {
            r if r == vst3_sys::base::kResultOk => ResultOk::ResOk,
            r if r == vst3_sys::base::kResultTrue => ResultOk::ResTrue,
            _ => {
                log::trace!("Unreachable ResultOk! {}", result);
                ResultOk::ResOk
            }
        }
    }
}

pub enum ResultErr {
    NoInterface,
    ResultFalse,
    InvalidArgument,
    NotImplemented,
    InternalError,
    NotInitialized,
    OutOfMemory,
}

impl From<ResultErr> for i32 {
    fn from(result: ResultErr) -> Self {
        match result {
            ResultErr::NoInterface => vst3_sys::base::kNoInterface,
            ResultErr::ResultFalse => vst3_sys::base::kResultFalse,
            ResultErr::InvalidArgument => vst3_sys::base::kInvalidArgument,
            ResultErr::NotImplemented => vst3_sys::base::kNotImplemented,
            ResultErr::InternalError => vst3_sys::base::kInternalError,
            ResultErr::NotInitialized => vst3_sys::base::kNotInitialized,
            ResultErr::OutOfMemory => vst3_sys::base::kOutOfMemory,
        }
    }
}

impl From<i32> for ResultErr {
    fn from(result: i32) -> Self {
        match result {
            r if r == vst3_sys::base::kNoInterface => ResultErr::NoInterface,
            r if r == vst3_sys::base::kResultFalse => ResultErr::ResultFalse,
            r if r == vst3_sys::base::kInvalidArgument => ResultErr::InvalidArgument,
            r if r == vst3_sys::base::kNotImplemented => ResultErr::NotImplemented,
            r if r == vst3_sys::base::kInternalError => ResultErr::InternalError,
            r if r == vst3_sys::base::kNotInitialized => ResultErr::NotInitialized,
            r if r == vst3_sys::base::kOutOfMemory => ResultErr::OutOfMemory,
            _ => {
                log::trace!("Unreachable ResultErr! {}", result);
                ResultErr::ResultFalse
            }
        }
    }
}
