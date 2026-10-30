// SPDX-License-Identifier: GPL-3.0-or-later
//! Runs the real `opencar` binary end to end, with a temporary home folder.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn opencar(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_opencar"))
        .args(args)
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("run opencar")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn built_in_database_validates_and_finds_the_reference_car() {
    let home = tempfile::tempdir().unwrap();
    let o = opencar(home.path(), &["compat", "validate"]);
    assert!(o.status.success(), "{}", stdout(&o));
    let o = opencar(home.path(), &["compat", "search", "Scion", "fr-s"]);
    assert!(stdout(&o).contains("scion-frs-pioneer-2013-2014"));
}

#[test]
fn validate_fails_on_a_bad_profile_folder() {
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join("profiles");
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("fake.toml"),
        "id='fake'\nmanufacturer='X'\nmodel='Y'\ninstallation='native-linux'\nandroid_auto='yes'\ncarplay='yes'\nstatus='verified'\n",
    )
    .unwrap();
    let o = opencar(home.path(), &["compat", "validate", "--dir", dir.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    let out = stdout(&o);
    assert!(out.contains("needs at least one [[evidence]]"));
    assert!(out.contains("carplay_auth"));
}

#[test]
fn setup_then_configure_livi_keeps_its_other_settings() {
    let home = tempfile::tempdir().unwrap();
    let livi = home.path().join(".config/LIVI/config.json");
    fs::create_dir_all(livi.parent().unwrap()).unwrap();
    fs::write(&livi, r#"{"carName":"FR-S","carPlayMfiI2cBus":2}"#).unwrap();

    let o = opencar(home.path(), &["setup", "--profile", "raspberry-pi-5", "--mfi-bus", "1"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(home.path().join(".config/open-car/config.toml").exists());

    let o = opencar(home.path(), &["livi", "configure", "--dry-run"]);
    assert!(stdout(&o).contains("would set carPlayMfiI2cBus: 2 -> 1"));
    assert!(fs::read_to_string(&livi).unwrap().contains("\"carPlayMfiI2cBus\":2"), "dry run wrote nothing");

    let o = opencar(home.path(), &["livi", "configure"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&livi).unwrap()).unwrap();
    assert_eq!(v["carPlayMfiI2cBus"], 1);
    assert_eq!(v["carName"], "FR-S");
}

#[test]
fn setup_refuses_an_unknown_profile() {
    let home = tempfile::tempdir().unwrap();
    let o = opencar(home.path(), &["setup", "--profile", "no-such-unit"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(!home.path().join(".config/open-car/config.toml").exists());
}

#[test]
fn check_runs_on_a_fake_root_and_diag_hides_the_home_folder() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join("root");
    fs::create_dir_all(root.join("sys/bus/usb/devices/usb1")).unwrap();
    let o = opencar(home.path(), &["check", "--root", root.to_str().unwrap()]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("[ ready ] Wired Android Auto"));

    let out = home.path().join("d.json");
    let o = opencar(home.path(), &["diag", "-o", out.to_str().unwrap()]);
    assert!(o.status.success());
    let text = fs::read_to_string(&out).unwrap();
    assert!(text.contains("\"opencar_version\""));
    assert!(!text.contains(home.path().to_str().unwrap()));
}
