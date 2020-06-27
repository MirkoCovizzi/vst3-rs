use std::panic;

use crate::register_panic_msg;
use flexi_logger::{opt_format, Logger};

pub fn setup_logger(path: &str) {
    #[cfg(debug_assertions)]
    {
        let log_path = std::env::var(path);
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
    }

    panic::set_hook(Box::new(|info| unsafe {
        register_panic_msg(&format!("{}", info));
    }));
}
