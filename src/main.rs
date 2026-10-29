//! nysakura - watch a sakura tree grow and shed its petals in your terminal.

mod cli;
mod color;
mod render;
mod rng;
mod scene;
mod sys;

use cli::Opts;
use color::{ansi256, theme, Px, NAMES};
use render::Term;
use scene::Scene;
use std::io::Write;
use std::time::{Duration, Instant};

fn die(msg: &str) -> ! {
    eprintln!("nysakura: {}\ntry 'nysakura --help'", msg);
    std::process::exit(2);
}

fn env_num(name: &str) -> Option<usize> {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
}

fn size(o: &Opts, printing: bool) -> (usize, usize) {
    let ts = sys::term_size();
    let c = env_num("COLUMNS").or(ts.map(|t| t.0)).unwrap_or(80);
    let r = env_num("LINES").or(ts.map(|t| t.1)).unwrap_or(24);
    let c = if o.width > 0 { o.width } else { c };
    let r = if o.height > 0 {
        o.height
    } else if printing {
        r.saturating_sub(1).max(8)
    } else {
        r
    };
    (c.max(20), r.max(8))
}

fn truecolor(o: &Opts) -> bool {
    if o.truecolor || o.c256 {
        return o.truecolor;
    }
    let colorterm = std::env::var("COLORTERM").unwrap_or_default();
    let term = std::env::var("TERM").unwrap_or_default();
    colorterm == "truecolor"
        || colorterm == "24bit"
        || ["direct", "kitty", "alacritty", "foot", "wezterm"]
            .iter()
            .any(|k| term.contains(k))
}

fn write_out(bytes: &[u8]) -> bool {
    let out = std::io::stdout();
    let mut lock = out.lock();
    lock.write_all(bytes).and_then(|_| lock.flush()).is_ok()
}

fn do_print(o: &Opts) {
    let (cols, rows) = size(o, true);
    let mut sc = Scene::new(cols, rows, o, o.seed);
    sc.fast_forward(sc.done_t + 6.0);
    let mut cv: Vec<Px> = Vec::new();
    sc.frame(&mut cv);
    write_out(
        &Term {
            truecolor: truecolor(o),
        }
        .lines(&cv, cols, rows),
    );
}

fn do_list(o: &Opts) {
    let tc = truecolor(o);
    let mut out = String::new();
    let mut names: Vec<&str> = NAMES.to_vec();
    names.push("rainbow");
    for n in names {
        let c = theme(n).unwrap_or_else(|| theme("pink").unwrap())[2];
        let (r, g, b) = color::rgb(c);
        let code = if tc {
            format!("38;2;{};{};{}", r, g, b)
        } else {
            format!("38;5;{}", ansi256(c))
        };
        out.push_str(&format!("\x1b[{}m██\x1b[0m -{}\n", code, n));
    }
    write_out(out.as_bytes());
}

fn live(o: &Opts) {
    let tin = sys::is_tty(0);
    sys::install_signals();
    let term = Term {
        truecolor: truecolor(o),
    };
    let fdt = 1.0 / o.fps as f64;
    if tin {
        sys::raw_on();
    }
    sys::enter_screen();
    let (mut cols, mut rows) = size(o, false);
    let mut sc = Scene::new(cols, rows, o, o.seed);
    if o.instant {
        sc.fast_forward(sc.done_t);
    }
    let (mut cur, mut prev): (Vec<Px>, Vec<Px>) = (Vec::new(), Vec::new());
    let mut have_prev = false;
    let mut out: Vec<u8> = Vec::with_capacity(1 << 16);
    let mut paused = false;
    let mut stdin_open = tin;
    let start = Instant::now();
    let mut last = start;
    loop {
        if sys::stopped() {
            break;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64().min(0.1);
        last = now;
        let now_size = size(o, false);
        if now_size != (cols, rows) {
            let (c, r) = now_size;
            let t_old = sc.t;
            let seed = sc.seed;
            cols = c;
            rows = r;
            sc = Scene::new(cols, rows, o, Some(seed));
            sc.fast_forward(t_old);
            have_prev = false;
            out.clear();
            out.extend_from_slice(b"\x1b[2J");
            write_out(&out);
        }
        if !paused {
            sc.step(dt * o.speed);
        }
        sc.frame(&mut cur);
        out.clear();
        out.extend_from_slice(b"\x1b[?2026h");
        term.render(
            &cur,
            if have_prev { Some(&prev) } else { None },
            cols,
            rows,
            &mut out,
        );
        out.extend_from_slice(b"\x1b[?2026l");
        if !write_out(&out) {
            break; // the terminal went away
        }
        std::mem::swap(&mut cur, &mut prev);
        have_prev = true;
        let mut regrow = o.cycle > 0.0 && sc.t > sc.done_t + o.cycle;
        if o.time > 0.0 && now.duration_since(start).as_secs_f64() >= o.time {
            break;
        }
        let left = (fdt - Instant::now().duration_since(now).as_secs_f64()).max(0.0);
        if stdin_open {
            match sys::wait_input((left * 1000.0).ceil() as i32) {
                sys::Input::Keys(buf, n) => {
                    let data = &buf[..n];
                    if data == b"\x1b" || matches!(data[0], b'q' | b'Q' | 3) {
                        break;
                    }
                    match data[0] {
                        b' ' => paused = !paused,
                        b'r' | b'R' => regrow = true,
                        _ => {}
                    }
                }
                sys::Input::Eof => stdin_open = false,
                sys::Input::Nothing => {}
            }
        } else {
            std::thread::sleep(Duration::from_secs_f64(left));
        }
        if regrow {
            sc = Scene::new(cols, rows, o, None);
            have_prev = false;
            out.clear();
            out.extend_from_slice(b"\x1b[2J");
            write_out(&out);
        }
    }
    sys::leave_screen();
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        sys::leave_screen();
        eprintln!("nysakura: internal error: {}", info);
    }));
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let o = match cli::parse(&args) {
        Ok(o) => o,
        Err(e) => die(&e),
    };
    if o.help {
        write_out(cli::help().as_bytes());
    } else if o.version {
        println!("nysakura {}", cli::VERSION);
    } else if o.list {
        do_list(&o);
    } else if o.print || !sys::is_tty(1) {
        do_print(&o);
    } else {
        live(&o);
    }
}
