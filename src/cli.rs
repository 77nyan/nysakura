//! Command line parsing. Every option works exactly like in the Python version of the earlier Python releases.

use crate::color;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn help() -> String {
    format!(
        "\
nysakura {v} - a growing cherry blossom tree for your terminal

usage: nysakura [color] [options]

colors (pick one, e.g. nysakura -red):
  -sakura (default)  -pink  -red  -orange  -yellow  -gold  -green
  -cyan  -blue  -purple  -magenta  -white  -rainbow
  -hex RRGGBB          any custom color

options:
  -s, --seed N         reproducible tree (same seed = same tree)
  -p, --print          print the finished tree and exit (screenshots, MOTD)
  -i, --instant        skip the growing animation
      -mini            a shorter, wider tree (the default tree is tall and slim)
  -t, --time SEC       quit after SEC seconds
      --cycle SEC      grow a new tree SEC seconds after the last one finished
      --speed X        animation speed multiplier (default 1.0)
      --grow SEC       time the tree needs to grow (default 9)
      --petals N       number of falling petals (default: scales with window)
      --wind X         wind strength, negative blows left (default 1.0)
      --scale X        tree size multiplier (default 1.0)
      --width N        width in cells (default: terminal width)
      --height N       height in cells (default: terminal height)
      --fps N          frames per second (default 30)
      --truecolor      force 24-bit color
      --256            force 256-color mode
  -l, --list           list the color names
  -h, --help           show this help
  -v, --version        show the version

keys: q quit | r grow a new tree | space pause
",
        v = VERSION
    )
}

#[derive(Clone, Debug)]
pub struct Opts {
    pub color: String,
    pub hex: Option<String>,
    pub seed: Option<u64>,
    pub speed: f64,
    pub grow: f64,
    pub petals: Option<usize>,
    pub wind: f64,
    pub scale: f64,
    pub fps: u32,
    pub cycle: f64,
    pub time: f64,
    pub width: usize,
    pub height: usize,
    pub print: bool,
    pub instant: bool,
    pub tall: bool,
    pub truecolor: bool,
    pub c256: bool,
    pub list: bool,
    pub help: bool,
    pub version: bool,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            color: "sakura".into(),
            hex: None,
            seed: None,
            speed: 1.0,
            grow: 9.0,
            petals: None,
            wind: 1.0,
            scale: 1.0,
            fps: 30,
            cycle: 0.0,
            time: 0.0,
            width: 0,
            height: 0,
            print: false,
            instant: false,
            tall: true,
            truecolor: false,
            c256: false,
            list: false,
            help: false,
            version: false,
        }
    }
}

fn short(name: &str) -> &str {
    match name {
        "s" => "seed",
        "t" => "time",
        "c" => "color",
        "p" => "print",
        "i" => "instant",
        "l" => "list",
        "h" => "help",
        "v" => "version",
        other => other,
    }
}

fn is_value_option(name: &str) -> bool {
    matches!(
        name,
        "seed"
            | "speed"
            | "grow"
            | "petals"
            | "wind"
            | "scale"
            | "fps"
            | "cycle"
            | "time"
            | "width"
            | "height"
            | "hex"
            | "color"
    )
}

