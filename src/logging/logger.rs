//use log::*;

lazy_static::lazy_static! {
    pub static ref LOGGER: std::sync::Mutex<()> = std::sync::Mutex::new(());
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        let _guard = $crate::logging::LOGGER.lock().unwrap();
        info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        let _guard = $crate::logging::LOGGER.lock().unwrap();
        warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        let _guard = $crate::logging::LOGGER.lock().unwrap();
        error!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        let _guard = $crate::logging::LOGGER.lock().unwrap();
        debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        let _guard = $crate::logging::LOGGER.lock().unwrap();
        trace!($($arg)*);
    };
}