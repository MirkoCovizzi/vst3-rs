use std::panic;

use flexi_logger::{opt_format, Logger};

use vst3::BusDirection::{Input, Output};
use vst3::BusType::Main;
use vst3::MediaType::{Audio, Event};
use vst3::ParameterFlag::{CanAutomate, IsBypass, IsReadOnly};
use vst3::ResultErr::{InvalidArgument, NotImplemented, ResultFalse};
use vst3::ResultOk::ResOk;
use vst3::{
    get_channel_count, plugin_main, setup_logger, AudioProcessor, BaseAudioBus, BaseEventBus,
    BaseParameter, BusDirection, BusInfo, BusType, BusVec, Category, ClassInfo, ClassInfoBuilder,
    Component, ComponentHandler, EditController, FactoryInfo, FxSubcategory, HostApplication,
    IoMode, MediaType, Parameter, ParameterContainer, ParameterInfo, ParameterInfoBuilder,
    PlugView, PluginBase, PluginFactory, ProcessData, ProcessSetup, ResultErr, ResultOk,
    RoutingInfo, SeekMode, Stream, SymbolicSampleSize, Unit, UnitBuilder, UnitInfo, WebPlugView,
    NO_PROGRAM_LIST_ID, ROOT_UNIT_ID, STEREO, UID,
};

const GAIN_ID: usize = 0;
const VU_PPM: usize = 1;
const BYPASS_ID: usize = 2;

struct GainParameter {
    inner: BaseParameter,
}

impl GainParameter {
    fn new(flags: i32, id: usize) -> Box<Self> {
        let info = ParameterInfoBuilder::new("Gain", id)
            .units("dB")
            .step_count(0)
            .default_normalized_value(0.5)
            .unit_id(ROOT_UNIT_ID)
            .flags(flags)
            .build();
        let mut inner = *BaseParameter::new(info);
        inner.set_normalized(1.0);
        Box::new(Self { inner })
    }
}

impl Parameter for GainParameter {
    fn get_info(&self) -> &ParameterInfo {
        &self.inner.get_info()
    }

    fn get_info_mut(&mut self) -> &mut ParameterInfo {
        self.inner.get_info_mut()
    }

    fn set_unit_id(&mut self, id: u32) {
        self.inner.set_unit_id(id)
    }

    fn get_unit_id(&self) -> u32 {
        self.inner.get_unit_id()
    }

    fn set_normalized(&mut self, v: f64) {
        self.inner.set_normalized(v)
    }

    fn get_normalized(&self) -> f64 {
        self.inner.get_normalized()
    }

    fn set_precision(&mut self, val: usize) {
        self.inner.set_precision(val)
    }

    fn get_precision(&self) -> usize {
        self.inner.get_precision()
    }

    fn to_string(&self, norm_value: f64) -> String {
        return if norm_value > 0.0001 {
            format!("{:.2}", 20.0 * norm_value.log10())
        } else {
            "-oo".to_string()
        };
    }

    fn from_string(&self, string: &str) -> Result<f64, ResultErr> {
        match string.parse::<f64>() {
            Ok(mut val) => {
                if val > 0.0 {
                    val = -val;
                }
                let norm_value = (10.0f64.ln() * val / 20.0).exp();
                return Ok(norm_value);
            }
            Err(_) => Err(ResultFalse),
        }
    }
}

struct AGainEditController {
    context: Option<HostApplication>,
    units: Vec<Unit>,
    parameters: ParameterContainer,
    component_handler: Option<ComponentHandler>,
}

impl AGainEditController {
    const UID: UID = UID::new([1, 2, 3, 4]);
    const INFO: ClassInfo = ClassInfoBuilder::new(Self::UID)
        .name("AGain Rust Controller")
        .vendor("rust.audio")
        .category(Category::ComponentController)
        .build();
}

impl Default for AGainEditController {
    fn default() -> Self {
        Self {
            context: None,
            units: vec![],
            parameters: ParameterContainer::new(),
            component_handler: None,
        }
    }
}

impl PluginBase for AGainEditController {
    fn as_edit_controller(&mut self) -> Option<&mut dyn EditController> {
        Some(self)
    }

