//! The tree: procedural branches, blossom clusters, the growth timeline and the falling petals.

use crate::cli::Opts;
use crate::color::{mix, px, ramp, shade, theme, Pal, Px, NONE, RAINBOW};
use crate::rng::{fresh_seed, Rng};

const WHITE: Px = 0x01FF_FFFF;

struct Branch {
    path: Vec<(f32, f32, f32)>, // x, y, radius
    col: Px,
    t0: f64,
    dur: f64,
    done: usize,
}

struct Cluster {
    x: f64,
    y: f64,
    r: f64,
    branch: usize,
    s: f64,
}

/// one blossom pixel and the moment it appears
struct Blossom {
    t: f32,
    x: i16,
    y: i16,
    col: Px,
}

struct Petal {
    x: f64,
    y: f64,
    vy: f64,
    amp: f64,
    om: f64,
    ph: f64,
    age: f64,
    col: Px,
    col2: Px,
}

struct Spark {
    x: i32,
    y: i32,
    age: f64,
}

pub struct Scene {
    w: usize,
    h: usize,
    pub seed: u64,
    tall: bool,
    scale: f64,
    grow: f64,
    wind_k: f64,
    prng: Rng,
    canvas: Vec<Px>,
    pub t: f64,
    pals: Vec<Pal>,
    deep: Px,
    n_petals: usize,
    fall: f64,
    petals: Vec<Petal>,
    sparks: Vec<Spark>,
    bloomed: Vec<(i16, i16)>,
    spawn_acc: f64,
    spark_acc: f64,
    branches: Vec<Branch>,
    next_b: usize,
    running: Vec<usize>,
    blossoms: Vec<Blossom>,
    next_p: usize,
    pub done_t: f64,
}

/// state while generating one tree
struct Gen {
    rng: Rng,
    hx: f64,
    depth_max: i32,
    rb: f64,
    tall: bool,
    deep: Px,
    br: Vec<Branch>,
    cl: Vec<Cluster>,
    bb: [f64; 4], // min x, max x, min y, (max y)
}

impl Gen {
    fn add_cluster(&mut self, x: f64, y: f64, r: f64, branch: usize, s: f64) {
        self.cl.push(Cluster { x, y, r, branch, s });
        self.bb[0] = self.bb[0].min(x - r * 1.4);
        self.bb[1] = self.bb[1].max(x + r * 1.4);
        self.bb[2] = self.bb[2].min(y - r * 1.4);
        self.bb[3] = self.bb[3].max(y + r * 1.4);
    }

