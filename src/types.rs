use crate::EventList;
use num_traits::Float;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::slice;
use vst3_com::sys::GUID;
use vst3_com::{c_void, ComPtr};
use vst3_sys::base::IBStream;
use vst3_sys::vst::{
    kFx, kVstAudioEffectClass, kVstComponentControllerClass, AudioBusBuffers, IParamValueQueue,
    IParameterChanges, ProcessModes, SymbolicSampleSizes,
};

use vst3_sys::base::{
    kInternalError, kInvalidArgument, kNoInterface, kNotImplemented, kNotInitialized, kOutOfMemory,
    kResultFalse, kResultOk, kResultTrue,
};

pub enum PluginResult {
    NoInterface,
    ResultOk,
    ResultTrue,
    ResultFalse,
    InvalidArgument,
    NotImplemented,
    InternalError,
    NotInitialized,
    OutOfMemory,
}

impl PluginResult {
    pub fn unwrap(&self) {}
}

impl From<i32> for PluginResult {
    fn from(result: i32) -> Self {
        match result {
            kNoInterface => PluginResult::NoInterface,
            kResultOk => PluginResult::ResultOk,
            kResultTrue => PluginResult::ResultTrue,
            kResultFalse => PluginResult::ResultFalse,
            kInvalidArgument => PluginResult::InvalidArgument,
            kNotImplemented => PluginResult::NotImplemented,
            kInternalError => PluginResult::InternalError,
            kNotInitialized => PluginResult::NotInitialized,
            kOutOfMemory => PluginResult::OutOfMemory,
            _ => unreachable!(),
        }
    }
}

impl From<PluginResult> for i32 {
    fn from(plugin_result: PluginResult) -> Self {
        match plugin_result {
            PluginResult::NoInterface => kNoInterface,
            PluginResult::ResultOk => kResultOk,
            PluginResult::ResultTrue => kResultTrue,
            PluginResult::ResultFalse => kResultFalse,
            PluginResult::InvalidArgument => kInvalidArgument,
            PluginResult::NotImplemented => kNotImplemented,
            PluginResult::InternalError => kInternalError,
            PluginResult::NotInitialized => kNotInitialized,
            PluginResult::OutOfMemory => kOutOfMemory,
        }
    }
}

#[derive(Clone)]
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
        GUID { data: tuid }
    }
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
    Fx,
}
impl ToString for FxSubcategory {
    fn to_string(&self) -> String {
        match self {
            FxSubcategory::Fx => unsafe { CStr::from_ptr(kFx).to_string_lossy().to_string() },
        }
    }
}

#[derive(Debug)]
pub enum ProcessMode {
    Realtime,
    Prefetch,
    Offline,
}
impl From<i32> for ProcessMode {
    fn from(mode: i32) -> Self {
        match mode {
            0 => ProcessMode::Realtime,
            1 => ProcessMode::Prefetch,
            2 => ProcessMode::Offline,
            _ => unreachable!(),
        }
    }
}
impl From<ProcessModes> for ProcessMode {
    fn from(mode: ProcessModes) -> Self {
        match mode {
            ProcessModes::kRealtime => ProcessMode::Realtime,
            ProcessModes::kPrefetch => ProcessMode::Prefetch,
            ProcessModes::kOffline => ProcessMode::Offline,
        }
    }
}

#[derive(Debug)]
pub enum SymbolicSampleSize {
    Sample32,
    Sample64,
}
impl From<i32> for SymbolicSampleSize {
    fn from(symbolic_sample_size: i32) -> Self {
        match symbolic_sample_size {
            0 => SymbolicSampleSize::Sample32,
            1 => SymbolicSampleSize::Sample64,
            _ => unreachable!(),
        }
    }
}
impl From<SymbolicSampleSizes> for SymbolicSampleSize {
    fn from(symbolic_sample_size: SymbolicSampleSizes) -> Self {
        match symbolic_sample_size {
            SymbolicSampleSizes::kSample32 => SymbolicSampleSize::Sample32,
            SymbolicSampleSizes::kSample64 => SymbolicSampleSize::Sample64,
        }
    }
}

#[derive(Debug)]
pub struct PluginProcessData<'a, T: 'a + Float> {
    process_mode: ProcessMode,
    num_samples: usize,
    inputs: &'a [AudioBusBuffers],
    outputs: &'a mut [AudioBusBuffers],
    in_param_changes: Option<ParameterChanges>,
    out_param_changes: Option<ParameterChanges>,
    in_events: Option<EventList>,
    out_events: Option<EventList>,
    _marker: PhantomData<T>,
}

