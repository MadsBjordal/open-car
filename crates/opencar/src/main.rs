// SPDX-License-Identifier: GPL-3.0-or-later
//! `opencar`: the open-car command-line tool.
//!
//! Phone projection itself runs in LIVI (ADR-0002); this tool checks a computer, searches the
//! compatibility database, keeps open-car's settings and writes a profile's settings into LIVI.

use clap::{Parser, Subcommand};
use opencar_core::compat::CompatDb;
use opencar_core::config::{self, Config};
use opencar_core::diag::Diagnostics;
use opencar_core::probe::{self, HardwareReport, PhoneKind, SystemCommands};
use opencar_core::profile::DeviceProfile;
use opencar_core::readiness::{self, Check, Level};
use opencar_core::{fsutil, livi};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod built_in {
    include!(concat!(env!("OUT_DIR"), "/profiles.rs"));
}

#[derive(Parser)]
#[command(
    name = "opencar",
    version,
    about = "open-car: Android Auto and CarPlay for existing head units, built on LIVI"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check this computer: what phone projection needs and what is missing.
    Check {
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
        /// Probe another root folder instead of `/` (for testing).
        #[arg(long, default_value = "/", hide = true)]
        root: PathBuf,
    },
    /// The hardware compatibility database.
    Compat {
        #[command(subcommand)]
        command: CompatCmd,
        /// Read profiles from this folder instead of the built-in copy.
        #[arg(long, global = true)]
        dir: Option<PathBuf>,
    },
    /// open-car's settings on this computer.
    Setup {
        /// The hardware profile id that describes this computer.
        #[arg(long)]
        profile: Option<String>,
        /// A LIVI Link box is connected (true/false).
        #[arg(long)]
        livi_link: Option<bool>,
        /// I2C bus an MFi coprocessor is wired to; 'none' to clear.
        #[arg(long)]
        mfi_bus: Option<String>,
    },
    /// LIVI, the projection engine.
    Livi {
        #[command(subcommand)]
        command: LiviCmd,
    },
    /// Write a diagnostics file (stays on this computer; nothing is uploaded).
    Diag {
        /// Where to write it.
        #[arg(long, short, default_value = "opencar-diagnostics.json")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum CompatCmd {
    /// List every profile.
    List,
    /// Find profiles by maker, model or vehicle, e.g. `opencar compat search fr-s`.
    Search {
        /// Words that must all match.
        words: Vec<String>,
    },
    /// Show one profile in full.
    Show {
        /// Profile id.
        id: String,
    },
    /// Check every profile against the database rules (used by CI).
    Validate,
}

#[derive(Subcommand)]
enum LiviCmd {
    /// Is LIVI installed, and does it start by itself?
    Status,
    /// Write the chosen profile's LIVI settings into LIVI's config.json (stop LIVI first).
    Configure {
        /// Only show what would change.
        #[arg(long)]
        dry_run: bool,
    },
}

struct Env {
    home: Option<String>,
    xdg_config_home: Option<String>,
}