    fn initialize(&mut self, context: HostApplication) -> bool {
        if self.context.is_some() {
            return false;
        }

        self.context = Some(context);

        if let Ok(name) = self.context.as_ref().unwrap().get_name() {
            log::info!("Host name: {}", name);
        }

        let unit = UnitBuilder::new("Unit1", 1)
            .parent_unit_id(ROOT_UNIT_ID)
            .program_list_id(NO_PROGRAM_LIST_ID)
            .build();
        self.units.push(unit);

        let gain_param = GainParameter::new(CanAutomate as i32, GAIN_ID);
        self.parameters.add_parameter(gain_param);

        let vu_param_info = ParameterInfoBuilder::new("VuPPM", VU_PPM)
            .step_count(0)
            .default_normalized_value(0.0)
            .flags(IsReadOnly as i32)
            .build();
        let vu_param = BaseParameter::new(vu_param_info);
        self.parameters.add_parameter(vu_param);

        let bypass_param_info = ParameterInfoBuilder::new("Bypass", BYPASS_ID)
            .step_count(1)
            .default_normalized_value(0.0)
            .flags(CanAutomate as i32 | IsBypass as i32)
            .build();
        let bypass_param = BaseParameter::new(bypass_param_info);
        self.parameters.add_parameter(bypass_param);

        true
    }

    fn terminate(&mut self) -> bool {
        self.parameters.remove_all();

        self.context = None;

        true
    }
}

impl EditController for AGainEditController {
    fn set_component_state(&mut self, state: &Stream) -> Result<ResultOk, ResultErr> {
        if let Some(saved_gain) = state.read::<f64>() {
            self.set_param_normalized(GAIN_ID, saved_gain);
        } else {
            return Err(ResultFalse);
        }

        state.seek::<f64>(SeekMode::SeekCurrent);

        if let Some(bypass_state) = state.read::<bool>() {
            self.set_param_normalized(BYPASS_ID, if bypass_state { 1.0 } else { 0.0 });
        } else {
            return Err(ResultFalse);
        }

        Ok(ResOk)
    }

    fn set_state(&mut self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn get_state(&self, _state: &Stream) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn get_parameter_count(&self) -> Result<usize, ResultErr> {
        Ok(self.parameters.get_parameter_count())
    }

    fn get_parameter_info(&self, param_index: usize) -> Result<&ParameterInfo, ResultErr> {
        if let Some(param) = self.parameters.get_parameter_by_index(param_index) {
            return Ok(param.get_info());
        }
        Err(ResultFalse)
    }

    fn get_param_string_by_value(
        &self,
        id: usize,
        value_normalized: f64,
    ) -> Result<String, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_string(value_normalized));
        }
        Err(ResultFalse)
    }

    fn get_param_value_by_string(&self, id: usize, string: &str) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return param.from_string(string);
        }
        Err(ResultFalse)
    }

    fn normalized_param_to_plain(&self, id: usize, value: f64) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_plain(value));
        }
        Err(ResultFalse)
    }

    fn plain_param_to_normalized(&self, id: usize, plain: f64) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_normalized(plain));
        }
        Err(ResultFalse)
    }

    fn get_param_normalized(&self, id: usize) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.get_normalized());
        }
        Err(ResultFalse)
    }

    fn set_param_normalized(&mut self, id: usize, value: f64) -> Result<ResultOk, ResultErr> {
        if let Some(param) = self.parameters.get_parameter_mut(id) {
            param.set_normalized(value);
            return Ok(ResOk);
        }
        Err(ResultFalse)
    }

    fn set_component_handler(&self, _handler: ComponentHandler) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn create_view(&mut self) -> Option<&mut Box<dyn PlugView>> {
        None
    }
}

struct AGainComponent {
    context: Option<HostApplication>,
    audio_inputs: BusVec,
    audio_outputs: BusVec,
    event_inputs: BusVec,
    event_outputs: BusVec,
    gain: f64,
    gain_reduction: f64,
    bypass: bool,
}

impl AGainComponent {
    const UID: UID = UID::new([5, 6, 7, 8]);
    const INFO: ClassInfo = ClassInfoBuilder::new(Self::UID)
        .name("AGain Rust")
        .vendor("rust.audio")
        .category(Category::AudioEffect)
        .subcategories(FxSubcategory::Fx)
        .build();

    fn add_audio_input(&mut self, name: &str, arr: u64, bus_type: BusType, flags: i32) {
        let new_bus = BaseAudioBus::new(name, bus_type, flags, arr);
        self.audio_inputs.get_vec_mut().push(new_bus);
    }

    fn add_audio_output(&mut self, name: &str, arr: u64, bus_type: BusType, flags: i32) {
        let new_bus = BaseAudioBus::new(name, bus_type, flags, arr);
        self.audio_outputs.get_vec_mut().push(new_bus);
    }

    fn add_event_input(&mut self, name: &str, channels: i32, bus_type: BusType, flags: i32) {
        let new_bus = BaseEventBus::new(name, bus_type, flags, channels);
        self.event_inputs.get_vec_mut().push(new_bus);
    }

