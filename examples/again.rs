#[macro_use]
extern crate vst3;

use vst3::ClassCardinality::kManyInstances;
use vst3::{guid, kDefaultFactoryFlags, Plugin, PluginInfo, UID};
use vst3::{Factory, FactoryInfo};

struct AGain {}
impl AGain {
    const UID: UID = [0x84E8DE5F, 0x92554F53, 0x96FAE413, 0x3C935A18];
}

impl Default for AGain {
    fn default() -> Self {
        AGain {}
    }
}

impl Plugin for AGain {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            cid: guid(AGain::UID),
            cardinality: kManyInstances as i32,
            category: "Audio Module Class".to_string(),
            name: "AGain VST3".to_string(),
            class_flags: 0,
            subcategories: "Fx".to_string(),
            vendor: "".to_string(),
            version: "0.1.0".to_string(),
            sdk_version: "VST 3.6.14".to_string(),
        }
    }
}

struct AGainFactory {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for AGainFactory {
    fn default() -> Self {
        AGainFactory {
            plugins: vec![AGain::new()],
        }
    }
}

impl Factory for AGainFactory {
    fn info(&self) -> FactoryInfo {
        FactoryInfo {
            vendor: "rust.audio".to_string(),
            url: "https://rust.audio".to_string(),
            email: "mailto://mrkcvzz@gmail.com".to_string(),
            flags: kDefaultFactoryFlags,
        }
    }

    fn get_plugins(&self) -> &Vec<Box<dyn Plugin>> {
        &self.plugins
    }
}

factory_main!(AGainFactory);
