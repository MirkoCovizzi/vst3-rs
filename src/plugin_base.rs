use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::Mutex;

use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{IAudioProcessor, IComponent, IEditController, IUnitInfo};
use vst3_sys::VST3;

use crate::ResultErr::InvalidArgument;
use crate::{
    AudioProcessor, ClassInfo, Component, EditController, HostApplication, MidiMapping, ResultErr,
    ResultOk, UnitInfo, Unknown,
};

pub trait PluginBase {
    fn new() -> Box<dyn PluginBase>
    where
        Self: 'static + Sized + Default,
    {
        Box::new(Self::default()) as Box<dyn PluginBase>
    }

    fn as_component(&mut self) -> Option<&mut dyn Component> {
        None
    }
    fn as_edit_controller(&mut self) -> Option<&mut dyn EditController> {
        None
    }
    fn as_audio_processor(&mut self) -> Option<&mut dyn AudioProcessor> {
        None
    }
    fn as_unit_info(&mut self) -> Option<&mut dyn UnitInfo> {
        None
    }
    fn as_midi_mapping(&mut self) -> Option<&mut dyn MidiMapping> {
        None
    }

    fn initialize(&mut self, context: HostApplication) -> bool;
    fn terminate(&mut self) -> bool;
}