pub fn parse(argv: &[String]) -> Result<Opts, String> {
    let mut o = Opts::default();
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        i += 1;
        if !a.starts_with('-') || a == "-" {
            return Err(format!("unexpected argument: {}", a));
        }
        let body = a.trim_start_matches('-');
        let (raw_name, inline) = match body.find('=') {
            Some(p) => (&body[..p], Some(&body[p + 1..])),
            None => (body, None),
        };
        let lower = raw_name.to_lowercase();
        let name = short(&lower);
        if color::theme(name).is_some() || name == "rainbow" {
            o.color = name.to_string();
        } else if name == "mini" {
            o.tall = false;
        } else if name == "tall" {
            o.tall = true; // the default, kept so old commands still work
        } else if matches!(
            name,
            "print" | "instant" | "truecolor" | "256" | "list" | "help" | "version"
        ) {
            match name {
                "print" => o.print = true,
                "instant" => o.instant = true,
                "truecolor" => o.truecolor = true,
                "256" => o.c256 = true,
                "list" => o.list = true,
                "help" => o.help = true,
                _ => o.version = true,
            }
        } else if is_value_option(name) {
            let val: String = match inline {
                Some(v) => v.to_string(),
                None => {
                    if i >= argv.len() {
                        return Err(format!("option {} needs a value", a));
                    }
                    i += 1;
                    argv[i - 1].clone()
                }
            };
            let bad = || format!("bad value for {}: {}", a, val);
            match name {
                "seed" => o.seed = Some(val.trim().parse::<i64>().map_err(|_| bad())? as u64),
                "speed" => o.speed = val.trim().parse().map_err(|_| bad())?,
                "grow" => o.grow = val.trim().parse().map_err(|_| bad())?,
                "petals" => {
                    o.petals = Some(val.trim().parse::<i64>().map_err(|_| bad())?.max(0) as usize)
                }
                "wind" => o.wind = val.trim().parse().map_err(|_| bad())?,
                "scale" => o.scale = val.trim().parse().map_err(|_| bad())?,
                "fps" => o.fps = val.trim().parse::<i64>().map_err(|_| bad())?.max(0) as u32,
                "cycle" => o.cycle = val.trim().parse().map_err(|_| bad())?,
                "time" => o.time = val.trim().parse().map_err(|_| bad())?,
                "width" => o.width = val.trim().parse::<i64>().map_err(|_| bad())?.max(0) as usize,
                "height" => {
                    o.height = val.trim().parse::<i64>().map_err(|_| bad())?.max(0) as usize
                }
                "hex" => o.hex = Some(val.trim_start_matches('#').to_string()),
                _ => o.color = val.clone(),
            }
        } else {
            return Err(format!("unknown option: {}", a));
        }
    }
    if color::theme(&o.color).is_none() && o.color != "rainbow" {
        return Err(format!("unknown color: {} (see --list)", o.color));
    }
    if let Some(h) = &o.hex {
        if !(h.len() == 3 || h.len() == 6) || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("--hex wants a color like ff66aa".to_string());
        }
    }
    o.fps = o.fps.clamp(1, 120);
    o.grow = o.grow.max(0.5);
    o.speed = o.speed.max(0.05);
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<Opts, String> {
        parse(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn defaults_are_tall_and_sakura() {
        let o = p(&[]).unwrap();
        assert!(o.tall);
        assert_eq!(o.color, "sakura");
    }

    #[test]
    fn colors_and_mini() {
        let o = p(&["-red", "-mini"]).unwrap();
        assert_eq!(o.color, "red");
        assert!(!o.tall);
        assert_eq!(p(&["--BLUE"]).unwrap().color, "blue");
    }

    #[test]
    fn values() {
        let o = p(&["-s", "12", "--wind=-2", "--hex", "#ff8800", "-t", "1.5"]).unwrap();
        assert_eq!(o.seed, Some(12));
        assert_eq!(o.wind, -2.0);
        assert_eq!(o.hex.as_deref(), Some("ff8800"));
        assert_eq!(o.time, 1.5);
    }

    #[test]
    fn errors() {
        assert!(p(&["-nope"]).is_err());
        assert!(p(&["--hex", "zzz"]).is_err());
        assert!(p(&["--seed"]).is_err());
        assert!(p(&["--seed", "abc"]).is_err());
        assert!(p(&["-c", "mauve"]).is_err());
        assert!(p(&["red"]).is_err());
    }

    #[test]
    fn flags() {
        let o = p(&["-p", "-i", "--256", "-l", "-h", "-v"]).unwrap();
        assert!(o.print && o.instant && o.c256 && o.list && o.help && o.version);
    }
}