    #[allow(clippy::too_many_arguments)]
    fn make(&mut self, x: f64, y: f64, ang: f64, length: f64, th: f64, depth: i32, t0: f64) {
        let n = ((length / 0.7) as i64).max(2) as usize;
        let bend = self.rng.uniform(-0.5, 0.5) * if depth == 0 { 0.5 } else { 1.0 };
        let th_end = (th * 0.72).max(1.0);
        let step = length / n as f64;
        let mut path: Vec<(f64, f64, f64)> = Vec::with_capacity(n + 1);
        let (mut px_, mut py) = (x, y);
        for i in 0..=n {
            let s = i as f64 / n as f64;
            path.push((px_, py, (th + (th_end - th) * s) / 2.0));
            let a = ang + bend * s;
            px_ += a.sin() * step * self.hx;
            py -= a.cos() * step;
        }
        let k = (depth as f64 / (self.depth_max - 1).max(1) as f64).min(1.0);
        let col = mix(mix(px(70, 46, 40), px(128, 92, 76), k), self.deep, 0.08);
        let bi = self.br.len();
        self.br.push(Branch {
            path: path
                .iter()
                .map(|&(a, b, c)| (a as f32, b as f32, c as f32))
                .collect(),
            col,
            t0,
            dur: length,
            done: 0,
        });
        for &(qx, qy, _) in path.iter().step_by(3) {
            self.bb[0] = self.bb[0].min(qx);
            self.bb[1] = self.bb[1].max(qx);
            self.bb[2] = self.bb[2].min(qy);
        }
        let (ex, ey) = (path[n].0, path[n].1);
        if depth >= self.depth_max - 1 || length < 3.2 {
            let r = self.rb * self.rng.uniform(0.9, 1.3);
            self.add_cluster(ex, ey, r, bi, 1.0);
            if self.rng.f() < 0.55 {
                let k = self.rng.range((n / 2) as i64, n as i64) as usize;
                let r = self.rb * self.rng.uniform(0.7, 1.0);
                self.add_cluster(path[k].0, path[k].1, r, bi, k as f64 / n as f64);
            }
            return;
        }
        if depth >= 1 && self.rng.f() < 0.6 {
            // side twig
            let k = (n as f64 * self.rng.uniform(0.45, 0.8)) as usize;
            let s = k as f64 / n as f64;
            let sign = if self.rng.f() < 0.5 { -1.0 } else { 1.0 };
            let a = ang + bend * s + sign * self.rng.uniform(0.5, 1.0);
            let len = length * self.rng.uniform(0.4, 0.6);
            self.make(
                path[k].0,
                path[k].1,
                a,
                len,
                (th * 0.55).max(1.0),
                depth + 1,
                t0 + length * (1.0 - (1.0 - s).sqrt()),
            );
        }
        let mut kids: Vec<f64>;
        let mut lf: (f64, f64);
        if depth == 0 {
            kids = vec![self.rng.uniform(-0.8, -0.45), self.rng.uniform(0.45, 0.8)];
            if self.rng.f() < 0.5 {
                let mid = self.rng.uniform(-0.2, 0.2);
                kids.insert(1, mid);
            }
            lf = (0.9, 1.0);
        } else {
            let sp = self.rng.uniform(0.3, 0.65);
            let (a, b) = (self.rng.uniform(0.6, 1.2), self.rng.uniform(0.6, 1.2));
            kids = vec![-sp * a, sp * b];
            if self.rng.f() < 0.12 {
                let mid = self.rng.uniform(-0.15, 0.15);
                kids.push(mid);
            }
            lf = (0.7, 0.84);
        }
        if self.tall {
            lf = (lf.0 + 0.1, lf.1 + 0.08); // longer branches
        }
        if depth >= 2 && self.rng.f() < 0.35 {
            let k = (n as f64 * 0.8) as usize;
            self.add_cluster(path[k].0, path[k].1, self.rb * 0.8, bi, k as f64 / n as f64);
        }
        for da in kids {
            let len = length * self.rng.uniform(lf.0, lf.1);
            let thick = (th_end * self.rng.uniform(0.9, 1.0)).max(1.0);
            self.make(ex, ey, ang + bend + da, len, thick, depth + 1, t0 + length);
        }
    }
}

impl Scene {
    pub fn new(cols: usize, rows: usize, o: &Opts, seed: Option<u64>) -> Scene {
        let (w, h) = (cols, rows * 2);
        let seed = seed.unwrap_or_else(fresh_seed);
        let pals: Vec<Pal> = if let Some(hex) = &o.hex {
            let hx: String = if hex.len() == 3 {
                hex.chars().flat_map(|c| [c, c]).collect()
            } else {
                hex.clone()
            };
            let v = |i: usize| u8::from_str_radix(&hx[i..i + 2], 16).unwrap_or(0);
            vec![ramp(px(v(0), v(2), v(4)))]
        } else if o.color == "rainbow" {
            RAINBOW.iter().map(|n| theme(n).unwrap()).collect()
        } else {
            vec![theme(&o.color).unwrap()]
        };
        let deep = pals[0][3];
        let mut s = Scene {
            w,
            h,
            seed,
            tall: o.tall,
            scale: o.scale,
            grow: o.grow,
            wind_k: o.wind,
            prng: Rng::new(seed ^ 0x5EED),
            canvas: vec![NONE; w * h],
            t: 0.0,
            pals,
            deep,
            n_petals: o.petals.unwrap_or_else(|| (w / 2).clamp(30, 140)),
            fall: (h as f64 * 0.1).max(4.0),
            petals: Vec::new(),
            sparks: Vec::new(),
            bloomed: Vec::new(),
            spawn_acc: 0.0,
            spark_acc: 0.0,
            branches: Vec::new(),
            next_b: 0,
            running: Vec::new(),
            blossoms: Vec::new(),
            next_p: 0,
            done_t: 0.0,
        };
        s.build_tree();
        s
    }

