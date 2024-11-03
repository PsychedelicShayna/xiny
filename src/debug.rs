use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{self, Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dirs;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log {
    ($level:tt => $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use std::io::{BufWriter, Read, Write};
            let cache_dir = dirs::cache_dir().unwrap();

            let cache_dir = cache_dir.join("xiny");
            let cache_file = cache_dir.join("xiny.log");

            if !std::fs::exists(&cache_dir).unwrap_or(false) {
                std::fs::create_dir_all(&cache_dir).unwrap();
            }

            if !std::fs::exists(&cache_file).unwrap_or(false) {
                std::fs::File::create(&cache_file).unwrap();
            }

            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&cache_file)
                .unwrap();

            let mut writer = BufWriter::new(&file);

            let elapsed_millis =  unsafe {
                START_TIME
                    .as_ref()
                    .expect("START_TIME not set")  // This *should* be safe since main *should* set it.
                    .elapsed()
                    .as_secs_f64()
            };

            let separator = "-".to_string().repeat(30);

            let level = LogLevel::$level;

            let message = format!("{}\n{}| {}:{} | +{:.8} | {}\n", separator, level, file!(), line!(),elapsed_millis, format!($($arg)*));
            writer.write_all(message.as_bytes()).unwrap();
        }
    };
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use std::io::{BufWriter, Read, Write};
            let cache_dir = dirs::cache_dir().unwrap();

            let cache_dir = cache_dir.join("xiny");
            let cache_file = cache_dir.join("xiny.log");

            if !std::fs::exists(&cache_dir).unwrap_or(false) {
                std::fs::create_dir_all(&cache_dir).unwrap();
            }

            if !std::fs::exists(&cache_file).unwrap_or(false) {
                std::fs::File::create(&cache_file).unwrap();
            }

            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&cache_file)
                .unwrap();

            let mut writer = BufWriter::new(&file);

            let elapsed_millis =  unsafe {
                START_TIME
                    .as_ref()
                    .expect("START_TIME not set")  // This *should* be safe since main *should* set it.
                    .elapsed()
                    .as_secs_f64()
            };

            let separator = "-".to_string().repeat(30);

            let message = format!("{}: {}\n", elapsed_millis, format!($($arg)*));
            writer.write_all(message.as_bytes()).unwrap();
        }
    };
}

/// Since interactive mode basically takes over the terminal in raw mode, I
/// created a simple logger for debugging purposes, so I can at least output
/// to a file and `watch -n 0.1 cat filename.log`, you know how it goes.

#[cfg(debug_assertions)]
pub static mut START_TIME: Option<Instant> = None;

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(debug_assertions)]
impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, " DEBUG "),
            LogLevel::Info => write!(f, " INFO  "),
            LogLevel::Warn => write!(f, " WARN  "),
            LogLevel::Error => write!(f, " ERROR "),
        }
    }
}

#[cfg(debug_assertions)]
pub struct Log;

#[cfg(debug_assertions)]
impl Log {
    pub fn error(message: &str) {
        log!(Error => "{}", message);
    }

    pub fn warn(message: &str) {
        log!(Warn => "{}", message);
    }

    pub fn info(message: &str) {
        log!(Info => "{}", message);
    }

    pub fn debug(message: &str) {
        log!(Debug => "{}", message);
    }

    pub fn path() -> PathBuf {
        let cache_dir = dirs::cache_dir().unwrap();
        let cache_dir = cache_dir.join("xiny");
        let cache_file = cache_dir.join("xiny.log");

        cache_file
    }

    pub fn clear() {
        let cache_dir = dirs::cache_dir().unwrap();
        let cache_dir = cache_dir.join("xiny");
        let cache_file = cache_dir.join("xiny.log");

        if fs::exists(&cache_file).unwrap_or(false) {
            fs::remove_file(&cache_file).unwrap();
        }
    }
}

mod test {

    #[test]
    fn test_log() {
        use super::*;

        unsafe {
            START_TIME = Some(Instant::now());
        }

        log!(Debug => "This is a debug message {}", 1);
        std::thread::sleep(std::time::Duration::from_secs(1));
        log!(Info => "This is an info message {}", 2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        log!(Warn => "This is a warning message {}", 3);
        std::thread::sleep(std::time::Duration::from_secs(3));
        log!(Error => "This is an error message {}", 4);
    }
}
