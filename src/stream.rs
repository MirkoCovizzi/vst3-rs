use std::os::raw::c_void;

use vst3_com::ComPtr;
use vst3_sys::base::IBStream;

use crate::ResultOk::ResOk;
use crate::{ResultErr, ResultOk, Unknown, UID};

pub enum SeekMode {
    SeekSet,
    SeekCurrent,
    SeekEnd,
}

impl From<SeekMode> for i32 {
    fn from(mode: SeekMode) -> Self {
        match mode {
            SeekMode::SeekSet => vst3_sys::base::kIBSeekSet as i32,
            SeekMode::SeekCurrent => vst3_sys::base::kIBSeekCur as i32,
            SeekMode::SeekEnd => vst3_sys::base::kIBSeekEnd as i32,
        }
    }
}

pub struct Stream {
    inner: ComPtr<dyn IBStream>,
}

impl Unknown for Stream {
    const IID: UID = UID::new([0xC3BF6EA2, 0x30994752, 0x9B6BF990, 0x1EE33E9B]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IBStream> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}

impl Stream {
    pub fn read<T>(&self) -> Result<T, ResultErr> {
        let mut num_bytes_read = 0;
        let mut saved_value: T = unsafe { std::mem::zeroed() };
        let value_ptr = &mut saved_value as *mut T as *mut c_void;
        unsafe {
            match self.inner.read(
                value_ptr,
                std::mem::size_of::<T>() as i32,
                &mut num_bytes_read,
            ) {
                r if r == ResOk.into() => return Ok(saved_value),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn write<T>(&self, value: T) -> Result<ResultOk, ResultErr> {
        let mut num_bytes_written = 0;
        let value_ptr = &value as *const T as *const c_void;
        unsafe {
            match self.inner.write(
                value_ptr,
                std::mem::size_of::<T>() as i32,
                &mut num_bytes_written,
            ) {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }

    pub fn seek<T>(&self, mode: SeekMode) -> Result<ResultOk, ResultErr> {
        let mut result = 0i64;
        unsafe {
            match self
                .inner
                .seek(std::mem::size_of::<T>() as i64, mode.into(), &mut result)
            {
                r if r == ResOk.into() => Ok(ResOk),
                r => Err(ResultErr::from(r)),
            }
        }
    }
}
