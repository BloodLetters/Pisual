use pumpkin_plugin_api::logging::{LogLevel, log};

pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

pub fn error(message: &str) {
    log(LogLevel::Error, message);
}

pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}
