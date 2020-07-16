use std::marker::PhantomData;
use std::os::raw::c_void;
use std::slice;

use num_traits::Float;

use vst3_sys::vst::ProcessModes::{kOffline, kPrefetch, kRealtime};
use vst3_sys::vst::{AudioBusBuffers, IAudioProcessor, SymbolicSampleSizes};

use crate::ResultErr::{InternalError, InvalidArgument, NotImplemented, ResultFalse};
use crate::ResultOk::ResOk;
use crate::{
    register_panic_msg, BusDirection, Component, EventList, ParameterChanges, ResultErr, ResultOk,
    Unknown, VST3Component,
};
use std::sync::Mutex;

pub enum SymbolicSampleSize {
    Sample32,
    Sample64,
}

impl SymbolicSampleSize {
    pub(crate) fn is_valid(size: i32) -> bool {
        // todo: find better way to do this
        if size != SymbolicSampleSizes::kSample32 as i32
            && size != SymbolicSampleSizes::kSample64 as i32
        {
            false
        } else {
            true
        }
    }
}

impl From<i32> for SymbolicSampleSize {
    fn from(s: i32) -> Self {
        match s {
            s if s == SymbolicSampleSizes::kSample32 as i32 => SymbolicSampleSize::Sample32,
            s if s == SymbolicSampleSizes::kSample64 as i32 => SymbolicSampleSize::Sample64,
            _ => unreachable!(),
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
            m if m == kRealtime as i32 => ProcessMode::Realtime,
            m if m == kPrefetch as i32 => ProcessMode::Prefetch,
            m if m == kOffline as i32 => ProcessMode::Offline,
            _ => unreachable!(),
        }
    }
}

pub struct ProcessSetup {
    pub process_mode: ProcessMode,
    pub symbolic_sample_size: SymbolicSampleSize,
    pub max_samples_per_block: i32,
    pub sample_rate: f64,
}

impl From<vst3_sys::vst::ProcessSetup> for ProcessSetup {
    fn from(setup: vst3_sys::vst::ProcessSetup) -> Self {
        ProcessSetup {
            process_mode: ProcessMode::from(setup.process_mode),
            symbolic_sample_size: SymbolicSampleSize::from(setup.symbolic_sample_size),
            max_samples_per_block: setup.max_samples_per_block,
            sample_rate: setup.sample_rate,
        }
    }
}

#[derive(Debug)]
pub struct ProcessData<'a, T: 'a + Float> {
    process_mode: ProcessMode,
    num_samples: usize,
    inputs: &'a [AudioBusBuffers],
    outputs: &'a mut [AudioBusBuffers],
    in_param_changes: Option<Box<ParameterChanges>>,
    out_param_changes: Option<Box<ParameterChanges>>,
    in_events: Option<Box<EventList>>,
    out_events: Option<Box<EventList>>,
    _marker: PhantomData<T>,
}

