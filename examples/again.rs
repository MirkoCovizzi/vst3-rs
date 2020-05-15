use flexi_logger::{opt_format, Logger};
use std::panic;
use vst3::BusDirection::{Input, Output};
use vst3::BusType::Main;
use vst3::MediaType::{Audio, Event};
use vst3::ParameterFlag::{CanAutomate, IsBypass, IsReadOnly};
use vst3::ResultErr::{InvalidArgument, NotImplemented, ResultFalse};
use vst3::ResultOk::ResOk;
use vst3::{
    get_channel_count, plugin_main, AudioProcessor, BaseAudioBus, BaseEventBus, BaseParameter,
    BusDirection, BusInfo, BusType, BusVec, Category, ClassInfo, ClassInfoBuilder, Component,
    ComponentHandler, EditController, FxSubcategory, HostApplication, IoMode, MediaType, Parameter,
    ParameterContainer, ParameterInfo, ParameterInfoBuilder, PlugView, PluginBase, ProcessData,
    ProcessSetup, ResultErr, ResultOk, RoutingInfo, SeekMode, Stream, SymbolicSampleSize, Unit,
    UnitBuilder, UnitInfo, NO_PROGRAM_LIST_ID, ROOT_UNIT_ID, STEREO, UID,
};

const GAIN_ID: u32 = 0;
const VU_PPM: u32 = 1;
const BYPASS_ID: u32 = 2;

struct GainParameter {
    inner: BaseParameter,
}

impl GainParameter {
    fn new(flags: i32, id: u32) -> Box<Self> {
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

    fn set_unit_id(&mut self, id: i32) {
        self.inner.set_unit_id(id)
    }

    fn get_unit_id(&self) -> i32 {
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

    fn from_string(&self, string: String) -> Result<f64, ResultErr> {
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
    fn get_class_info(&self) -> ClassInfo {
        ClassInfoBuilder::new(AGainEditController::UID)
            .name("AGain Rust Controller")
            .vendor("rust.audio")
            .category(Category::ComponentController)
            .build()
    }

    fn initialize(&mut self, context: HostApplication) -> Result<ResultOk, ResultErr> {
        log::info!("I'm fine!");

        if self.context.is_some() {
            return Err(ResultFalse);
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

        log::info!("I'm fine!");
        Ok(ResOk)
    }

    fn terminate(&mut self) -> Result<ResultOk, ResultErr> {
        self.context = None;

        Ok(ResOk)
    }
}

impl EditController for AGainEditController {
    fn set_component_state(&mut self, state: Stream) -> Result<ResultOk, ResultErr> {
        if let Ok(saved_gain) = state.read::<f64>() {
            self.set_param_normalized(GAIN_ID, saved_gain);
        } else {
            return Err(ResultFalse);
        }

        state.seek::<f64>(SeekMode::SeekCurrent);

        if let Ok(bypass_state) = state.read::<bool>() {
            self.set_param_normalized(BYPASS_ID, if bypass_state { 1.0 } else { 0.0 });
        } else {
            return Err(ResultFalse);
        }

        Ok(ResOk)
    }

    fn set_state(&self, _state: Stream) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn get_state(&self, _state: Stream) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn get_parameter_count(&self) -> Result<i32, ResultErr> {
        Ok(self.parameters.get_parameter_count() as i32)
    }

    fn get_parameter_info(&self, param_index: i32) -> Result<&ParameterInfo, ResultErr> {
        if let Some(param) = self.parameters.get_parameter_by_index(param_index as usize) {
            return Ok(param.get_info());
        }
        Err(ResultFalse)
    }

    fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
    ) -> Result<String, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_string(value_normalized));
        }
        Err(ResultFalse)
    }

