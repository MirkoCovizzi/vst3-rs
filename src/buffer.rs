use std::os::raw::c_void;
use std::slice;

use num_traits::Float;

pub struct AudioBusBuffer<'a, T: 'a + Float> {
    silence_flags: u64,
    buffers: &'a mut [*mut T],
}

impl<'a, T: 'a + Float> AudioBusBuffer<'a, T> {
    #[inline]
    pub(crate) unsafe fn from_raw(
        num_channels: usize,
        silence_flags: u64,
        buffers_raw: *mut *mut T,
    ) -> Self {
        Self {
            silence_flags,
            buffers: slice::from_raw_parts_mut(buffers_raw, num_channels),
        }
    }

    #[inline]
    fn buffers_count(&self) -> usize {
        self.buffers.len()
    }

    #[inline]
    fn silence_flags(&self) -> &u64 {
        &self.silence_flags
    }

    #[inline]
    fn silence_flags_mut(&mut self) -> &mut u64 {
        &mut self.silence_flags
    }
}
