use std::marker::PhantomData;
use std::os::raw::c_void;
use std::slice;

use num_traits::Float;

use vst3_sys::vst::ProcessModes::{kOffline, kPrefetch, kRealtime};
use vst3_sys::vst::SymbolicSampleSizes::{kSample32, kSample64};
use vst3_sys::vst::{AudioBusBuffers, IAudioProcessor};

use crate::ResultErr::{InvalidArgument, NotImplemented};
use crate::ResultOk::ResOk;
use crate::{
    BusDirection, Component, EventList, ParameterChanges, ResultErr, ResultOk, Unknown,
    VST3Component,
};

pub enum SymbolicSampleSize {
    Sample32,
    Sample64,
}

impl From<i32> for SymbolicSampleSize {
    fn from(s: i32) -> Self {
        match s {
            s if s == kSample32 as i32 => SymbolicSampleSize::Sample32,
            s if s == kSample64 as i32 => SymbolicSampleSize::Sample64,
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
    fn set_bus_arrangements(
        &mut self,
        inputs: &[u64],
        outputs: &[u64],
    ) -> Result<ResultOk, ResultErr>;
    fn get_bus_arrangement(&self, dir: &BusDirection, index: i32) -> Result<u64, ResultErr>;
    fn can_process_sample_size(
        &self,
        symbolic_sample_size: SymbolicSampleSize,
    ) -> Result<ResultOk, ResultErr>;
    fn get_latency_samples(&self) -> Result<u32, ResultErr>;
    fn setup_processing(&self, setup: ProcessSetup) -> Result<ResultOk, ResultErr>;
    fn set_processing(&self, state: bool) -> Result<ResultOk, ResultErr>;
    fn process(&mut self, data: &mut ProcessData<f32>) -> Result<ResultOk, ResultErr>;
    fn process_f64(&self, data: ProcessData<f64>) -> Result<ResultOk, ResultErr>;
    fn get_tail_samples(&self) -> Result<u32, ResultErr>;
}

impl IAudioProcessor for VST3Component {
    unsafe fn set_bus_arrangements(
        &self,
        inputs: *mut u64,
        num_ins: i32,
        outputs: *mut u64,
        num_outs: i32,
    ) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            let inputs = slice::from_raw_parts(inputs, num_ins as usize);
            let outputs = slice::from_raw_parts(outputs, num_outs as usize);
            return match audio_processor.set_bus_arrangements(inputs, outputs) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_bus_arrangement(&self, dir: i32, index: i32, arr: *mut u64) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            return match audio_processor.get_bus_arrangement(&BusDirection::from(dir), index) {
                Ok(bus_arrangement) => {
                    *arr = bus_arrangement;
                    ResOk.into()
                }
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            return match audio_processor
                .can_process_sample_size(SymbolicSampleSize::from(symbolic_sample_size))
            {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            return match audio_processor.get_latency_samples() {
                Ok(latency_samples) => latency_samples,
                Err(_) => 0,
            };
        }
        0
    }

    unsafe fn setup_processing(&self, setup: *mut vst3_sys::vst::ProcessSetup) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            return match audio_processor.setup_processing(ProcessSetup::from(*setup)) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn set_processing(&self, state: u8) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            let state = if state != 0 { true } else { false };
            return match audio_processor.set_processing(state) {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        NotImplemented.into()
    }

    unsafe fn process(&self, data: *mut vst3_sys::vst::ProcessData) -> i32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            if data.is_null() {
                return InvalidArgument.into();
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

                    match audio_processor.process(&mut process_data) {
                        Ok(r) => r.into(),
                        Err(r) => r.into(),
                    }
                }
                SymbolicSampleSize::Sample64 => {
                    let process_data = ProcessData::<f64>::from_raw(
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

                    match audio_processor.process_f64(process_data) {
                        Ok(r) => r.into(),
                        Err(r) => r.into(),
                    }
                }
            };
        }
        NotImplemented.into()
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        if let Some(audio_processor) = self.get_component().lock().unwrap().as_audio_processor() {
            return match audio_processor.get_tail_samples() {
                Ok(tail_samples) => tail_samples,
                Err(_) => 0,
            };
        }
        0
    }
}
