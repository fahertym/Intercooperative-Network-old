extern crate lazy_static;

use lazy_static::lazy_static;
use std::sync::Mutex;
use log::{info, warn, error, debug, trace};

lazy_static! {
    static ref LOGGER: Mutex<()> = Mutex::new(());
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
