//! Colors are packed into one `u32`. `NONE` (0) means "nothing here / transparent".

pub type Px = u32;
pub type Pal = [Px; 5];
pub const NONE: Px = 0;
const SET: u32 = 1 << 24;

#[inline]
pub fn px(r: u8, g: u8, b: u8) -> Px {
    SET | (r as u32) << 16 | (g as u32) << 8 | b as u32
}

#[inline]
pub fn rgb(p: Px) -> (u8, u8, u8) {
    ((p >> 16) as u8, (p >> 8) as u8, p as u8)
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t) as u8
}

pub fn mix(a: Px, b: Px, t: f64) -> Px {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    px(lerp(ar, br, t), lerp(ag, bg, t), lerp(ab, bb, t))
}

pub fn shade(c: Px, f: f64) -> Px {
    let (r, g, b) = rgb(c);
    let s = |v: u8| (v as f64 * f).clamp(0.0, 255.0) as u8;
    px(s(r), s(g), s(b))
}

/// five shades of one color, light -> deep
pub fn ramp(base: Px) -> Pal {
    let (w, k) = (px(255, 255, 255), px(0, 0, 0));
    [
        mix(base, w, 0.72),
        mix(base, w, 0.4),
        base,
        mix(base, k, 0.2),
        mix(base, k, 0.38),
    ]
}

/// theme names in the order `--list` shows them
pub const NAMES: [&str; 12] = [
    "sakura", "green", "pink", "red", "orange", "yellow", "gold", "cyan", "blue", "purple",
    "magenta", "white",
];
pub const RAINBOW: [&str; 8] = [
    "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta",
];

pub fn theme(name: &str) -> Option<Pal> {
    Some(match name {
        "sakura" => [
            px(255, 238, 242),
            px(255, 216, 226),
            px(255, 186, 201),
            px(247, 156, 183),
            px(226, 120, 155),
        ],
        "green" => [
            px(198, 240, 150),
            px(148, 214, 104),
            px(100, 184, 84),
            px(64, 148, 70),
            px(42, 112, 58),
        ],
        "pink" => ramp(px(255, 105, 170)),
        "red" => ramp(px(232, 48, 70)),
        "orange" => ramp(px(255, 146, 48)),
        "yellow" => ramp(px(255, 212, 66)),
        "gold" => ramp(px(236, 180, 40)),
        "cyan" => ramp(px(66, 212, 230)),
        "blue" => ramp(px(80, 140, 255)),
        "purple" => ramp(px(168, 98, 240)),
        "magenta" => ramp(px(232, 66, 190)),
        "white" => ramp(px(250, 250, 252)),
        _ => return None,
    })
}

fn round(x: f64) -> i32 {
    (x + 0.5).floor() as i32
}

/// nearest color in the 256-color palette (for terminals without 24-bit color)
pub fn ansi256(c: Px) -> u8 {
    let (r, g, b) = rgb(c);
    let (ri, gi, bi) = (r as i32, g as i32, b as i32);
    if (ri - gi).abs() < 10 && (gi - bi).abs() < 10 {
        let gray = (ri + gi + bi) / 3;
        if gray < 8 {
            16
        } else if gray > 248 {
            231
        } else {
            (232 + round((gray - 8) as f64 / 247.0 * 23.0)) as u8
        }
    } else {
        let q = |v: i32| round(v as f64 / 255.0 * 5.0);
        (16 + 36 * q(ri) + 6 * q(gi) + q(bi)) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_roundtrip() {
        assert_eq!(rgb(px(1, 2, 3)), (1, 2, 3));
        assert_ne!(px(0, 0, 0), NONE);
    }

    #[test]
    fn mix_ends() {
        let (a, b) = (px(10, 20, 30), px(200, 100, 50));
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(rgb(mix(a, b, 1.0)), (200, 100, 50));
    }

    #[test]
    fn ansi_known_values() {
        assert_eq!(ansi256(px(255, 255, 255)), 231);
        assert_eq!(ansi256(px(0, 0, 0)), 16);
        assert_eq!(ansi256(px(255, 0, 0)), 196);
        assert_eq!(ansi256(px(0, 0, 255)), 21);
    }

    #[test]
    fn all_themes_exist() {
        for n in NAMES.iter().chain(RAINBOW.iter()) {
            assert!(theme(n).is_some(), "{}", n);
        }
    }
}
