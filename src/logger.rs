use std::collections::HashMap;

use log::{Metadata, Record};
use once_cell::sync::Lazy;

static mut LOGGER_CONFIG: Lazy<LoggerConfig> = Lazy::new(LoggerConfig::new);

pub struct LoggerConfig {
    module_levels: HashMap<String, mhw_toolkit::logger::LogLevel>,
}

impl LoggerConfig {
    pub fn new() -> Self {
        LoggerConfig {
            module_levels: HashMap::new(),
        }
    }

    // pub fn set_module_level(&mut self, module: &str, level: mhw_toolkit::logger::LogLevel) {
    //     self.module_levels.insert(module.to_string(), level);
    // }

    pub fn get_level(&self, module: &str) -> Option<log::Level> {
        self.module_levels.get(module).map(|m| m.clone().into())
    }
}

pub struct MASLogger {
    prefix: String,
}

impl MASLogger {
    pub fn new() -> Self {
        Self {
            prefix: "More Auto Shoutouts".to_string(),
        }
    }
}

impl log::Log for MASLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target_max_level = match metadata.target().strip_prefix("more_auto_shoutouts::") {
            Some(target) => unsafe { LOGGER_CONFIG.get_level(target).unwrap_or(log::Level::Debug) },
            None => log::Level::Debug,
        };
        if target_max_level < metadata.level() {
            return false;
        }
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            mhw_toolkit::logger::log_to_loader(
                record.level().into(),
                &format!(
                    "[{}] {} - {} - {}",
                    self.prefix,
                    record.level(),
                    record.target().strip_prefix("more_auto_shoutouts::").unwrap_or(""),
                    record.args()
                ),
            );
        }
    }

    fn flush(&self) {}
}
