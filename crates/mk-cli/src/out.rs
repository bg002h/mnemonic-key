//! The one stdout path. Every `print!`/`println!` in mk-cli resolves to the
//! macros below (declared `#[macro_use]` first in `main.rs`, so they shadow
//! std's in every module), and so do `eprint!`/`eprintln!`.
//!
//! std's macros panic (exit 101) when a write fails. Here a write error stops
//! the output instead. A reader that closed the pipe (`mk … | head -3`,
//! `ErrorKind::BrokenPipe`) only stops it: the exit code is the verb's own.
//! Any other stdout error (a full disk, `> /dev/full`) is kept, and `main`
//! prints "mk: cannot write the output: <error>" and exits 1 (crash sweep F1,
//! grade-mk-crash-sweep-1). A stderr write error is ignored.
//! `tests/output_errors.rs` pins both cases and that no other file of the
//! crate names stdout.

use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static CLOSED: AtomicBool = AtomicBool::new(false);
static WRITE_ERROR: OnceLock<String> = OnceLock::new();

/// Writes to stdout; after the first failure, writes nothing.
pub(crate) fn stdout_write(args: std::fmt::Arguments<'_>) {
    if CLOSED.load(Ordering::Relaxed) || read_only() {
        return;
    }
    let mut o = std::io::stdout().lock();
    if let Err(e) = o.write_fmt(args).and_then(|()| o.flush()) {
        failed(&e);
    }
}

/// std's stdout reports EBADF as a successful write, so a stdout opened
/// read-only (`1</dev/null`) would lose the output with exit 0. Checked once,
/// at the first write (or clap's `--help`/`--version`): then EBADF is recorded
/// as the write error. A verb that writes nothing to stdout is not touched.
pub(crate) fn read_only() -> bool {
    static RO: OnceLock<bool> = OnceLock::new();
    *RO.get_or_init(|| {
        #[cfg(unix)]
        {
            // SAFETY: fcntl(F_GETFL) takes no pointer; a bad fd returns -1.
            let fl = unsafe { libc::fcntl(1, libc::F_GETFL) };
            let ro = fl >= 0 && (fl & libc::O_ACCMODE) == libc::O_RDONLY;
            if ro {
                failed(&std::io::Error::from_raw_os_error(libc::EBADF));
            }
            ro
        }
        // Windows: std turns only an invalid handle into a success, so a
        // read-only handle's error reaches the sink as written.
        #[cfg(not(unix))]
        {
            false
        }
    })
}

/// Records a failed stdout write (also used for clap's `--help`/`--version`).
pub(crate) fn failed(e: &std::io::Error) {
    CLOSED.store(true, Ordering::Relaxed);
    if e.kind() != std::io::ErrorKind::BrokenPipe {
        let _ = WRITE_ERROR.set(e.to_string());
    }
}

/// Writes to stderr, ignoring a failure.
pub(crate) fn stderr_write(args: std::fmt::Arguments<'_>) {
    let _ = std::io::stderr().lock().write_fmt(args);
}

/// The process exit code: `code`, unless a stdout write failed for a reason
/// other than a closed pipe. Then the error is printed, and a code that says
/// the output is the result (0, and `repair`'s 5) becomes 1; a failing code
/// is kept.
pub(crate) fn exit_code(code: u8) -> u8 {
    match WRITE_ERROR.get() {
        None => code,
        Some(e) => {
            stderr_write(format_args!("mk: cannot write the output: {e}\n"));
            if code == 0 || code == 5 { 1 } else { code }
        }
    }
}

macro_rules! print {
    ($($a:tt)*) => { $crate::out::stdout_write(format_args!($($a)*)) };
}
macro_rules! println {
    () => { $crate::out::stdout_write(format_args!("\n")) };
    ($($a:tt)*) => { $crate::out::stdout_write(format_args!("{}\n", format_args!($($a)*))) };
}
// Unused today; defined so a future `eprint!` cannot reach std's.
#[allow(unused_macros)]
macro_rules! eprint {
    ($($a:tt)*) => { $crate::out::stderr_write(format_args!($($a)*)) };
}
macro_rules! eprintln {
    () => { $crate::out::stderr_write(format_args!("\n")) };
    ($($a:tt)*) => { $crate::out::stderr_write(format_args!("{}\n", format_args!($($a)*))) };
}