impl<'a, T: 'a + Float> ProcessData<'a, T> {
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
    pub fn split_buffers<'b>(&'b self) -> (Inputs<'b, T>, Outputs<'b, T>)
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

    pub fn get_input_param_changes(&self) -> Option<&Box<ParameterChanges>> {
        self.in_param_changes.as_ref()
    }

    pub fn get_output_param_changes_mut(&mut self) -> Option<&mut Box<ParameterChanges>> {
        self.out_param_changes.as_mut()
    }

    pub fn get_input_events(&self) -> Option<&Box<EventList>> {
        self.in_events.as_ref()
    }

    pub fn get_output_events_mut(&mut self) -> Option<&mut Box<EventList>> {
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
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

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

pub trait AudioProcessor: Component {
    fn set_bus_arrangements(&mut self, inputs: &[u64], outputs: &[u64]) -> bool;
    fn get_bus_arrangement(&self, dir: &BusDirection, index: usize) -> Option<u64>;
    fn can_process_sample_size(&self, symbolic_sample_size: &SymbolicSampleSize) -> bool;
    fn get_latency_samples(&self) -> usize;
    fn setup_processing(&mut self, setup: &ProcessSetup) -> bool;
    fn set_processing(&mut self, state: bool) -> bool;
    fn process(&mut self, data: &mut ProcessData<f32>);
    fn process_f64(&mut self, data: &mut ProcessData<f64>);
    fn get_tail_samples(&self) -> usize;
}

impl IAudioProcessor for VST3Component {
    unsafe fn set_bus_arrangements(
        &self,
        inputs: *mut u64,
        num_ins: i32,
        outputs: *mut u64,
        num_outs: i32,
    ) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    if inputs.is_null() || outputs.is_null() || num_ins < 0 || num_outs < 0 {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    let inputs = slice::from_raw_parts(inputs, num_ins as usize);
                    let outputs = slice::from_raw_parts(outputs, num_outs as usize);
                    return if audio_processor.set_bus_arrangements(inputs, outputs) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: set_bus_arrangements: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_bus_arrangement(&self, dir: i32, index: i32, arr: *mut u64) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    if !BusDirection::is_valid(dir) || index < 0 || arr.is_null() {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return match audio_processor
                        .get_bus_arrangement(&BusDirection::from(dir), index as usize)
                    {
                        Some(bus_arrangement) => {
                            *arr = bus_arrangement;
                            *ret.lock().unwrap() = ResOk.into()
                        }
                        None => *ret.lock().unwrap() = ResultFalse.into(),
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_bus_arrangement: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    if !SymbolicSampleSize::is_valid(symbolic_sample_size) {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return if audio_processor
                        .can_process_sample_size(&SymbolicSampleSize::from(symbolic_sample_size))
                    {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: can_process_sample_size: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<u32> = Mutex::new(0);
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    let latency_samples = audio_processor.get_latency_samples();
                    if latency_samples > u32::MAX as usize {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "VST3Component: get_latency_samples: returned value is too big! \
                                    {}usize > {}u32",
                            latency_samples,
                            u32::MAX
                        );
                        return;
                    } else {
                        return *ret.lock().unwrap() = latency_samples as u32;
                    }
                }
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: get_latency_samples: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn setup_processing(&self, setup: *const vst3_sys::vst::ProcessSetup) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    if setup.is_null() {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return if audio_processor.setup_processing(&ProcessSetup::from(*setup)) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: setup_processing: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn set_processing(&self, state: u8) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    let state = if state != 0 { true } else { false };
                    return if audio_processor.set_processing(state) {
                        *ret.lock().unwrap() = ResOk.into()
                    } else {
                        *ret.lock().unwrap() = ResultFalse.into()
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: set_processing: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn process(&self, data: *mut vst3_sys::vst::ProcessData) -> i32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<i32> = Mutex::new(InternalError.into());
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    if data.is_null() {
                        return *ret.lock().unwrap() = InvalidArgument.into();
                    }
                    return match SymbolicSampleSize::from((*data).symbolic_sample_size) {
                        SymbolicSampleSize::Sample32 => {
                            let mut process_data = ProcessData::<f32>::from_raw(
                                (*data).num_inputs as usize,
                                (*data).num_outputs as usize,
                                (*data).inputs as *const AudioBusBuffers,
                                (*data).outputs,
                                ProcessMode::from((*data).process_mode),
                                (*data).num_samples as usize,
                                (*data).input_parameter_changes as *mut c_void,
                                (*data).output_parameter_changes as *mut c_void,
                                (*data).input_events as *mut c_void,
                                (*data).output_events as *mut c_void,
                            );

                            audio_processor.process(&mut process_data);

                            *ret.lock().unwrap() = ResOk.into();
                        }
                        SymbolicSampleSize::Sample64 => {
                            let mut process_data = ProcessData::<f64>::from_raw(
                                (*data).num_inputs as usize,
                                (*data).num_outputs as usize,
                                (*data).inputs as *const AudioBusBuffers,
                                (*data).outputs,
                                ProcessMode::from((*data).process_mode),
                                (*data).num_samples as usize,
                                (*data).input_parameter_changes as *mut c_void,
                                (*data).output_parameter_changes as *mut c_void,
                                (*data).input_events as *mut c_void,
                                (*data).output_events as *mut c_void,
                            );

                            audio_processor.process_f64(&mut process_data);

                            *ret.lock().unwrap() = ResOk.into();
                        }
                    };
                }
                return *ret.lock().unwrap() = NotImplemented.into();
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: process: panic");
                *ret.lock().unwrap()
            }
        }
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        let mutex_plugin_base = self.get_plugin_base();
        let ret: Mutex<u32> = Mutex::new(0);
        match std::panic::catch_unwind(|| {
            if let Ok(mut plugin_base) = mutex_plugin_base.lock() {
                if let Some(audio_processor) = plugin_base.as_audio_processor() {
                    let tail_samples = audio_processor.get_tail_samples();
                    if tail_samples > u32::MAX as usize {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "VST3Component: get_tail_samples: returned value is too big! \
                                    {}usize > {}u32",
                            tail_samples,
                            u32::MAX
                        );
                        return;
                    } else {
                        return *ret.lock().unwrap() = tail_samples as u32;
                    }
                }
            }
        }) {
            Ok(_) => *ret.lock().unwrap(),
            Err(_) => {
                #[cfg(debug_assertions)]
                log::error!("VST3Component: terminate: panic");
                *ret.lock().unwrap()
            }
        }
    }
}
