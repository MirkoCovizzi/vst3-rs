#[macro_use]
extern crate vst3;

use vst3::kDefaultFactoryFlags;
use vst3::{Factory, FactoryInfo};

struct AGainFactory {}

impl Default for AGainFactory {
    fn default() -> Self {
        AGainFactory {}
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
}

factory_main!(AGainFactory);