impl Env {
    fn read() -> Self {
        Env { home: std::env::var("HOME").ok(), xdg_config_home: std::env::var("XDG_CONFIG_HOME").ok() }
    }
    fn config_path(&self) -> Result<PathBuf, String> {
        config::config_path(self.xdg_config_home.as_deref(), self.home.as_deref())
            .ok_or_else(|| "HOME is not set".into())
    }
    fn config(&self) -> Result<Config, String> {
        Config::load(&self.config_path()?).map_err(|e| e.to_string())
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command, &Env::read()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(cmd: Cmd, env: &Env) -> Result<ExitCode, String> {
    match cmd {
        Cmd::Check { json, root } => {
            let cfg = env.config()?;
            let d = diagnostics(&root, env, cfg.profile.clone(), &cfg);
            if json {
                println!("{}", d.to_redacted_json(env.home.as_deref()));
            } else {
                print!("{}", check_text(&d.hardware, &d.readiness, &d.livi, &cfg));
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Compat { command, dir } => {
            let db = match &dir {
                Some(d) => CompatDb::load_dir(d).map_err(|e| format!("{}: {e}", d.display()))?,
                None => built_in_db(),
            };
            compat(command, &db)
        }
        Cmd::Setup { profile, livi_link, mfi_bus } => setup(env, profile, livi_link, mfi_bus),
        Cmd::Livi { command: LiviCmd::Status } => {
            print!(
                "{}",
                livi_text(&livi::find_install(Path::new("/"), env.home.as_deref(), env.xdg_config_home.as_deref()))
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Livi { command: LiviCmd::Configure { dry_run } } => configure_livi(env, dry_run),
        Cmd::Diag { output } => {
            let cfg = env.config()?;
            let d = diagnostics(Path::new("/"), env, cfg.profile.clone(), &cfg);
            fsutil::write_atomic(&output, d.to_redacted_json(env.home.as_deref()).as_bytes(), false)
                .map_err(|e| format!("{}: {e}", output.display()))?;
            println!(
                "Wrote {}. It stays on this computer; attach it to a report yourself if you want to.",
                output.display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn diagnostics(root: &Path, env: &Env, profile: Option<String>, cfg: &Config) -> Diagnostics {
    let hardware = probe::probe(root, &SystemCommands);
    let readiness = readiness::assess(&hardware, options(cfg));
    let livi = livi::find_install(root, env.home.as_deref(), env.xdg_config_home.as_deref());
    Diagnostics { opencar_version: opencar_core::VERSION.into(), profile, hardware, readiness, livi }
}

fn setup(
    env: &Env,
    profile: Option<String>,
    livi_link: Option<bool>,
    mfi_bus: Option<String>,
) -> Result<ExitCode, String> {
    let path = env.config_path()?;
    let mut cfg = Config::load(&path).map_err(|e| e.to_string())?;
    let changed = profile.is_some() || livi_link.is_some() || mfi_bus.is_some();
    if let Some(id) = profile {
        if built_in_db().get(&id).is_none() {
            return Err(format!("no profile '{id}'; see `opencar compat list`"));
        }
        cfg.profile = Some(id);
    }
    if let Some(v) = livi_link {
        cfg.livi_link = v;
    }
    if let Some(b) = mfi_bus {
        cfg.mfi_i2c_bus = match b.as_str() {
            "none" => None,
            n => Some(n.parse().map_err(|_| format!("--mfi-bus must be a number or 'none', not '{n}'"))?),
        };
    }
    if changed {
        cfg.save(&path).map_err(|e| e.to_string())?;
        println!("Saved {}", path.display());
    }
    println!("profile:   {}", cfg.profile.as_deref().unwrap_or("(none)"));
    println!("LIVI Link: {}", if cfg.livi_link { "yes" } else { "no" });
    println!("MFi bus:   {}", cfg.mfi_i2c_bus.map(|b| b.to_string()).unwrap_or_else(|| "(none)".into()));
    Ok(ExitCode::SUCCESS)
}

fn configure_livi(env: &Env, dry_run: bool) -> Result<ExitCode, String> {
    let cfg = env.config()?;
    let path = livi::config_path(env.xdg_config_home.as_deref(), env.home.as_deref()).ok_or("HOME is not set")?;
    let settings = livi_settings(&cfg, &built_in_db())?;
    let changes = if dry_run { livi::plan_settings(&path, &settings) } else { livi::apply_settings(&path, &settings) }
        .map_err(|e| e.to_string())?;
    if changes.is_empty() {
        println!("LIVI's settings already match ({}).", path.display());
    }
    for c in &changes {
        println!("{} {}: {} -> {}", if dry_run { "would set" } else { "set" }, c.key, c.before, c.after);
    }
    if !dry_run && !changes.is_empty() {
        println!(
            "Saved {} (previous copy: {}). Restart LIVI to use it.",
            path.display(),
            fsutil::backup_path(&path).display()
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn built_in_db() -> CompatDb {
    CompatDb::from_sources(built_in::PROFILES.iter().copied())
}

fn options(cfg: &Config) -> readiness::Options {
    readiness::Options { livi_link: cfg.livi_link, mfi_i2c_bus: cfg.mfi_i2c_bus }
}

/// The LIVI settings for this computer: the profile's, with the MFi bus from open-car's settings on top.
fn livi_settings(cfg: &Config, db: &CompatDb) -> Result<opencar_core::profile::LiviSettings, String> {
    let mut s = match &cfg.profile {
        Some(id) => {
            db.get(id).ok_or_else(|| format!("profile '{id}' is not in the database"))?.livi.clone().unwrap_or_default()
        }
        None => Default::default(),
    };
    if cfg.mfi_i2c_bus.is_some() {
        s.mfi_i2c_bus = cfg.mfi_i2c_bus;
    }
    if s == Default::default() {
        return Err(
            "nothing to configure: choose a profile with LIVI settings (`opencar setup --profile …`) or set --mfi-bus"
                .into(),
        );
    }
    Ok(s)
}

fn compat(cmd: CompatCmd, db: &CompatDb) -> Result<ExitCode, String> {
    match cmd {
        CompatCmd::List => {
            print!("{}", profile_lines(&db.search("")));
            Ok(ExitCode::SUCCESS)
        }
        CompatCmd::Search { words } => {
            let found = db.search(&words.join(" "));
            if found.is_empty() {
                println!(
                    "Nothing found. Not in the list yet? See docs/COMPATIBILITY_REPORTS.md to add your head unit."
                );
            }
            print!("{}", profile_lines(&found));
            Ok(ExitCode::SUCCESS)
        }
        CompatCmd::Show { id } => {
            let p = db.get(&id).ok_or_else(|| format!("no profile '{id}'"))?;
            println!("{}", serde_json::to_string_pretty(p).expect("profile serialises"));
            Ok(ExitCode::SUCCESS)
        }
        CompatCmd::Validate => {
            let problems = db.validate();
            for (file, p) in &problems {
                println!("{file}: {p}");
            }
            if problems.is_empty() {
                println!("{} profile(s), all valid.", db.entries.len());
                Ok(ExitCode::SUCCESS)
            } else {
                println!("{} problem(s).", problems.len());
                Ok(ExitCode::FAILURE)
            }
        }
    }
}

fn profile_lines(profiles: &[&DeviceProfile]) -> String {
    profiles
        .iter()
        .map(|p| {
            let vehicles = if p.vehicles.is_empty() { String::new() } else { format!(" ({})", p.vehicles.join(", ")) };
            format!("{:<34} {:<27} {} {}{}\n", p.id, p.status.label(), p.manufacturer, p.model, vehicles)
        })
        .collect()
}

fn mark(l: Level) -> &'static str {
    match l {
        Level::Ready => "[ ready ]",
        Level::Limited => "[limited]",
        Level::Unknown => "[   ?   ]",
        Level::Missing => "[missing]",
    }
}

fn check_text(r: &HardwareReport, checks: &[Check], install: &livi::Install, cfg: &Config) -> String {
    let mut s = format!("open-car {}: hardware check\n\n", opencar_core::VERSION);
    let board = r.board_model.clone().or_else(|| r.cpu_model.clone()).unwrap_or_else(|| "unknown computer".into());
    let ram = r.ram_mb.map(|m| format!(", {m} MB")).unwrap_or_default();
    let os = r.os.as_ref().map(|o| format!(", {o}")).unwrap_or_default();
    s += &format!("Computer: {board} ({}, {} cores{ram}{os})\n", r.cpu_arch, r.cpu_cores);
    s += &format!("Profile:  {}\n", cfg.profile.as_deref().unwrap_or("(none chosen: `opencar setup --profile <id>`)"));
    let phones: Vec<String> = r
        .phones()
        .iter()
        .map(|d| match d.phone {
            Some(PhoneKind::Apple) => format!("Apple device ({})", d.product.as_deref().unwrap_or("iPhone?")),
            _ => "Android phone in accessory mode".to_string(),
        })
        .collect();
    if !phones.is_empty() {
        s += &format!("Phones on USB: {}\n", phones.join(", "));
    }
    s += "\n";
    for c in checks {
        s += &format!("{} {}\n", mark(c.level), c.feature.label());
        for why in &c.reasons {
            s += &format!("            {why}\n");
        }
    }
    s += "\n";
    s += &livi_text(install);
    s += "\n\"ready\" means the requirements are present. Only a test with a real phone shows it works.\n";
    s
}

fn livi_text(i: &livi::Install) -> String {
    let mut s = String::new();
    match &i.appimage {
        Some(p) => s += &format!("LIVI: installed ({})\n", p.display()),
        None => s += "LIVI: not found in ~/LIVI. Install it with LIVI's installer: see docs/INSTALL_LINUX.md\n",
    }
    let boot = if i.kiosk_service {
        "yes (livi-kiosk.service)"
    } else if i.desktop_autostart {
        "yes (desktop autostart)"
    } else {
        "no"
    };
    s += &format!("Starts by itself: {boot}\n");
    s += &format!(
        "LIVI settings: {}\n",
        if i.config.is_some() { "present" } else { "not created yet (LIVI has not run)" }
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_database_is_valid() {
        let db = built_in_db();
        assert!(!db.entries.is_empty());
        let problems: Vec<String> = db.validate().iter().map(|(f, p)| format!("{f}: {p}")).collect();
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn livi_settings_need_something_to_write() {
        let db = built_in_db();
        assert!(livi_settings(&Config::default(), &db).is_err());
        let cfg = Config { mfi_i2c_bus: Some(1), ..Default::default() };
        assert_eq!(livi_settings(&cfg, &db).unwrap().mfi_i2c_bus, Some(1));
    }

    #[test]
    fn check_text_never_says_working() {
        let r = HardwareReport::default();
        let checks = readiness::assess(&r, Default::default());
        let text = check_text(&r, &checks, &livi::Install::default(), &Config::default());
        assert!(text.contains("[missing] Wired CarPlay"));
        assert!(text.contains("Only a test with a real phone shows it works"));
    }
}