    fn add_event_output(&mut self, name: &str, channels: i32, bus_type: BusType, flags: i32) {
        let new_bus = BaseEventBus::new(name, bus_type, flags, channels);
        self.event_outputs.get_vec_mut().push(new_bus);
    }

    fn get_bus_vec(&self, type_: &MediaType, dir: &BusDirection) -> &BusVec {
        match (type_, dir) {
            (Audio, Input) => &self.audio_inputs,
            (Audio, Output) => &self.audio_outputs,
            (Event, Input) => &self.event_inputs,
            (Event, Output) => &self.event_outputs,
            _ => unreachable!(),
        }
    }

    fn get_bus_vec_mut(&mut self, type_: &MediaType, dir: &BusDirection) -> &mut BusVec {
        match (type_, dir) {
            (Audio, Input) => &mut self.audio_inputs,
            (Audio, Output) => &mut self.audio_outputs,
            (Event, Input) => &mut self.event_inputs,
            (Event, Output) => &mut self.event_outputs,
            _ => unreachable!(),
        }
    }

    fn remove_audio_busses(&mut self) {
        self.audio_inputs.get_vec_mut().clear();
        self.audio_outputs.get_vec_mut().clear();
    }

    fn remove_event_busses(&mut self) {
        self.event_inputs.get_vec_mut().clear();
        self.event_outputs.get_vec_mut().clear();
    }
}

impl Default for AGainComponent {
    fn default() -> Self {
        Self {
            context: None,
            audio_inputs: BusVec::new(Audio, Input),
            audio_outputs: BusVec::new(Audio, Output),
            event_inputs: BusVec::new(Event, Input),
            event_outputs: BusVec::new(Event, Output),
            gain: 1.0,
            gain_reduction: 0.0,
            bypass: false,
        }
    }
}

impl PluginBase for AGainComponent {
    fn as_component(&mut self) -> Option<&mut dyn Component> {
        Some(self)
    }

    fn as_audio_processor(&mut self) -> Option<&mut dyn AudioProcessor> {
        Some(self)
    }

    fn initialize(&mut self, context: HostApplication) -> bool {
        setup_logger("VST3_LOG_PATH");

        if self.context.is_some() {
            return false;
        }
        self.context = Some(context);

        self.add_audio_input("Stereo In", STEREO, Main, 1);
        self.add_audio_output("Stereo Out", STEREO, Main, 1);

        self.add_event_input("Event In", 1, Main, 1);

        true
    }
    fn terminate(&mut self) -> bool {
        self.remove_audio_busses();
        self.remove_event_busses();

        self.context = None;

        true
    }
}

impl Component for AGainComponent {
    fn get_controller_class_id(&self) -> Option<&UID> {
        Some(&AGainEditController::UID)
    }

    fn set_io_mode(&self, _mode: &IoMode) -> bool {
        false
    }

    fn get_bus_count(&self, media_type: &MediaType, dir: &BusDirection) -> usize {
        let bus_list = self.get_bus_vec(media_type, dir);
        bus_list.get_vec().len()
    }

    fn get_bus_info(&self, type_: &MediaType, dir: &BusDirection, index: usize) -> Option<BusInfo> {
        let mut bus_list = self.get_bus_vec(type_, dir);
        if index >= bus_list.get_vec().len() {
            return None;
        }

        let bus = &bus_list.get_vec()[index];
        let mut bus_info = BusInfo {
            media_type: type_.clone(),
            direction: dir.clone(),
            channel_count: 0,
            name: "".to_string(),
            bus_type: BusType::Main,
            flags: 0,
        };

        bus.get_info(&mut bus_info);

        Some(bus_info)
    }

    fn get_routing_info(&self) -> Option<(&RoutingInfo, &RoutingInfo)> {
        None
    }

    fn activate_bus(
        &mut self,
        media_type: &MediaType,
        dir: &BusDirection,
        index: usize,
        state: bool,
    ) -> bool {
        let mut bus_list = self.get_bus_vec_mut(media_type, dir);
        if index >= bus_list.get_vec().len() {
            return false;
        }

        let mut bus = &mut bus_list.get_vec_mut()[index];
        bus.set_active(state);

        true
    }

    fn set_active(&self, _state: bool) -> bool {
        true
    }

    fn set_state(&mut self, state: &Stream) -> bool {
        if let Some(gain) = state.read::<f64>() {
            self.gain = gain;
        }
        if let Some(gain_reduction) = state.read::<f64>() {
            self.gain_reduction = gain_reduction;
        }
        if let Some(bypass) = state.read::<bool>() {
            self.bypass = bypass;
        }

        true
    }

