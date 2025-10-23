use std::{
    fmt,
    io::{BufWriter, Stderr, Write},
    sync::Mutex,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    Verbose,
    Info,
    Always,
}

#[derive(Debug)]
struct Logger {
    level: Level,
    writer: BufWriter<Stderr>,
}

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub fn init(level: Level) {
    let writer = BufWriter::new(std::io::stderr());
    let logger = Logger { level, writer };
    *(LOGGER.lock().unwrap()) = Some(logger);
}

pub fn write(level: Level, message: fmt::Arguments) {
    let mut guard = LOGGER.lock().unwrap();
    let Some(logger) = &mut *guard else {
        return;
    };
    if level < logger.level {
        return;
    }
    writeln!(logger.writer, "{message}").unwrap();
}

pub fn flush() {
    let mut guard = LOGGER.lock().unwrap();
    let Some(logger) = &mut *guard else {
        return;
    };
    logger.writer.flush().unwrap();
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        $crate::log::write($crate::log::Level::Verbose, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log::write($crate::log::Level::Info, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! always {
    ($($arg:tt)*) => {
        $crate::log::write($crate::log::Level::Always, format_args!($($arg)*));
    };
}

pub use {always, info, verbose};