    fn gen(&self, scale: f64) -> Gen {
        let (w, h) = (self.w as f64, self.h as f64);
        let hx = if self.tall {
            (0.62 * w / h).clamp(0.3, 0.6)
        } else {
            1.0
        };
        let l0 = h * if self.tall { 0.32 } else { 0.2 } * scale;
        let depth_max = 5 + (h >= 56.0) as i32 + (h >= 90.0) as i32 + (h >= 130.0) as i32;
        let rb = (h * 0.05).clamp(2.8, 6.5) * scale.clamp(0.8, 1.4);
        let mut g = Gen {
            rng: Rng::new(self.seed),
            hx,
            depth_max,
            rb,
            tall: self.tall,
            deep: self.deep,
            br: Vec::new(),
            cl: Vec::new(),
            bb: [w, 0.0, h, 0.0],
        };
        let lean = g.rng.uniform(-0.08, 0.08);
        g.make(w / 2.0, h, lean, l0, (l0 * 0.3).max(2.0), 0, 0.0); // the trunk grows from the bottom edge
        g
    }

    fn build_tree(&mut self) {
        let tall = self.tall;
        let (w, h) = (self.w as f64, self.h as f64);
        let mut scale = (if tall { 2.4 } else { 1.35 }) * self.scale; // start big, shrink until it fits
        let tries = if tall { 140 } else { 60 };
        let mut g = self.gen(scale);
        for _ in 1..tries {
            if g.bb[0] >= w * 0.04 && g.bb[1] <= w * 0.96 && g.bb[2] >= h * 0.05 {
                break;
            }
            scale *= if tall { 0.97 } else { 0.95 };
            g = self.gen(scale);
        }
        let grow = self.grow;
        let end = g.br.iter().map(|b| b.t0 + b.dur).fold(0.0, f64::max);
        let k = 0.68 * grow / end.max(1e-9); // bare branches first, blossoms follow
        for b in g.br.iter_mut() {
            b.t0 *= k;
            b.dur *= k;
        }
        let mut blossoms: Vec<Blossom> = Vec::new();
        let rng = &mut g.rng;
        for c in &g.cl {
            let pal = rng.pick(&self.pals);
            let b = &g.br[c.branch];
            let mut tc = b.t0
                + b.dur
                    * if c.s >= 1.0 {
                        1.0
                    } else {
                        1.0 - (1.0 - c.s).sqrt()
                    };
            tc = 0.36 * grow + tc / (0.68 * grow) * 0.5 * grow + rng.uniform(0.0, 0.6);
            let r = c.r;
            let mut blobs = vec![(0.0, 0.0, r)];
            for _ in 0..4 {
                let (a, b2, c2) = (
                    rng.uniform(-0.6, 0.6) * r,
                    rng.uniform(-0.6, 0.6) * r,
                    r * rng.uniform(0.4, 0.65),
                );
                blobs.push((a, b2, c2));
            }
            for (bx, by, br) in blobs {
                for _ in 0..((br * br * 2.6) as i32 + 2) {
                    let a = rng.uniform(0.0, std::f64::consts::TAU);
                    let d = br * rng.f().sqrt();
                    let (dx, dy) = (bx + a.cos() * d, by + a.sin() * d * 0.85);
                    if rng.f() < 0.15 {
                        continue;
                    }
                    let sh = (dx * 0.45 + dy * 0.75) / r + rng.gauss(0.0, 0.35);
                    let ci = (((sh + 1.3) / 2.6 * 5.0) as i32).clamp(0, 4) as usize;
                    blossoms.push(Blossom {
                        t: (tc + rng.uniform(0.0, 1.4)) as f32,
                        x: (c.x + dx) as i16,
                        y: (c.y + dy) as i16,
                        col: pal[ci],
                    });
                }
            }
        }
        blossoms.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
        g.br.sort_by(|a, b| a.t0.partial_cmp(&b.t0).unwrap_or(std::cmp::Ordering::Equal));
        self.done_t = grow.max(blossoms.last().map_or(0.0, |p| p.t as f64)) + 0.5;
        self.branches = g.br;
        self.blossoms = blossoms;
    }