impl<'a, T: 'a + Float> PluginProcessData<'a, T> {
    #[inline]
    pub(crate) unsafe fn from_raw(
        num_inputs: usize,
        num_outputs: usize,
        inputs_raw: *const AudioBusBuffers,
        outputs_raw: *mut AudioBusBuffers,
        process_mode: ProcessMode,
        num_samples: usize,
        in_param_ptr: *mut c_void,
        out_param_ptr: *mut c_void,
        in_events_ptr: *mut c_void,
        out_events_ptr: *mut c_void,
    ) -> Self {
        Self {
            inputs: slice::from_raw_parts(inputs_raw, num_inputs),
            outputs: slice::from_raw_parts_mut(outputs_raw, num_outputs),
            process_mode,
            num_samples,
            in_param_changes: ParameterChanges::from_raw(in_param_ptr),
            out_param_changes: ParameterChanges::from_raw(out_param_ptr),
            _marker: PhantomData,
            in_events: EventList::from_raw(in_events_ptr),
            out_events: EventList::from_raw(out_events_ptr),
        }
    }

    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    #[inline]
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }

    #[inline]
    pub fn split<'b>(&'b self) -> (Inputs<'b, T>, Outputs<'b, T>)
    where
        'a: 'b,
    {
        (
            Inputs {
                buffers: self.inputs,
                num_samples: self.num_samples,
                _marker: PhantomData,
            },
            Outputs {
                buffers: self.outputs,
                num_samples: self.num_samples,
                _marker: PhantomData,
            },
        )
    }

    pub fn get_input_param_changes(&self) -> Option<&ParameterChanges> {
        self.in_param_changes.as_ref()
    }

    pub fn get_output_param_changes_mut(&mut self) -> Option<&mut ParameterChanges> {
        self.out_param_changes.as_mut()
    }

    pub fn get_input_events(&self) -> Option<&EventList> {
        self.in_events.as_ref()
    }

    pub fn get_output_events_mut(&mut self) -> Option<&mut EventList> {
        self.out_events.as_mut()
    }
}

pub struct Inputs<'a, T: 'a> {
    buffers: &'a [AudioBusBuffers],
    num_samples: usize,
    _marker: PhantomData<T>,
}

impl<'a, T> Inputs<'a, T> {
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    pub fn get(&self, index: usize) -> InputAudioBusBuffer<'a, T> {
        unsafe {
            InputAudioBusBuffer {
                silence_flags: self.buffers[index].silence_flags,
                buffers: slice::from_raw_parts(
                    self.buffers[index].buffers as *const *const T,
                    self.buffers[index].num_channels as usize,
                ),
                num_samples: self.num_samples,
            }
        }
    }
}

pub struct InputAudioBusBuffer<'a, T: 'a> {
    silence_flags: u64,
    buffers: &'a [*const T],
    num_samples: usize,
}

impl<'a, T> InputAudioBusBuffer<'a, T> {
    pub fn get(&self, index: usize) -> &'a [T] {
        unsafe { slice::from_raw_parts(self.buffers[index], self.num_samples) }
    }
}

pub struct Outputs<'a, T: 'a> {
    buffers: &'a [AudioBusBuffers],
    num_samples: usize,
    _marker: PhantomData<T>,
}

impl<'a, T> Outputs<'a, T> {
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    pub fn get(&self, index: usize) -> OutputAudioBusBuffer<'a, T> {
        unsafe {
            OutputAudioBusBuffer {
                silence_flags: self.buffers[index].silence_flags,
                buffers: slice::from_raw_parts(
                    self.buffers[index].buffers as *mut *mut T,
                    self.buffers[index].num_channels as usize,
                ),
                num_samples: self.num_samples,
            }
        }
    }

    pub fn get_mut(&mut self, index: usize) -> OutputAudioBusBuffer<'a, T> {
        unsafe {
            OutputAudioBusBuffer {
                silence_flags: self.buffers[index].silence_flags,
                buffers: slice::from_raw_parts_mut(
                    self.buffers[index].buffers as *mut *mut T,
                    self.buffers[index].num_channels as usize,
                ),
                num_samples: self.num_samples,
            }
        }
    }
}

pub struct OutputAudioBusBuffer<'a, T: 'a> {
    silence_flags: u64,
    buffers: &'a [*mut T],
    num_samples: usize,
}

impl<'a, T> OutputAudioBusBuffer<'a, T> {
    pub fn get_mut(&mut self, index: usize) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buffers[index], self.num_samples) }
    }
}

pub struct ParameterChanges {
    inner: ComPtr<dyn IParameterChanges>,
}

impl ParameterChanges {
    pub fn from_raw(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IParameterChanges> = ComPtr::new(ptr);
            Some(Self { inner: ptr })
        }
    }

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

    // todo: add_parameter_data()
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

    pub fn get_parameter_id(&self) -> u32 {
        unsafe { self.inner.get_parameter_id() as u32 }
    }

    pub fn get_point_count(&self) -> i32 {
        unsafe { self.inner.get_point_count() as i32 }
    }

    // todo: make a sort of Option but for tresult
    pub fn get_point(&self, index: i32) -> Option<ParamValuePoint> {
        let mut value = 0.0;
        let mut sample_offset = 0i32;
        unsafe {
            match self
                .inner
                .get_point(index, &mut sample_offset as *mut _, &mut value as *mut _)
            {
                kResultTrue => Some(ParamValuePoint {
                    value,
                    sample_offset,
                }),
                _ => None,
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
    pub value: f64,
    pub sample_offset: i32,
}

pub struct BStream {
    inner: ComPtr<dyn IBStream>,
}

impl BStream {
    pub fn from_raw(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IBStream> = ComPtr::new(ptr);
            Some(Self { inner: ptr })
        }
    }

    pub fn read<T>(&self) -> Option<T> {
        let mut num_bytes_read = 0;
        let mut saved_value: T = unsafe { std::mem::zeroed() };
        let value_ptr = &mut saved_value as *mut T as *mut c_void;
        unsafe {
            match self.inner.read(
                value_ptr,
                std::mem::size_of::<T>() as i32,
                &mut num_bytes_read,
            ) {
                kResultTrue => return Some(saved_value),
                _ => None,
            }
        }
    }

    pub fn write<T>(&self, value: T) -> PluginResult {
        let mut num_bytes_written = 0;
        let value_ptr = &value as *const T as *const c_void;
        unsafe {
            PluginResult::from(self.inner.write(
                value_ptr,
                std::mem::size_of::<T>() as i32,
                &mut num_bytes_written,
            ))
        }
    }
}
