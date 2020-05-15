use vst3_sys::vst::EventTypes::*;
use vst3_sys::vst::{
    ChordEvent, DataEvent, Event, EventData, EventTypes, IEventList, LegacyMidiCCOutEvent,
    NoteExpressionTextEvent, NoteExpressionValueEvent, NoteOffEvent, NoteOnEvent,
    PolyPressureEvent, ScaleEvent,
};

pub enum EventType {
    NoteOnEvent,
    NoteOffEvent,
    DataEvent,
    PolyPressureEvent,
    NoteExpressionValueEvent,
    NoteExpressionTextEvent,
    ChordEvent,
    ScaleEvent,
    LegacyMIDICCOutEvent,
}

impl From<u16> for EventType {
    fn from(e_ty: u16) -> Self {
        match e_ty {
            e_ty if e_ty == kNoteOnEvent as u16 => EventType::NoteOnEvent,
            e_ty if e_ty == kNoteOffEvent as u16 => EventType::NoteOffEvent,
            e_ty if e_ty == kDataEvent as u16 => EventType::DataEvent,
            e_ty if e_ty == kPolyPressureEvent as u16 => EventType::PolyPressureEvent,
            e_ty if e_ty == kNoteExpressionValueEvent as u16 => EventType::NoteExpressionValueEvent,
            e_ty if e_ty == kNoteExpressionTextEvent as u16 => EventType::NoteExpressionTextEvent,
            e_ty if e_ty == kChordEvent as u16 => EventType::ChordEvent,
            e_ty if e_ty == kScaleEvent as u16 => EventType::ScaleEvent,
            e_ty if e_ty == kLegacyMIDICCOutEvent as u16 => EventType::LegacyMIDICCOutEvent,
            _ => unreachable!(),
        }
    }
}

impl From<EventType> for u16 {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::NoteOnEvent => kNoteOnEvent as u16,
            EventType::NoteOffEvent => kNoteOffEvent as u16,
            EventType::DataEvent => kDataEvent as u16,
            EventType::PolyPressureEvent => kPolyPressureEvent as u16,
            EventType::NoteExpressionValueEvent => kNoteExpressionValueEvent as u16,
            EventType::NoteExpressionTextEvent => kNoteExpressionTextEvent as u16,
            EventType::ChordEvent => kChordEvent as u16,
            EventType::ScaleEvent => kScaleEvent as u16,
            EventType::LegacyMIDICCOutEvent => kLegacyMIDICCOutEvent as u16,
        }
    }
}

use crate::ResultOk::ResOk;
use crate::{ResultErr, Unknown, UID};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::os::raw::c_void;
use std::ptr::null;
use vst3_com::ComPtr;

pub struct EventList {
    inner: ComPtr<dyn IEventList>,
}

impl Unknown for EventList {
    const IID: UID = UID::new([0x3A2C4214, 0x346349FE, 0xB2C4F397, 0xB9695A44]);

    fn from_raw(ptr: *mut c_void) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        let ptr = ptr as *mut *mut _;
        unsafe {
            let ptr: ComPtr<dyn IEventList> = ComPtr::new(ptr);
            Some(Box::new(Self { inner: ptr }))
        }
    }
}

impl EventList {
    pub fn get_event_count(&self) -> i32 {
        unsafe { self.inner.get_event_count() }
    }

    pub fn get_event(&self, index: i32) -> Result<Event, ResultErr> {
        let mut event: Event = Event {
            bus_index: 0,
            sample_offset: 0,
            ppq_position: 0.0,
            flags: 0,
            type_: 0,
            event: EventData {
                note_on: NoteOnEvent {
                    channel: 0,
                    pitch: 0,
                    tuning: 0.0,
                    velocity: 0,
                    length: 0,
                    note_id: 0,
                },
            },
        };
        unsafe {
            match self.inner.get_event(index, &mut event as *mut _) {
                r if r == ResOk.into() => Ok(event),
                r => Err(ResultErr::from(r)),
            }
        }
    }
}

impl Debug for EventList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventList").finish()
    }
}