    fn get_param_value_by_string(&self, id: u32, string: String) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return param.from_string(string);
        }
        Err(ResultFalse)
    }

    fn normalized_param_to_plain(&self, id: u32, value: f64) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_plain(value));
        }
        Err(ResultFalse)
    }

    fn plain_param_to_normalized(&self, id: u32, plain: f64) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.to_normalized(plain));
        }
        Err(ResultFalse)
    }

    fn get_param_normalized(&self, id: u32) -> Result<f64, ResultErr> {
        if let Some(param) = self.parameters.get_parameter(id) {
            return Ok(param.get_normalized());
        }
        Err(ResultFalse)
    }

    fn set_param_normalized(&mut self, id: u32, value: f64) -> Result<ResultOk, ResultErr> {
        if let Some(param) = self.parameters.get_parameter_mut(id) {
            param.set_normalized(value);
            return Ok(ResOk);
        }
        Err(ResultFalse)
    }

    fn set_component_handler(&self, _handler: ComponentHandler) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn create_view(&self, _name: String) -> Result<Box<dyn PlugView>, ResultErr> {
        Err(ResultFalse)
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
    fn get_class_info(&self) -> ClassInfo {
        ClassInfoBuilder::new(AGainComponent::UID)
            .name("AGain Rust")
            .vendor("rust.audio")
            .category(Category::AudioEffect)
            .subcategories(FxSubcategory::Fx)
            .build()
    }

    fn initialize(&mut self, context: HostApplication) -> Result<ResultOk, ResultErr> {
        #[cfg(debug_assertions)]
        {
            let log_path = std::env::var("VST3_LOG_PATH");
            match log_path {
                Ok(path) => {
                    match Logger::with_env_or_str("info")
                        .log_to_file()
                        .directory(path)
                        .format(opt_format)
                        .start()
                    {
                        Ok(_) => log::info!("Started logger..."),
                        Err(_) => (),
                    }
                }
                Err(_) => (),
            }

            panic::set_hook(Box::new(|info| {
                log::error!("{}", info);
            }));
        }

        if self.context.is_some() {
            return Err(ResultFalse);
        }
        self.context = Some(context);

        self.add_audio_input("Stereo In", STEREO, Main, 1);
        self.add_audio_output("Stereo Out", STEREO, Main, 1);

        self.add_event_input("Event In", 1, Main, 1);

        Ok(ResOk)
    }
    fn terminate(&mut self) -> Result<ResultOk, ResultErr> {
        self.remove_audio_busses();
        self.remove_event_busses();

        self.context = None;

        Ok(ResOk)
    }
}

impl Component for AGainComponent {
    fn as_audio_processor(&mut self) -> Option<&mut dyn AudioProcessor> {
        Some(self)
    }

    fn get_controller_class_id(&self) -> Result<UID, ResultErr> {
        Ok(AGainEditController::UID)
    }

    fn set_io_mode(&self, _mode: IoMode) -> Result<ResultOk, ResultErr> {
        Err(NotImplemented)
    }

    fn get_bus_count(&self, type_: &MediaType, dir: &BusDirection) -> Result<i32, ResultErr> {
        let bus_list = self.get_bus_vec(type_, dir);
        Ok(bus_list.get_vec().len() as i32)
    }

    fn get_bus_info(
        &self,
        type_: &MediaType,
        dir: &BusDirection,
        index: i32,
    ) -> Result<BusInfo, ResultErr> {
        if index < 0 {
            return Err(InvalidArgument);
        }
        let mut bus_list = self.get_bus_vec(type_, dir);
        if index >= bus_list.get_vec().len() as i32 {
            return Err(InvalidArgument);
        }

        let bus = &bus_list.get_vec()[index as usize];
        let mut bus_info = BusInfo {
            media_type: type_.clone(),
            direction: dir.clone(),
            channel_count: 0,
            name: "".to_string(),
            bus_type: BusType::Main,
            flags: 0,
        };

        bus.get_info(&mut bus_info);

        Ok(bus_info)
    }

    fn get_routing_info(&self) -> Result<(RoutingInfo, RoutingInfo), ResultErr> {
        Err(NotImplemented)
    }

    fn activate_bus(
        &mut self,
        type_: &MediaType,
        dir: &BusDirection,
        index: i32,
        state: bool,
    ) -> Result<ResultOk, ResultErr> {
        if index < 0 {
            return Err(InvalidArgument);
        }
        let mut bus_list = self.get_bus_vec_mut(type_, dir);
        if index >= bus_list.get_vec().len() as i32 {
            return Err(InvalidArgument);
        }

        let mut bus = &mut bus_list.get_vec_mut()[index as usize];
        bus.set_active(state);

        Ok(ResOk)
    }

