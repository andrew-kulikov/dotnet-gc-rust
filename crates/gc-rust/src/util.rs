use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::str::FromStr for LogLevel {
    type Err = &'static str;

    fn from_str(level: &str) -> Result<Self, Self::Err> {
        match level.to_ascii_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err("invalid log level"),
        }
    }
}

static LOG_LEVEL: LazyLock<LogLevel> = LazyLock::new(|| {
    std::env::var("RUST_GC_LOG_LEVEL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(LogLevel::Info)
});

// Check the level before evaluating formatting arguments, which may be expensive.
macro_rules! log {
    ($level:expr, $($args:tt)*) => {{
        let level = $level;
        if $crate::util::enabled(&level) {
            $crate::util::write_log(level, format_args!($($args)*));
        }
    }};
}

macro_rules! log_error {
    ($($args:tt)*) => {
        $crate::util::log!($crate::util::LogLevel::Error, $($args)*)
    };
}

#[allow(
    unused_macros,
    reason = "keep warning-level logging available without current call sites"
)]
macro_rules! log_warn {
    ($($args:tt)*) => {
        $crate::util::log!($crate::util::LogLevel::Warn, $($args)*)
    };
}

macro_rules! log_info {
    ($($args:tt)*) => {
        $crate::util::log!($crate::util::LogLevel::Info, $($args)*)
    };
}

macro_rules! log_debug {
    ($($args:tt)*) => {
        $crate::util::log!($crate::util::LogLevel::Debug, $($args)*)
    };
}

macro_rules! log_trace {
    ($($args:tt)*) => {
        $crate::util::log!($crate::util::LogLevel::Trace, $($args)*)
    };
}

#[allow(unused_imports, reason = "make every log level available to the crate")]
pub(crate) use {log, log_debug, log_error, log_info, log_trace, log_warn};

pub(crate) fn enabled(level: &LogLevel) -> bool {
    level <= &*LOG_LEVEL
}

pub(crate) fn write_log(level: LogLevel, args: fmt::Arguments<'_>) {
    if level == LogLevel::Error {
        eprintln!("[{level:?}] {args}");
    } else {
        println!("[{level:?}] {args}");
    }
}
