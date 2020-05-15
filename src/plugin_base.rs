use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::Mutex;

use vst3_sys::base::IPluginBase;
use vst3_sys::vst::{IAudioProcessor, IComponent, IEditController, IUnitInfo};
use vst3_sys::VST3;

use crate::ResultErr::InvalidArgument;
use crate::{
    AudioProcessor, ClassInfo, Component, EditController, HostApplication, ResultErr, ResultOk,
    UnitInfo, Unknown,
};

pub trait PluginBase {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn get_class_info(&self) -> ClassInfo;

    fn initialize(&mut self, context: HostApplication) -> Result<ResultOk, ResultErr>;
    fn terminate(&mut self) -> Result<ResultOk, ResultErr>;
}
