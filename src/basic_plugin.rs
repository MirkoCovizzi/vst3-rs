/*
use crate::ResultErr::{NotImplemented, ResultOk};
use crate::{
    AudioProcessor, BusDirection, BusInfo, Category, ClassInfo, ClassInfoBuilder, Component,
    ComponentHandler, EditController, EventType, FxSubcategory, HostApplication, IoMode, MediaType,
    ParamInfo, Parameters, PlugView, PluginBase, ProcessData, ProcessSetup, ResultErr, RoutingInfo,
    Stream, SymbolicSampleSize, UID,
};
use std::cell::{Cell, RefCell};

struct DummyPlugin {}

impl DummyPlugin {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }
}

impl Default for DummyPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for DummyPlugin {
    fn initialize(&mut self, _parameters: &mut Parameters) -> ResultErr {
        unimplemented!()
    }

    fn process(&mut self, _data: &mut Data) -> ResultErr {
        unimplemented!()
    }

    fn create_view(&mut self) -> Result<Box<PlugView>, ResultErr> {
        unimplemented!()
    }
}

pub struct BasicPluginEditController {
    inner: RefCell<Box<dyn Plugin>>,
    uid: UID,
    name: String,
    vendor: String,
}

impl BasicPluginEditController {
    pub fn new(plugin: Box<dyn Plugin>, uid: UID, name: &str, vendor: &str) -> Box<Self> {
        Box::new(Self {
            inner: RefCell::new(plugin),
            uid,
            name: name.to_string() + " Controller",
            vendor: vendor.to_string(),
        })
    }
}

impl Default for BasicPluginEditController {
    fn default() -> Self {
        Self {
            inner: RefCell::new(DummyPlugin::new()),
            uid: UID::new([0, 0, 0, 0]),
            name: "".to_string(),
            vendor: "".to_string(),
        }
    }
}

impl PluginBase for BasicPluginEditController {
    fn get_class_info(&self) -> ClassInfo {
        ClassInfoBuilder::new(self.uid.clone())
            .name(&self.name)
            .vendor(&self.name)
            .category(Category::ComponentController)
            .build()
    }

    fn initialize(&mut self, _context: HostApplication) -> ResultErr {
        ResultOk
    }
    fn terminate(&self) -> ResultErr {
        ResultOk
    }
}

impl EditController for BasicPluginEditController {
    fn set_component_state(&mut self, _state: Stream) -> ResultErr {
        NotImplemented
    }
    fn set_state(&self, _state: Stream) -> ResultErr {
        NotImplemented
    }
    fn get_state(&self, _state: Stream) -> ResultErr {
        NotImplemented
    }
    fn get_parameter_count(&self) -> Result<i32, ResultErr> {
        Err(NotImplemented)
    }
    fn get_parameter_info(&self, _param_index: i32) -> Result<ParamInfo, ResultErr> {
        Err(NotImplemented)
    }
    fn get_param_string_by_value(
        &self,
        _id: u32,
        _value_normalized: f64,
    ) -> Result<String, ResultErr> {
        Err(NotImplemented)
    }
    fn get_param_value_by_string(&self, _id: u32, _string: String) -> Result<f64, ResultErr> {
        Err(NotImplemented)
    }
    fn normalized_param_to_plain(&self, _id: u32, _value: f64) -> Result<f64, ResultErr> {
        Err(NotImplemented)
    }
    fn plain_param_to_normalized(&self, _id: u32, _plain: f64) -> Result<f64, ResultErr> {
        Err(NotImplemented)
    }
    fn get_param_normalized(&self, _id: u32) -> Result<f64, ResultErr> {
        Err(NotImplemented)
    }
    fn set_param_normalized(&self, _id: u32, _value: f64) -> ResultErr {
        NotImplemented
    }
    fn set_component_handler(&self, _handler: ComponentHandler) -> ResultErr {
        NotImplemented
    }
    fn create_view(&self, _name: String) -> Result<Box<dyn PlugView>, ResultErr> {
        Err(NotImplemented)
    }
}

pub struct BasicPluginComponent {
    inner: RefCell<Box<dyn Plugin>>,
    uid: UID,
    name: String,
    vendor: String,
}

impl BasicPluginComponent {
    pub fn new(plugin: Box<dyn Plugin>, uid: UID, name: &str, vendor: &str) -> Box<Self> {
        Box::new(Self {
            inner: RefCell::new(plugin),
            uid: uid.auto_inc(),
            name: name.to_string(),
            vendor: vendor.to_string(),
        })
    }
}

impl Default for BasicPluginComponent {
    fn default() -> Self {
        Self {
            inner: RefCell::new(DummyPlugin::new()),
            uid: UID::new([0, 0, 0, 0]),
            name: "".to_string(),
            vendor: "".to_string(),
        }
    }
}

impl PluginBase for BasicPluginComponent {
    fn get_class_info(&self) -> ClassInfo {
        ClassInfoBuilder::new(self.uid.clone())
            .name(&self.name)
            .vendor(&self.vendor)
            .category(Category::AudioEffect)
            .subcategories(FxSubcategory::Fx)
            .build()
    }

    fn initialize(&mut self, _context: HostApplication) -> ResultErr {
        ResultOk
    }
    fn terminate(&self) -> ResultErr {
        ResultOk
    }
}

impl Component for BasicPluginComponent {
    fn get_controller_class_id(&self) -> Result<UID, ResultErr> {
        Ok(self.uid.clone().auto_dec())
    }
    fn set_io_mode(&self, _mode: IoMode) -> ResultErr {
        NotImplemented
    }
    fn get_bus_count(&self, _type_: MediaType, _dir: BusDirection) -> Result<i32, ResultErr> {
        Err(NotImplemented)
    }
    fn get_bus_info(
        &self,
        _type_: MediaType,
        _dir: BusDirection,
        _index: i32,
    ) -> Result<BusInfo, ResultErr> {
        println!("{:?}", self.uid);
        Err(NotImplemented)
    }
    fn get_routing_info(&self) -> Result<(RoutingInfo, RoutingInfo), ResultErr> {
        Err(NotImplemented)
    }
    fn activate_bus(
        &self,
        _type_: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: bool,
    ) -> ResultErr {
        NotImplemented
    }
    fn set_active(&self, _state: bool) -> ResultErr {
        NotImplemented
    }
    fn set_state(&self, _state: Stream) -> ResultErr {
        NotImplemented
    }
    fn get_state(&self, _state: Stream) -> ResultErr {
        NotImplemented
    }
}

impl AudioProcessor for BasicPluginComponent {
    fn set_bus_arrangements(&self, _inputs: &[u64], _outputs: &[u64]) -> ResultErr {
        NotImplemented
    }

    fn get_bus_arrangement(&self, _dir: BusDirection, _index: i32) -> Result<u64, ResultErr> {
        Err(NotImplemented)
    }

    fn can_process_sample_size(&self, _symbolic_sample_size: SymbolicSampleSize) -> ResultErr {
        NotImplemented
    }

    fn get_latency_samples(&self) -> Result<u32, ResultErr> {
        Err(NotImplemented)
    }

    fn setup_processing(&self, _setup: ProcessSetup) -> ResultErr {
        NotImplemented
    }

    fn set_processing(&self, _state: bool) -> ResultErr {
        NotImplemented
    }

    fn process(&mut self, _data: ProcessData<f32>) -> ResultErr {
        NotImplemented
    }

    fn process_f64(&self, _data: ProcessData<f64>) -> ResultErr {
        NotImplemented
    }

    fn get_tail_samples(&self) -> Result<u32, ResultErr> {
        Err(NotImplemented)
    }
}

pub trait Plugin {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }
    fn initialize(&mut self, parameters: &mut Parameters) -> ResultErr;
    fn process(&mut self, data: &mut Data) -> ResultErr;
    fn create_view(&mut self) -> Result<Box<dyn PlugView>, ResultErr>;
}

pub struct Data {}
impl Data {
    pub fn get_buffer(&self) {}
    pub fn get_events(&self) {}
}
*/
