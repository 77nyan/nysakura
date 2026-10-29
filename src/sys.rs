//! The few libc calls nysakura needs, declared by hand so the crate has no dependencies.
//! Linux only: terminal size, cbreak mode, key polling, signals.

#[cfg(not(target_os = "linux"))]
compile_error!("nysakura supports Linux only");

use std::os::raw::{c_int, c_ulong, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[repr(C)]
struct WinSize {
    rows: u16,
    cols: u16,
    xpix: u16,
    ypix: u16,
}

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}

/// `struct termios` is 60 bytes on Linux (glibc and musl). We keep a bigger opaque block and only
/// touch `c_lflag`, which sits at byte 12 on every Linux architecture.
#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct Termios([u8; 128]);

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn isatty(fd: c_int) -> c_int;
    fn tcgetattr(fd: c_int, t: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, action: c_int, t: *const Termios) -> c_int;
    fn poll(fds: *mut PollFd, n: c_ulong, timeout: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, n: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, n: usize) -> isize;
    fn signal(sig: c_int, handler: usize) -> usize;
}

#[cfg(not(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "sparc",
    target_arch = "sparc64"
)))]
const TIOCGWINSZ: c_ulong = 0x5413;
#[cfg(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "sparc",
    target_arch = "sparc64"
))]
const TIOCGWINSZ: c_ulong = 0x4008_7468;

const ISIG: u32 = 0o1;
const ICANON: u32 = 0o2;
const ECHO: u32 = 0o10;
const TCSADRAIN: c_int = 1;
const POLLIN: i16 = 1;

static SAVED: Mutex<Option<Termios>> = Mutex::new(None);
static IN_SCREEN: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);

pub fn is_tty(fd: i32) -> bool {
    unsafe { isatty(fd) == 1 }
}

/// (columns, rows) of the terminal, if we are attached to one
pub fn term_size() -> Option<(usize, usize)> {
    for fd in [1, 2, 0] {
        let mut ws = WinSize {
            rows: 0,
            cols: 0,
            xpix: 0,
            ypix: 0,
        };
        let ok = unsafe { ioctl(fd, TIOCGWINSZ, &mut ws as *mut WinSize) } == 0;
        if ok && ws.cols > 0 && ws.rows > 0 {
            return Some((ws.cols as usize, ws.rows as usize));
        }
    }
    None
}

fn lflag(t: &mut Termios) -> &mut u32 {
    unsafe { &mut *(t.0.as_mut_ptr().add(12) as *mut u32) }
}

/// keys arrive one by one, without Enter, without echo (Ctrl+C arrives as a key too)
pub fn raw_on() -> bool {
    let mut t = Termios([0; 128]);
    if unsafe { tcgetattr(0, &mut t) } != 0 {
        return false;
    }
    if let Ok(mut g) = SAVED.lock() {
        *g = Some(t);
    }
    *lflag(&mut t) &= !(ICANON | ECHO | ISIG);
    unsafe { tcsetattr(0, TCSADRAIN, &t) == 0 }
}

pub fn raw_off() {
    let saved = SAVED.lock().ok().and_then(|mut g| g.take());
    if let Some(t) = saved {
        unsafe {
            tcsetattr(0, TCSADRAIN, &t);
        }
    }
}

fn raw_write(b: &[u8]) {
    unsafe {
        write(1, b.as_ptr() as *const c_void, b.len());
    }
}

pub fn enter_screen() {
    raw_write(b"\x1b[?1049h\x1b[?25l\x1b[2J");
    IN_SCREEN.store(true, Ordering::SeqCst);
}

/// safe to call any number of times, also from the panic hook
pub fn leave_screen() {
    if IN_SCREEN.swap(false, Ordering::SeqCst) {
        raw_write(b"\x1b[0m\x1b[?25h\x1b[?1049l");
    }
    raw_off();
}

extern "C" fn on_signal(_: c_int) {
    STOP.store(true, Ordering::SeqCst);
}

/// Ctrl+C, `kill` and a closing terminal end the program cleanly
pub fn install_signals() {
    for sig in [1, 2, 15] {
        unsafe {
            signal(sig, on_signal as extern "C" fn(c_int) as usize);
        }
    }
}

pub fn stopped() -> bool {
    STOP.load(Ordering::SeqCst)
}

pub enum Input {
    Nothing,
    Eof,
    Keys([u8; 64], usize),
}

/// wait up to `ms` milliseconds for a key press on stdin
pub fn wait_input(ms: i32) -> Input {
    let mut p = PollFd {
        fd: 0,
        events: POLLIN,
        revents: 0,
    };
    if unsafe { poll(&mut p, 1, ms.max(0)) } <= 0 || p.revents == 0 {
        return Input::Nothing;
    }
    let mut buf = [0u8; 64];
    let n = unsafe { read(0, buf.as_mut_ptr() as *mut c_void, buf.len()) };
    if n > 0 {
        Input::Keys(buf, n as usize)
    } else if n == 0 {
        Input::Eof
    } else {
        Input::Nothing
    }
}
