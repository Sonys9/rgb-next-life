use crate::log::Serial;

#[macro_export]
macro_rules! terminal {
    ($($data:expr),* $(,)?) => {
        {
            use core::fmt::Write;
            let mut writer = crate::log::Terminal;
            let _ = write!(writer, $($data),*);

            // if let Ok(serial) = crate::log::Serial::get() {
            //     if let Some(serial) = &*serial {
            //         let _ = serial.flush();
            //     }
            // }
        }
    };
}

macro_rules! log_type {
    ($name:ident, $title:expr) => {
        #[macro_export]
        macro_rules! $name {
            ($$($$data:expr),* $$(,)?) => {
                terminal!("[{}] {}\n", $title, format_args!($$($$data),*))
            }
	}
    };
}

log_type!(info, "\x1b[32mℹ️  INFO\x1b[0m");
log_type!(error, "\x1b[31m⛔  ERROR\x1b[0m");
log_type!(warn, "\x1b[33m⚠️  WARN\x1b[0m");

pub struct Terminal;

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let Ok(mut serial) = Serial::get() else {
            return Ok(());
        };
        let Some(ref serial) = *&serial.as_mut() else {
            return Ok(());
        };
        let _ = serial.print(s.as_bytes());
        Ok(())
    }
}
