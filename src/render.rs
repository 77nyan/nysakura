//! Half-block rendering: every terminal cell shows two vertically stacked pixels.
//! Only the cells that changed since the last frame are sent to the terminal.

use crate::color::{ansi256, rgb, Px, NONE};

const UPPER: &[u8] = "▀".as_bytes();
const LOWER: &[u8] = "▄".as_bytes();
const FULL: &[u8] = "█".as_bytes();

pub struct Term {
    pub truecolor: bool,
}

pub fn num(out: &mut Vec<u8>, mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    out.extend_from_slice(&buf[i..]);
}

impl Term {
    fn col(&self, out: &mut Vec<u8>, kind: u8, c: Px) {
        num(out, kind as u32);
        if self.truecolor {
            let (r, g, b) = rgb(c);
            out.extend_from_slice(b";2;");
            num(out, r as u32);
            out.push(b';');
            num(out, g as u32);
            out.push(b';');
            num(out, b as u32);
        } else {
            out.extend_from_slice(b";5;");
            num(out, ansi256(c) as u32);
        }
    }

    /// emit cells lo..=hi of one pair of pixel rows
    fn run(&self, t: &[Px], b: &[Px], lo: usize, hi: usize, st: &mut (Px, Px), out: &mut Vec<u8>) {
        for x in lo..=hi {
            let (u, d) = (t[x], b[x]);
            let (ch, f, g): (&[u8], Px, Px) = if u == NONE && d == NONE {
                (b" ", NONE, NONE)
            } else if u == NONE {
                (LOWER, d, NONE)
            } else if d == NONE {
                (UPPER, u, NONE)
            } else if u == d {
                (FULL, u, NONE)
            } else {
                (UPPER, u, d)
            };
            let mark = out.len();
            out.extend_from_slice(b"\x1b[");
            let start = out.len();
            if g == NONE {
                if st.1 != NONE {
                    out.extend_from_slice(b"49");
                    st.1 = NONE;
                }
            } else if g != st.1 {
                self.col(out, 48, g);
                st.1 = g;
            }
            if ch != b" " && f != st.0 {
                if out.len() > start {
                    out.push(b';');
                }
                self.col(out, 38, f);
                st.0 = f;
            }
            if out.len() == start {
                out.truncate(mark);
            } else {
                out.push(b'm');
            }
            out.extend_from_slice(ch);
        }
    }

    /// append the escape codes that turn the previous frame into `cv` (or draw all of `cv`)
    pub fn render(
        &self,
        cv: &[Px],
        prev: Option<&[Px]>,
        cols: usize,
        rows: usize,
        out: &mut Vec<u8>,
    ) {
        let mut st = (NONE, NONE);
        out.extend_from_slice(b"\x1b[0m");
        for r in 0..rows {
            let (t, b) = (
                &cv[2 * r * cols..(2 * r + 1) * cols],
                &cv[(2 * r + 1) * cols..(2 * r + 2) * cols],
            );
            let (lo, hi);
            match prev {
                None => {
                    lo = 0;
                    hi = cols - 1;
                }
                Some(p) => {
                    let (pt, pb) = (
                        &p[2 * r * cols..(2 * r + 1) * cols],
                        &p[(2 * r + 1) * cols..(2 * r + 2) * cols],
                    );
                    if t == pt && b == pb {
                        continue;
                    }
                    let same = |x: usize| t[x] == pt[x] && b[x] == pb[x];
                    let mut l = 0;
                    while same(l) {
                        l += 1;
                    }
                    let mut h = cols - 1;
                    while same(h) {
                        h -= 1;
                    }
                    lo = l;
                    hi = h;
                }
            }
            out.extend_from_slice(b"\x1b[");
            num(out, r as u32 + 1);
            out.push(b';');
            num(out, lo as u32 + 1);
            out.push(b'H');
            self.run(t, b, lo, hi, &mut st, out);
        }
        out.extend_from_slice(b"\x1b[0m");
    }

    /// the finished picture as text lines (no cursor movement), for --print
    pub fn lines(&self, cv: &[Px], cols: usize, rows: usize) -> Vec<u8> {
        let mut lines: Vec<Vec<u8>> = Vec::with_capacity(rows);
        for r in 0..rows {
            let (t, b) = (
                &cv[2 * r * cols..(2 * r + 1) * cols],
                &cv[(2 * r + 1) * cols..(2 * r + 2) * cols],
            );
            let mut hi = cols as isize - 1;
            while hi >= 0 && t[hi as usize] == NONE && b[hi as usize] == NONE {
                hi -= 1;
            }
            let mut line = Vec::new();
            if hi >= 0 {
                self.run(t, b, 0, hi as usize, &mut (NONE, NONE), &mut line);
                line.extend_from_slice(b"\x1b[0m");
            }
            lines.push(line);
        }
        while lines.first().map_or(false, |l| l.is_empty()) {
            lines.remove(0);
        }
        while lines.last().map_or(false, |l| l.is_empty()) {
            lines.pop();
        }
        let mut out = Vec::new();
        for l in lines {
            out.extend_from_slice(&l);
            out.push(b'\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::px;

    #[test]
    fn numbers() {
        let mut v = Vec::new();
        num(&mut v, 0);
        num(&mut v, 255);
        assert_eq!(v, b"0255");
    }

    #[test]
    fn half_blocks() {
        let t = Term { truecolor: true };
        // 2 cols, 1 row (2 pixel rows): top-left red, bottom-left blue, right cell empty
        let cv = vec![px(255, 0, 0), NONE, px(0, 0, 255), NONE];
        let out = String::from_utf8(t.lines(&cv, 2, 1)).unwrap();
        assert!(out.contains('▀'));
        assert!(out.contains("38;2;255;0;0"));
        assert!(out.contains("48;2;0;0;255"));
        assert!(out.ends_with("\x1b[0m\n"));
    }

    #[test]
    fn unchanged_frame_draws_nothing_new() {
        let t = Term { truecolor: false };
        let cv = vec![px(9, 9, 9); 8];
        let mut out = Vec::new();
        t.render(&cv, Some(&cv), 2, 2, &mut out);
        assert_eq!(out, b"\x1b[0m\x1b[0m");
    }
}
