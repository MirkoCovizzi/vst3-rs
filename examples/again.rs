#[macro_use]
use std::panic;

use flexi_logger::{opt_format, Logger};

use vst3::PluginSpeakerArrangement::Stereo;
use vst3::{
    AudioBusBuilder, Category, EventBusBuilder, EventType, FxSubcategory, Info, InfoBuilder,
    ParameterBuilder, Plugin, PluginProcessData,
};
use vst3::{Events, Parameters, IO, UID};

struct AGain {
    parameters: Parameters,
    events: Events,
    io: IO,
}
impl AGain {
    const UID: UID = UID::new([0x84E8DE5F, 0x92554F53, 0x96FAE413, 0x3C935A18]);
    const GAIN: u32 = 0;
    const BYPASS: u32 = 1;
    const GAIN_REDUCTION: u32 = 2;
}

impl Default for AGain {
    fn default() -> Self {
        AGain {
            parameters: Parameters::new(),
            events: Events::new(),
            io: IO::new(),
        }
    }
}

impl Plugin for AGain {
    fn get_info(&self) -> Info {
        InfoBuilder::new(AGain::UID)
            .name("AGain Rust")
            .vendor("rust.audio")
            .category(Category::AudioEffect)
            .subcategories(FxSubcategory::Fx)
            .build()
    }

    fn initialize(&self) {
        let log_path = std::env::var("VST3_LOG_PATH");
        match log_path {
            Ok(path) => {
                match Logger::with_env_or_str("info")
                    .log_to_file()
                    .directory(path)
                    .format(opt_format)
                    .start()
                {
                    Ok(_) => info!("Started logger..."),
                    Err(_) => (),
                }
            }
            Err(_) => (),
        }

        panic::set_hook(Box::new(|info| {
            error!("{}", info);
        }));

        self.io
            .add_audio_input(AudioBusBuilder::new("Stereo In", Stereo).build());
        self.io
            .add_audio_output(AudioBusBuilder::new("Stereo Out", Stereo).build());
        self.io
            .add_event_input(EventBusBuilder::new("Event In", 1).build());

        self.parameters.add_parameter(
            ParameterBuilder::new(AGain::GAIN)
                .title("Gain")
                .units("%")
                .step_count(0)
                .default_value_normalized(0.5)
                .can_automate()
                .get_param_string_by_value(|v| format!("{:.0}", v * 100.0))
                .normalized_param_to_plain(|v| v * 100.0)
                .plain_param_to_normalized(|v| v / 100.0)
                .build(),
        );
        self.parameters.add_parameter(
            ParameterBuilder::new(AGain::BYPASS)
                .title("Bypass")
                .step_count(1)
                .default_value_normalized(0.0)
                .can_automate()
                .is_bypass()
                .build(),
        );
        self.parameters.add_parameter(
            ParameterBuilder::new(AGain::GAIN_REDUCTION)
                .default_value_normalized(1.0)
                .processor_only()
                .build(),
        );

        self.events.add_event(EventType::NoteOnEvent, |e, p| {
            unsafe { info!("Note Velocity: {}", e.event.note_on.velocity) };
            p.set(AGain::GAIN_REDUCTION, 0.0).unwrap()
        });
        self.events.add_event(EventType::NoteOffEvent, |e, p| {
            p.set(AGain::GAIN_REDUCTION, 1.0).unwrap()
        });
    }

    fn process(&self, data: PluginProcessData<f32>) {
        if let Some(param_changes) = data.get_input_param_changes() {
            self.read_input_param_changes(param_changes);
        }

        if let Some(input_events) = data.get_input_events() {
            self.read_input_events(input_events);
        }

        if data.num_inputs() == 0 || data.num_outputs() == 0 {
            return;
        }

        let gain_reduction = self.parameters.get(AGain::GAIN_REDUCTION).unwrap() as f32;
        let gain = self.parameters.get(AGain::GAIN).unwrap() as f32 * gain_reduction;

        let (inputs, mut outputs) = data.split();
        let input = inputs.get(0);
        let mut output = outputs.get_mut(0);
        let input_stereo_0 = input.get(0);
        let input_stereo_1 = input.get(1);
        let output_stereo_0 = output.get_mut(0);
        let output_stereo_1 = output.get_mut(1);
        for (i, sample) in output_stereo_0.iter_mut().enumerate() {
            *sample = &*input_stereo_0.get(i).unwrap() * gain;
        }
        for (i, sample) in output_stereo_1.iter_mut().enumerate() {
            *sample = &*input_stereo_1.get(i).unwrap() * gain;
        }
    }

    fn get_parameters(&self) -> &Parameters {
        &self.parameters
    }

    fn get_events(&self) -> &Events {
        &self.events
    }

    fn get_io(&self) -> &IO {
        &self.io
    }
}

plugin_main!(
    vendor: "rust.audio",
    url: "https://rust.audio",
    email: "mailto://rust@audio.com",
    plugins: [AGain]
);
