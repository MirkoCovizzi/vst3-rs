#[macro_use]
extern crate vst3;

use vst3::ClassCardinality::kManyInstances;
use vst3::{kDefaultFactoryFlags, Component, ComponentInfo, GUID};
use vst3::{Factory, FactoryInfo};

struct AGain {}

impl Default for AGain {
    fn default() -> Self {
        AGain {}
    }
}

impl Component for AGain {
    fn info(&self) -> ComponentInfo {
        ComponentInfo {
            cid: GUID { data: [0; 16] },
            cardinality: kManyInstances as i32,
            category: "Audio Module Class".to_string(),
            name: "AGain VST3".to_string(),
            class_flags: 1,
            subcategories: "Fx".to_string(),
            vendor: "".to_string(),
            version: "0.1.0".to_string(),
            sdk_version: "VST 3.6.14".to_string(),
        }
    }
}

struct AGainFactory {
    components: Vec<Box<dyn Component>>,
}

impl Default for AGainFactory {
    fn default() -> Self {
        AGainFactory {
            components: vec![AGain::new() as Box<dyn Component>],
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

    fn get_components(&self) -> &Vec<Box<dyn Component>> {
        &self.components
    }
}

factory_main!(AGainFactory);