    // --- simulation
    fn stamp(&mut self, x: f64, y: f64, r: f64, c: Px) {
        let (w, h) = (self.w as i32, self.h as i32);
        let (xi, yi) = (x.round() as i32, y.round() as i32);
        if r < 0.8 {
            if xi >= 0 && xi < w && yi >= 0 && yi < h {
                self.canvas[(yi * w + xi) as usize] = c;
            }
            return;
        }
        let ri = r.ceil() as i32;
        for dy in -ri..=ri {
            let yy = yi + dy;
            if yy < 0 || yy >= h {
                continue;
            }
            for dx in -ri..=ri {
                let xx = xi + dx;
                if xx < 0 || xx >= w {
                    continue;
                }
                if ((dx * dx + dy * dy) as f64) <= r * r + 0.3 {
                    let tl = (dx as f64 * 0.8 + dy as f64 * 0.6) / (r + 0.5);
                    self.canvas[(yy * w + xx) as usize] = shade(c, 1.15 - 0.3 * (0.5 + 0.5 * tl));
                }
            }
        }
    }

    fn wind(&self) -> f64 {
        let t = self.t;
        self.wind_k * (3.2 + 2.8 * (0.31 * t).sin() + 1.6 * (0.83 * t + 1.3).sin())
    }

    pub fn step(&mut self, dt: f64) {
        self.t += dt;
        let (t, w, h) = (self.t, self.w as i32, self.h as i32);
        while self.next_b < self.branches.len() && self.branches[self.next_b].t0 <= t {
            self.running.push(self.next_b);
            self.next_b += 1;
        }
        let running = std::mem::take(&mut self.running);
        let mut still = Vec::with_capacity(running.len());
        for bi in running {
            let (u, len, done, col) = {
                let b = &self.branches[bi];
                ((t - b.t0) / b.dur, b.path.len(), b.done, b.col)
            };
            let idx = if u >= 1.0 {
                len
            } else {
                ((1.0 - (1.0 - u) * (1.0 - u)) * len as f64) as usize
            };
            for i in done..idx {
                let (x, y, r) = self.branches[bi].path[i];
                self.stamp(x as f64, y as f64, r as f64, col);
            }
            self.branches[bi].done = idx;
            if idx < len {
                still.push(bi);
            }
        }
        self.running = still;
        while self.next_p < self.blossoms.len() && (self.blossoms[self.next_p].t as f64) <= t {
            let p = &self.blossoms[self.next_p];
            let (x, y, col) = (p.x as i32, p.y as i32, p.col);
            if x >= 0 && x < w && y >= 0 && y < h {
                self.canvas[(y * w + x) as usize] = col;
                if self.next_p % 3 == 0 {
                    self.bloomed.push((p.x, p.y));
                }
            }
            self.next_p += 1;
        }
        self.update_petals(dt);
    }

    fn update_petals(&mut self, dt: f64) {
        let (w, h) = (self.w as f64, self.h as f64);
        let frac = self.next_p as f64 / self.blossoms.len().max(1) as f64;
        let target = (self.n_petals as f64 * ((frac - 0.35) / 0.5).clamp(0.0, 1.0)) as usize;
        if !self.bloomed.is_empty() && self.petals.len() < target {
            self.spawn_acc += dt * self.n_petals as f64 / 6.0;
            while self.spawn_acc >= 1.0 && self.petals.len() < target {
                let r = &mut self.prng;
                let (bx, by) = r.pick(&self.bloomed);
                let pal = r.pick(&self.pals);
                let petal = Petal {
                    x: bx as f64 + r.uniform(-1.0, 1.0),
                    y: by as f64 + r.uniform(0.0, 2.0),
                    vy: self.fall * r.uniform(0.7, 1.3),
                    amp: r.uniform(1.0, 2.6),
                    om: r.uniform(1.4, 3.2),
                    ph: r.uniform(0.0, std::f64::consts::TAU),
                    age: 0.0,
                    col: pal[r.pick(&[0usize, 0, 1, 1, 2, 3])],
                    col2: 0,
                };
                self.petals.push(Petal {
                    col2: shade(petal.col, 0.86),
                    ..petal
                });
                self.spawn_acc -= 1.0;
            }
            self.spawn_acc = self.spawn_acc.min(1.0);
        }
        if !self.bloomed.is_empty() {
            // little glints in the canopy
            self.spark_acc += dt * 5.0;
            while self.spark_acc >= 1.0 {
                let (x, y) = self.prng.pick(&self.bloomed);
                self.sparks.push(Spark {
                    x: x as i32,
                    y: y as i32,
                    age: 0.0,
                });
                self.spark_acc -= 1.0;
            }
        }
        for s in self.sparks.iter_mut() {
            s.age += dt;
        }
        self.sparks.retain(|s| s.age < 0.6);
        let wind = self.wind();
        for p in self.petals.iter_mut() {
            p.age += dt;
            p.x += (wind + p.amp * p.om * (p.om * p.age + p.ph).cos()) * dt;
            p.y += p.vy * dt * (1.0 + 0.3 * (p.om * 1.7 * p.age + p.ph).sin());
        }
        self.petals.retain(|p| p.x >= 0.0 && p.x < w && p.y < h); // blown away or fallen off the bottom
    }