    fn set_active(&self, _state: bool) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn set_state(&mut self, state: Stream) -> Result<ResultOk, ResultErr> {
        self.gain = state.read::<f64>()?;
        self.gain_reduction = state.read::<f64>()?;
        self.bypass = state.read::<bool>()?;

        Ok(ResOk)
    }

    fn get_state(&self, state: Stream) -> Result<ResultOk, ResultErr> {
        state.write::<f64>(self.gain);
        state.write::<f64>(self.gain_reduction);
        state.write::<bool>(self.bypass);

        Ok(ResOk)
    }
}

impl AudioProcessor for AGainComponent {
    fn set_bus_arrangements(
        &mut self,
        inputs: &[u64],
        outputs: &[u64],
    ) -> Result<ResultOk, ResultErr> {
        if inputs.len() == 1 && outputs.len() == 1 {
            return if get_channel_count(inputs[0]) == 1 && get_channel_count(outputs[0]) == 1 {
                let bus = self.audio_inputs.get_vec()[0].as_audio_bus().unwrap();
                if bus.get_speaker_arrangement() != inputs[0] {
                    self.remove_audio_busses();
                    self.add_audio_input("Mono In", inputs[0], Main, 1);
                    self.add_audio_output("Mono Out", inputs[0], Main, 1);
                }
                Ok(ResOk)
            } else {
                let mut result = Err(ResultFalse);
                let bus = self.audio_inputs.get_vec()[0].as_audio_bus().unwrap();
                if get_channel_count(inputs[0]) == 2 && get_channel_count(outputs[0]) == 2 {
                    self.remove_audio_busses();
                    self.add_audio_input("Stereo In", inputs[0], Main, 1);
                    self.add_audio_output("Stereo Out", inputs[0], Main, 1);
                    result = Ok(ResOk);
                } else if bus.get_speaker_arrangement() != STEREO {
                    self.remove_audio_busses();
                    self.add_audio_input("Stereo In", inputs[0], Main, 1);
                    self.add_audio_output("Stereo Out", inputs[0], Main, 1);
                    result = Err(ResultFalse);
                }
                result
            };
        }

        Err(ResultFalse)
    }

    fn get_bus_arrangement(&self, dir: &BusDirection, index: i32) -> Result<u64, ResultErr> {
        let bus_vec = self.get_bus_vec(&Audio, dir);
        if index < 0 || bus_vec.get_vec().len() <= index as usize {
            return Err(InvalidArgument);
        }

        let audio_bus = bus_vec.get_vec()[index as usize].as_audio_bus().unwrap();
        Ok(audio_bus.get_speaker_arrangement())
    }

    fn can_process_sample_size(
        &self,
        symbolic_sample_size: SymbolicSampleSize,
    ) -> Result<ResultOk, ResultErr> {
        match symbolic_sample_size {
            SymbolicSampleSize::Sample32 => Ok(ResOk),
            SymbolicSampleSize::Sample64 => Err(ResultFalse),
        }
    }

    fn get_latency_samples(&self) -> Result<u32, ResultErr> {
        Ok(0)
    }

    fn setup_processing(&self, _setup: ProcessSetup) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn set_processing(&self, _state: bool) -> Result<ResultOk, ResultErr> {
        Ok(ResOk)
    }

    fn process(&mut self, data: &mut ProcessData<f32>) -> Result<ResultOk, ResultErr> {
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
            return Ok(ResOk);
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

        Ok(ResOk)
    }

    fn process_f64(&self, _data: ProcessData<f64>) -> Result<ResultOk, ResultErr> {
        Err(NotImplemented)
    }

    fn get_tail_samples(&self) -> Result<u32, ResultErr> {
        Ok(0)
    }
}

plugin_main!(
    vendor: "rust.audio",
    url: "https://rust.audio",
    email: "mailto://rust@audio.com",
    edit_controllers: [AGainEditController],
    components: [AGainComponent]
);
