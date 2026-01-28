use std::path::PathBuf;

#[path = "demo_showcase/state.rs"]
mod state;
#[path = "demo_showcase/theme.rs"]
mod theme;
#[path = "demo_showcase/timing.rs"]
mod timing;
#[path = "demo_showcase/typography.rs"]
mod typography;

/// Standalone rich_rust showcase binary (roadmap).
///
/// This file intentionally avoids heavy CLI dependencies (e.g. clap) and uses a
/// small hand-rolled parser per `bd-1o8x`.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cfg = match parse_args(args) {
        Ok(cfg) => cfg,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    if cfg.help {
        print!("{HELP_TEXT}");
        return;
    }

    if cfg.list_scenes {
        print_scenes();
        return;
    }

    if let Some(scene) = cfg.scene.as_deref() {
        // Scene execution is implemented in the scene runner beads.
        println!("(demo_showcase) TODO: run scene `{scene}`");
        return;
    }

    let timing = timing::Timing::new(cfg.speed, cfg.quick);
    let mut rng = timing::DemoRng::new(cfg.seed);
    let run_id = 1000 + rng.gen_range(0..9000);

    // Touch these helpers early so they stay exercised in non-test builds.
    let _scaled = timing.scale(std::time::Duration::from_millis(150));
    timing.sleep(std::time::Duration::from_millis(1));

    let state = if timing.quick() {
        state::SharedDemoState::new(run_id, cfg.seed)
    } else {
        state::SharedDemoState::demo_seeded(run_id, cfg.seed)
    };

    state.update(|demo| {
        demo.headline = "Ready to deploy".to_string();
    });

    let snapshot = state.snapshot();
    let last_log = snapshot
        .logs
        .last()
        .map(|line| {
            format!(
                "{}+{}ms {}",
                line.level.as_str(),
                line.t.as_millis(),
                line.message
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let eta_count = snapshot
        .pipeline
        .iter()
        .filter(|stage| stage.eta.is_some())
        .count();
    println!(
        "(demo_showcase) TODO: run full demo (run_id={} seed={} elapsed={}ms headline={:?} services={} stages={} (eta={}) logs={} last_log={:?})\n\nTip: run with `--help` or `--list-scenes`.",
        snapshot.run_id,
        snapshot.seed,
        snapshot.elapsed.as_millis(),
        snapshot.headline,
        snapshot.services.len(),
        snapshot.pipeline.len(),
        eta_count,
        snapshot.logs.len(),
        last_log,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ColorMode {
    #[default]
    Auto,
    None,
    Standard,
    EightBit,
    TrueColor,
}

impl ColorMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "none" | "no" | "off" => Ok(Self::None),
            "standard" | "16" => Ok(Self::Standard),
            "eight_bit" | "eightbit" | "256" => Ok(Self::EightBit),
            "truecolor" | "true" | "24bit" => Ok(Self::TrueColor),
            _ => Err(format!(
                "Invalid --color-system value `{value}` (expected: auto|none|standard|eight_bit|truecolor)."
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum ExportMode {
    #[default]
    Off,
    TempDir,
    Dir(PathBuf),
}

#[derive(Debug, Clone, Default)]
struct Config {
    help: bool,
    list_scenes: bool,
    scene: Option<String>,
    seed: u64,

    quick: bool,
    speed: f64,

    interactive: Option<bool>,
    live: Option<bool>,
    screen: Option<bool>,

    force_terminal: bool,
    width: Option<usize>,
    height: Option<usize>,
    color_system: ColorMode,
    emoji: Option<bool>,
    safe_box: Option<bool>,
    links: Option<bool>,

    export: ExportMode,
}

impl Config {
    fn with_defaults() -> Self {
        Self {
            speed: 1.0,
            ..Self::default()
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut iter = args.into_iter();
    // Drop binary name if present.
    let _ = iter.next();

    let mut cfg = Config::with_defaults();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => cfg.help = true,
            "--list-scenes" => cfg.list_scenes = true,
            "--scene" => {
                if cfg.scene.is_some() {
                    return Err("`--scene` provided more than once.".to_string());
                }
                let scene = next_value(&mut iter, "--scene")?;
                if !is_known_scene(&scene) {
                    return Err(format!(
                        "Unknown scene `{scene}`.\n\n{}",
                        available_scenes_help()
                    ));
                }
                cfg.scene = Some(scene);
            }
            "--seed" => {
                let raw = next_value(&mut iter, "--seed")?;
                cfg.seed = parse_u64_flag("--seed", &raw)?;
            }
            "--quick" => cfg.quick = true,
            "--speed" => {
                let raw = next_value(&mut iter, "--speed")?;
                cfg.speed = raw.parse::<f64>().map_err(|_| {
                    format!("Invalid --speed value `{raw}` (expected a number like 0.5, 1.0, 2.0).")
                })?;
                if !cfg.speed.is_finite() || cfg.speed <= 0.0 {
                    return Err(format!(
                        "Invalid --speed value `{raw}` (expected a finite number > 0)."
                    ));
                }
            }

            "--interactive" => cfg.interactive = Some(true),
            "--no-interactive" => cfg.interactive = Some(false),
            "--live" => cfg.live = Some(true),
            "--no-live" => cfg.live = Some(false),
            "--screen" => cfg.screen = Some(true),
            "--no-screen" => cfg.screen = Some(false),

            "--force-terminal" => cfg.force_terminal = true,
            "--width" => {
                let raw = next_value(&mut iter, "--width")?;
                cfg.width = Some(parse_usize_flag("--width", &raw)?);
            }
            "--height" => {
                let raw = next_value(&mut iter, "--height")?;
                cfg.height = Some(parse_usize_flag("--height", &raw)?);
            }
            "--color-system" => {
                let raw = next_value(&mut iter, "--color-system")?;
                cfg.color_system = ColorMode::parse(&raw)?;
            }
            "--emoji" => cfg.emoji = Some(true),
            "--no-emoji" => cfg.emoji = Some(false),
            "--safe-box" => cfg.safe_box = Some(true),
            "--no-safe-box" => cfg.safe_box = Some(false),
            "--links" => cfg.links = Some(true),
            "--no-links" => cfg.links = Some(false),

            "--export" => {
                if !matches!(cfg.export, ExportMode::Off) {
                    return Err("`--export`/`--export-dir` provided more than once.".to_string());
                }
                cfg.export = ExportMode::TempDir;
            }
            "--export-dir" => {
                if !matches!(cfg.export, ExportMode::Off) {
                    return Err("`--export`/`--export-dir` provided more than once.".to_string());
                }
                let raw = next_value(&mut iter, "--export-dir")?;
                cfg.export = ExportMode::Dir(PathBuf::from(raw));
            }

            "--" => {
                return Err(
                    "Unexpected positional arguments (this CLI has no positional args)."
                        .to_string(),
                );
            }

            _ => {
                return Err(format!(
                    "Unknown flag: {arg}\n\nRun with `--help` to see valid options."
                ));
            }
        }
    }

    Ok(cfg)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("Missing value for `{flag}`."))
}

fn parse_usize_flag(flag: &str, raw: &str) -> Result<usize, String> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| format!("Invalid {flag} value `{raw}` (expected a positive integer)."))?;
    if value == 0 {
        return Err(format!("Invalid {flag} value `{raw}` (expected >= 1)."));
    }
    Ok(value)
}

fn parse_u64_flag(flag: &str, raw: &str) -> Result<u64, String> {
    raw.parse::<u64>()
        .map_err(|_| format!("Invalid {flag} value `{raw}` (expected a non-negative integer)."))
}

#[derive(Debug, Clone, Copy)]
struct SceneSpec {
    name: &'static str,
    summary: &'static str,
}

const SCENES: &[SceneSpec] = &[
    SceneSpec {
        name: "hero",
        summary: "Introduce Nebula Deploy and the visual “brand”.",
    },
    SceneSpec {
        name: "dashboard",
        summary: "Live split-screen dashboard (services + pipeline + logs).",
    },
    SceneSpec {
        name: "deep_dive_markdown",
        summary: "Runbook / release notes (feature: markdown).",
    },
    SceneSpec {
        name: "deep_dive_syntax",
        summary: "Config/code snippet view (feature: syntax).",
    },
    SceneSpec {
        name: "deep_dive_json",
        summary: "API payload view (feature: json).",
    },
    SceneSpec {
        name: "debug_tools",
        summary: "Pretty/Inspect + Traceback + RichLogger (+ tracing).",
    },
    SceneSpec {
        name: "export",
        summary: "Export HTML/SVG bundle.",
    },
    SceneSpec {
        name: "outro",
        summary: "Summary + next steps.",
    },
];

fn is_known_scene(name: &str) -> bool {
    SCENES.iter().any(|scene| scene.name == name)
}

fn available_scenes_help() -> String {
    let mut out = String::from("Available scenes:\n");
    let width = SCENES
        .iter()
        .map(|scene| scene.name.len())
        .max()
        .unwrap_or(0);

    for scene in SCENES {
        out.push_str(&format!(
            "  {:width$} - {}\n",
            scene.name,
            scene.summary,
            width = width
        ));
    }

    out.push_str("\nRun with `--list-scenes` to print this list and exit.");
    out
}

fn print_scenes() {
    // Ensure theme definitions stay parseable even before the full scene runner is wired.
    let _theme = theme::demo_theme();

    print!("{}", available_scenes_help());
    println!();
}

const HELP_TEXT: &str = r#"demo_showcase — Nebula Deploy (rich_rust showcase)

USAGE:
    demo_showcase [OPTIONS]

OPTIONS:
    --list-scenes               List available scenes and exit
    --scene <name>              Run a single scene (see --list-scenes)
    --seed <u64>                Seed deterministic demo data (default: 0)
    --quick                     Reduce sleeps/runtime (CI-friendly)
    --speed <multiplier>        Animation speed multiplier (default: 1.0)

    --interactive               Force interactive mode
    --no-interactive            Disable prompts/pager/etc
    --live                      Force live refresh
    --no-live                   Disable live refresh; print snapshots
    --screen                    Use alternate screen (requires live)
    --no-screen                 Disable alternate screen

    --force-terminal            Treat stdout as a TTY (even when piped)
    --width <cols>              Override console width
    --height <rows>             Override console height
    --color-system <mode>       auto|none|standard|eight_bit|truecolor
    --emoji                     Enable emoji (default)
    --no-emoji                  Disable emoji
    --safe-box                  Use ASCII-safe box characters
    --no-safe-box               Use Unicode box characters (default)
    --links                     Enable OSC8 hyperlinks
    --no-links                  Disable OSC8 hyperlinks

    --export                    Write an HTML/SVG bundle to a temp dir
    --export-dir <path>         Write an HTML/SVG bundle to a directory

    -h, --help                  Print help and exit

EXAMPLES:
    demo_showcase               Run the full demo (TTY-friendly defaults)
    demo_showcase --list-scenes List scenes
    demo_showcase --scene hero  Run a single scene
    demo_showcase --quick       Fast mode for CI/dev
    demo_showcase | cat         Non-interactive output (no live/prompt)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> Result<Config, String> {
        parse_args(argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn help_flag_sets_help() {
        let cfg = parse(&["demo_showcase", "--help"]).expect("parse");
        assert!(cfg.help);
    }

    #[test]
    fn list_scenes_parses() {
        let cfg = parse(&["demo_showcase", "--list-scenes"]).expect("parse");
        assert!(cfg.list_scenes);
    }

    #[test]
    fn scene_parses_once() {
        let cfg = parse(&["demo_showcase", "--scene", "hero"]).expect("parse");
        assert_eq!(cfg.scene.as_deref(), Some("hero"));
    }

    #[test]
    fn scene_rejects_unknown() {
        let err = parse(&["demo_showcase", "--scene", "wat"]).expect_err("error");
        assert!(err.contains("Unknown scene"));
        assert!(err.contains("Available scenes"));
    }

    #[test]
    fn scene_rejects_duplicates() {
        let err =
            parse(&["demo_showcase", "--scene", "hero", "--scene", "outro"]).expect_err("error");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn boolean_no_forms_parse() {
        let cfg = parse(&[
            "demo_showcase",
            "--no-interactive",
            "--live",
            "--no-screen",
            "--no-emoji",
            "--safe-box",
            "--no-links",
        ])
        .expect("parse");

        assert_eq!(cfg.interactive, Some(false));
        assert_eq!(cfg.live, Some(true));
        assert_eq!(cfg.screen, Some(false));
        assert_eq!(cfg.emoji, Some(false));
        assert_eq!(cfg.safe_box, Some(true));
        assert_eq!(cfg.links, Some(false));
    }

    #[test]
    fn speed_parses_and_requires_positive_finite() {
        let cfg = parse(&["demo_showcase", "--speed", "1.5"]).expect("parse");
        assert_eq!(cfg.speed, 1.5);

        let err = parse(&["demo_showcase", "--speed", "0"]).expect_err("error");
        assert!(err.contains("> 0"));
    }

    #[test]
    fn seed_parses_as_u64() {
        let cfg = parse(&["demo_showcase", "--seed", "42"]).expect("parse");
        assert_eq!(cfg.seed, 42);

        let err = parse(&["demo_showcase", "--seed", "wat"]).expect_err("error");
        assert!(err.contains("Invalid --seed"));

        let err = parse(&["demo_showcase", "--seed", "-1"]).expect_err("error");
        assert!(err.contains("Invalid --seed"));
    }

    #[test]
    fn links_toggle_parses() {
        let cfg = parse(&["demo_showcase", "--links"]).expect("parse");
        assert_eq!(cfg.links, Some(true));

        let cfg = parse(&["demo_showcase", "--no-links"]).expect("parse");
        assert_eq!(cfg.links, Some(false));
    }

    #[test]
    fn width_height_require_positive_ints() {
        let cfg = parse(&["demo_showcase", "--width", "80", "--height", "24"]).expect("parse");
        assert_eq!(cfg.width, Some(80));
        assert_eq!(cfg.height, Some(24));

        let err = parse(&["demo_showcase", "--width", "0"]).expect_err("error");
        assert!(err.contains(">= 1"));
    }

    #[test]
    fn color_system_parses_known_values() {
        let cfg = parse(&["demo_showcase", "--color-system", "eight_bit"]).expect("parse");
        assert_eq!(cfg.color_system, ColorMode::EightBit);

        let err = parse(&["demo_showcase", "--color-system", "wat"]).expect_err("error");
        assert!(err.contains("Invalid --color-system"));
    }

    #[test]
    fn export_flags_are_mutually_exclusive() {
        let cfg = parse(&["demo_showcase", "--export"]).expect("parse");
        assert!(matches!(cfg.export, ExportMode::TempDir));

        let cfg = parse(&["demo_showcase", "--export-dir", "out"]).expect("parse");
        assert!(matches!(cfg.export, ExportMode::Dir(_)));

        let err = parse(&["demo_showcase", "--export", "--export-dir", "out"]).expect_err("error");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn unknown_flags_error_is_friendly() {
        let err = parse(&["demo_showcase", "--wat"]).expect_err("error");
        assert!(err.contains("Unknown flag"));
        assert!(err.contains("--help"));
    }
}
