use crate::PluginResult::{InvalidArgument, ResultTrue};
use crate::{
    AudioBus, BaseBus, BusVec, EventBus, PluginBusType, PluginResult, PluginSpeakerArrangement,
};
use std::cell::RefCell;
use vst3_sys::base::kChanged;
use vst3_sys::vst::BusDirections::{kInput, kOutput};
use vst3_sys::vst::BusFlags::kDefaultActive;
use vst3_sys::vst::BusTypes::kMain;
use vst3_sys::vst::MediaTypes::{kAudio, kEvent};
use vst3_sys::vst::SpeakerArrangement;

pub struct IO {
    pub(crate) audio_inputs: RefCell<BusVec>,
    pub(crate) audio_outputs: RefCell<BusVec>,
    pub(crate) event_inputs: RefCell<BusVec>,
    pub(crate) event_outputs: RefCell<BusVec>,
}

impl IO {
    pub fn new() -> Self {
        Self {
            audio_inputs: RefCell::new(BusVec::new(kAudio as i32, kInput as i32)),
            audio_outputs: RefCell::new(BusVec::new(kAudio as i32, kOutput as i32)),
            event_inputs: RefCell::new(BusVec::new(kEvent as i32, kInput as i32)),
            event_outputs: RefCell::new(BusVec::new(kEvent as i32, kOutput as i32)),
        }
    }

    pub fn add_audio_input(&self, audio_bus: AudioBus) {
        self.audio_inputs
            .borrow_mut()
            .inner
            .push(Box::new(audio_bus));
    }

    pub fn add_audio_output(&self, audio_bus: AudioBus) {
        self.audio_outputs
            .borrow_mut()
            .inner
            .push(Box::new(audio_bus));
    }

    pub fn add_event_input(&self, event_bus: EventBus) {
        self.event_inputs
            .borrow_mut()
            .inner
            .push(Box::new(event_bus));
    }

    pub fn add_event_output(&self, event_bus: EventBus) {
        self.event_outputs
            .borrow_mut()
            .inner
            .push(Box::new(event_bus));
    }

    pub fn clear_audio_inputs(&self) {
        self.audio_inputs.borrow_mut().inner.clear()
    }

    pub fn clear_audio_outputs(&self) {
        self.audio_outputs.borrow_mut().inner.clear()
    }

    pub fn clear_event_inputs(&self) {
        self.event_inputs.borrow_mut().inner.clear()
    }

    pub fn clear_event_outputs(&self) {
        self.event_outputs.borrow_mut().inner.clear()
    }

    pub fn activate_bus(&self, type_: i32, dir: i32, index: i32, state: u8) -> PluginResult {
        if let Some(bus_vec) = self.get_bus_vec_mut(type_, dir) {
            if index >= bus_vec.borrow().inner.len() as i32 {
                return InvalidArgument;
            }
            let mut bus = &mut bus_vec.borrow_mut().inner[index as usize];
            bus.set_active(state != 0);
            return ResultTrue;
        }
        InvalidArgument
    }

    pub(crate) fn get_bus_vec(&self, type_: i32, dir: i32) -> Option<&RefCell<BusVec>> {
        if type_ == kAudio as i32 {
            return if dir == kInput as i32 {
                Some(&self.audio_inputs)
            } else {
                Some(&self.audio_outputs)
            };
        } else if type_ == kEvent as i32 {
            return if dir == kInput as i32 {
                Some(&self.event_inputs)
            } else {
                Some(&self.event_outputs)
            };
        }
        None
    }

    pub(crate) fn get_bus_vec_mut(&self, type_: i32, dir: i32) -> Option<&RefCell<BusVec>> {
        if type_ == kAudio as i32 {
            return if dir == kInput as i32 {
                Some(&self.audio_inputs)
            } else {
                Some(&self.audio_outputs)
            };
        } else if type_ == kEvent as i32 {
            return if dir == kInput as i32 {
                Some(&self.event_inputs)
            } else {
                Some(&self.event_outputs)
            };
        }
        None
    }
}

pub struct AudioBusBuilder {
    inner: BaseBus,
    speaker_arr: SpeakerArrangement,
}
impl AudioBusBuilder {
    pub fn new(name: &str, arr: PluginSpeakerArrangement) -> Self {
        Self {
            inner: BaseBus {
                name: name.to_string(),
                bus_type: kMain as i32,
                flags: kDefaultActive as i32,
                active: false,
            },
            speaker_arr: arr as u64,
        }
    }

    pub fn bus_type(mut self, bus_type: PluginBusType) -> Self {
        self.inner.bus_type = bus_type as i32;
        self
    }

    pub fn build(&self) -> AudioBus {
        AudioBus {
            inner: self.inner.clone(),
            speaker_arr: self.speaker_arr,
        }
    }
}

pub struct EventBusBuilder {
    inner: BaseBus,
    channel_count: i32,
}
impl EventBusBuilder {
    pub fn new(name: &str, channel_count: i32) -> Self {
        Self {
            inner: BaseBus {
                name: name.to_string(),
                bus_type: kMain as i32,
                flags: kDefaultActive as i32,
                active: false,
            },
            channel_count,
        }
    }

    pub fn bus_type(mut self, bus_type: PluginBusType) -> Self {
        self.inner.bus_type = bus_type as i32;
        self
    }

    pub fn build(&self) -> EventBus {
        EventBus {
            inner: self.inner.clone(),
            channel_count: self.channel_count,
        }
    }
}