    fn get_state(&self, state: &Stream) -> bool {
        state.write::<f64>(self.gain);
        state.write::<f64>(self.gain_reduction);
        state.write::<bool>(self.bypass);

        true
    }
}

impl AudioProcessor for AGainComponent {
    fn set_bus_arrangements(&mut self, inputs: &[u64], outputs: &[u64]) -> bool {
        if inputs.len() == 1 && outputs.len() == 1 {
            return if get_channel_count(inputs[0]) == 1 && get_channel_count(outputs[0]) == 1 {
                let bus = self.audio_inputs.get_vec()[0].as_audio_bus().unwrap();
                if bus.get_speaker_arrangement() != inputs[0] {
                    self.remove_audio_busses();
                    self.add_audio_input("Mono In", inputs[0], Main, 1);
                    self.add_audio_output("Mono Out", inputs[0], Main, 1);
                }
                true
            } else {
                let mut result = false;
                let bus = self.audio_inputs.get_vec()[0].as_audio_bus().unwrap();
                if get_channel_count(inputs[0]) == 2 && get_channel_count(outputs[0]) == 2 {
                    self.remove_audio_busses();
                    self.add_audio_input("Stereo In", inputs[0], Main, 1);
                    self.add_audio_output("Stereo Out", inputs[0], Main, 1);
                    result = true;
                } else if bus.get_speaker_arrangement() != STEREO {
                    self.remove_audio_busses();
                    self.add_audio_input("Stereo In", inputs[0], Main, 1);
                    self.add_audio_output("Stereo Out", inputs[0], Main, 1);
                    result = false;
                }
                result
            };
        }

        false
    }

    fn get_bus_arrangement(&self, dir: &BusDirection, index: usize) -> Option<u64> {
        let bus_vec = self.get_bus_vec(&Audio, dir);
        if index >= bus_vec.get_vec().len() {
            return None;
        }

        let audio_bus = bus_vec.get_vec()[index].as_audio_bus().unwrap();
        Some(audio_bus.get_speaker_arrangement())
    }

    fn can_process_sample_size(&self, symbolic_sample_size: &SymbolicSampleSize) -> bool {
        match symbolic_sample_size {
            SymbolicSampleSize::Sample32 => true,
            SymbolicSampleSize::Sample64 => false,
        }
    }

    fn get_latency_samples(&self) -> usize {
        0
    }

    fn setup_processing(&mut self, _setup: &ProcessSetup) -> bool {
        true
    }

    fn set_processing(&mut self, _state: bool) -> bool {
        true
    }

    fn process(&mut self, data: &mut ProcessData<f32>) {
        if let Some(param_changes) = data.get_input_param_changes() {
            let num_params_changed = param_changes.get_parameter_count();
            for i in 0..num_params_changed {
                if let Some(param_queue) = param_changes.get_parameter_data(i) {
                    let num_points = param_queue.get_point_count();
                    match param_queue.get_parameter_id() {
                        GAIN_ID => {
                            if let Ok(point) = param_queue.get_point(num_points - 1) {
                                self.gain = point.value;
                            }
                        }
                        BYPASS_ID => {
                            if let Ok(point) = param_queue.get_point(num_points - 1) {
                                self.bypass = if point.value > 0.5 { true } else { false };
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        if data.num_inputs() == 0 || data.num_outputs() == 0 {
            return;
        }

        let mut temp = 0.0;

        let (inputs, mut outputs) = data.split_buffers();

        let input = inputs.get(0);
        let mut output = outputs.get_mut(0);
        for i in 0..input.len() {
            let in_ = input.get(i);
            let out_ = output.get_mut(i);
            for (j, sample) in out_.iter_mut().enumerate() {
                *sample = in_[j] * self.gain as f32;
                temp = *sample;
            }
        }

        if let Some(out_param_changes) = data.get_output_param_changes_mut() {
            let mut index = 0;
            if let Some(param_queue) = out_param_changes.add_parameter_data(&VU_PPM, &mut index) {
                let mut index_2 = 0;
                param_queue.add_point(0, temp as f64, &mut index_2);
            }
        }
    }

    fn process_f64(&mut self, _data: &mut ProcessData<f64>) {}

    fn get_tail_samples(&self) -> usize {
        0
    }
}

plugin_main!(
    vendor: "rust.audio",
    url: "https://rust.audio",
    email: "mailto:rust@audio.com",
    classes: [AGainEditController, AGainComponent]
);