    pub fn fast_forward(&mut self, to: f64) {
        while self.t < to {
            let dt = (1.0f64 / 30.0).min(to - self.t);
            self.step(dt);
        }
    }

    /// the picture at this moment: canvas + petals + glints, written into `cv`
    pub fn frame(&self, cv: &mut Vec<Px>) {
        let (w, h) = (self.w as i32, self.h as i32);
        cv.clear();
        cv.extend_from_slice(&self.canvas);
        for p in &self.petals {
            let (x, y) = (p.x as i32, p.y as i32);
            if x >= 0 && x < w && y >= 0 && y < h {
                cv[(y * w + x) as usize] = p.col;
                if x + 1 < w && (p.age * p.om * 2.0 + p.ph).sin() > 0.5 {
                    cv[(y * w + x + 1) as usize] = p.col2;
                }
            }
        }
        for s in &self.sparks {
            let i = (s.y * w + s.x) as usize;
            let c = cv[i];
            if c != NONE {
                cv[i] = mix(c, WHITE, 0.75 * (std::f64::consts::PI * s.age / 0.6).sin());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finished(cols: usize, rows: usize, o: &Opts, seed: u64) -> Vec<Px> {
        let mut s = Scene::new(cols, rows, o, Some(seed));
        s.fast_forward(s.done_t + 6.0);
        let mut cv = Vec::new();
        s.frame(&mut cv);
        cv
    }

    #[test]
    fn same_seed_same_tree() {
        let o = Opts::default();
        assert!(finished(100, 40, &o, 7) == finished(100, 40, &o, 7));
        assert!(finished(100, 40, &o, 7) != finished(100, 40, &o, 8));
    }

    #[test]
    fn tree_stays_inside_the_window_and_touches_the_bottom() {
        for (c, r) in [(60, 20), (100, 50), (200, 50), (40, 60)] {
            for tall in [true, false] {
                let o = Opts {
                    tall,
                    ..Opts::default()
                };
                let cv = finished(c, r, &o, 3);
                let w = c;
                let h = r * 2;
                assert_ne!(
                    cv[(h - 1) * w..].iter().filter(|&&p| p != NONE).count(),
                    0,
                    "{}x{} tall={}",
                    c,
                    r,
                    tall
                );
                let top = (0..h)
                    .find(|&y| cv[y * w..(y + 1) * w].iter().any(|&p| p != NONE))
                    .unwrap();
                // -mini is deliberately short in narrow windows; the default tall tree must fill them
                assert!(
                    !tall || top < h * 7 / 10,
                    "tree too short: {}x{} tall={} top={}",
                    c,
                    r,
                    tall,
                    top
                );
            }
        }
    }

    #[test]
    fn tall_is_taller_than_mini() {
        let (c, r) = (100, 50);
        let top = |tall: bool| {
            let o = Opts {
                tall,
                ..Opts::default()
            };
            let cv = finished(c, r, &o, 5);
            (0..r * 2)
                .find(|&y| cv[y * c..(y + 1) * c].iter().any(|&p| p != NONE))
                .unwrap()
        };
        assert!(top(true) < top(false));
    }

    #[test]
    fn petals_fall_after_blooming() {
        let o = Opts::default();
        let mut s = Scene::new(120, 40, &o, Some(2));
        s.fast_forward(s.done_t + 3.0);
        assert!(!s.petals.is_empty());
    }
}
