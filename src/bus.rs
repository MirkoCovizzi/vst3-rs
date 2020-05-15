use crate::{get_channel_count, wstrcpy, BusDirection, BusInfo, BusType, MediaType};

pub trait Bus {
    fn is_active(&self) -> bool;
    fn set_active(&mut self, state: bool);
    fn set_name(&mut self, new_name: &str);
    fn set_bus_type(&mut self, new_bus_type: BusType);
    fn set_flags(&mut self, new_flags: i32);
    fn get_info(&self, info: &mut BusInfo);
    fn as_audio_bus(&self) -> Option<&dyn AudioBus> {
        None
    }
    fn as_audio_bus_mut(&mut self) -> Option<&mut dyn AudioBus> {
        None
    }
    fn as_event_bus(&self) -> Option<&dyn EventBus> {
        None
    }
    fn as_event_bus_mut(&mut self) -> Option<&mut dyn EventBus> {
        None
    }
}

pub trait AudioBus {
    fn get_speaker_arrangement(&self) -> u64;
}

pub trait EventBus {
    fn get_channel_count(&self) -> i32;
}

struct BaseBus {
    name: String,
    bus_type: BusType,
    flags: i32,
    active: bool,
}

impl BaseBus {
    pub fn new(name: &str, bus_type: BusType, flags: i32) -> Box<Self> {
        Box::new(Self {
            name: name.to_string(),
            bus_type,
            flags,
            active: false,
        })
    }
}

impl Bus for BaseBus {
    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, state: bool) {
        self.active = state
    }

    fn set_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    fn set_bus_type(&mut self, new_bus_type: BusType) {
        self.bus_type = new_bus_type
    }

    fn set_flags(&mut self, new_flags: i32) {
        self.flags = new_flags
    }

    fn get_info(&self, info: &mut BusInfo) {
        info.name = self.name.clone();
        info.bus_type = self.bus_type.clone();
        info.flags = self.flags as u32;
    }
}

pub struct BaseAudioBus {
    inner: Box<BaseBus>,
    pub speaker_arr: u64,
}

impl BaseAudioBus {
    pub fn new(name: &str, bus_type: BusType, flags: i32, arr: u64) -> Box<Self> {
        Box::new(Self {
            inner: BaseBus::new(name, bus_type, flags),
            speaker_arr: arr,
        })
    }
}

impl Bus for BaseAudioBus {
    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn set_active(&mut self, state: bool) {
        self.inner.set_active(state)
    }

    fn set_name(&mut self, new_name: &str) {
        self.inner.set_name(new_name)
    }

    fn set_bus_type(&mut self, new_bus_type: BusType) {
        self.set_bus_type(new_bus_type)
    }

    fn set_flags(&mut self, new_flags: i32) {
        self.set_flags(new_flags)
    }

    fn get_info(&self, info: &mut BusInfo) {
        info.channel_count = get_channel_count(self.speaker_arr);
        self.inner.get_info(info);
    }

    fn as_audio_bus(&self) -> Option<&dyn AudioBus> {
        Some(self)
    }

    fn as_audio_bus_mut(&mut self) -> Option<&mut dyn AudioBus> {
        Some(self)
    }
}

impl AudioBus for BaseAudioBus {
    fn get_speaker_arrangement(&self) -> u64 {
        self.speaker_arr
    }
}

pub struct BaseEventBus {
    inner: Box<BaseBus>,
    pub channel_count: i32,
}

impl BaseEventBus {
    pub fn new(name: &str, bus_type: BusType, flags: i32, channel_count: i32) -> Box<Self> {
        Box::new(Self {
            inner: BaseBus::new(name, bus_type, flags),
            channel_count,
        })
    }
}

impl Bus for BaseEventBus {
    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn set_active(&mut self, state: bool) {
        self.inner.set_active(state)
    }

    fn set_name(&mut self, new_name: &str) {
        self.inner.set_name(new_name)
    }

    fn set_bus_type(&mut self, new_bus_type: BusType) {
        self.set_bus_type(new_bus_type)
    }

    fn set_flags(&mut self, new_flags: i32) {
        self.set_flags(new_flags)
    }

    fn get_info(&self, info: &mut BusInfo) {
        info.channel_count = self.channel_count;
        self.inner.get_info(info);
    }

    fn as_event_bus(&self) -> Option<&dyn EventBus> {
        Some(self)
    }

    fn as_event_bus_mut(&mut self) -> Option<&mut dyn EventBus> {
        Some(self)
    }
}

impl EventBus for BaseEventBus {
    fn get_channel_count(&self) -> i32 {
        self.channel_count
    }
}

pub struct BusVec {
    inner: Vec<Box<dyn Bus>>,
    pub type_: MediaType,
    pub direction: BusDirection,
}

impl BusVec {
    pub fn new(type_: MediaType, direction: BusDirection) -> Self {
        Self {
            inner: vec![],
            type_,
            direction,
        }
    }

    pub fn get_vec(&self) -> &Vec<Box<dyn Bus>> {
        &self.inner
    }

    pub fn get_vec_mut(&mut self) -> &mut Vec<Box<dyn Bus>> {
        &mut self.inner
    }
}
