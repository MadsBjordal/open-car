// SPDX-License-Identifier: GPL-3.0-or-later
//! Builds the compatibility database (`compat/profiles/*.toml`) into the binary, so `opencar`
//! works on a head unit without the repository next to it.

use std::{env, fs, path::Path};

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../compat/profiles");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".toml"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let mut code = String::from(
        "/// The profiles of `compat/profiles`, built in: `(file name, TOML text)`.\npub const PROFILES: &[(&str, &str)] = &[\n",
    );
    for n in &names {
        let path = dir.join(n).canonicalize().expect("profile path");
        println!("cargo:rerun-if-changed={}", path.display());
        code.push_str(&format!("    ({n:?}, include_str!({:?})),\n", path.display().to_string()));
    }
    code.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("profiles.rs"), code).unwrap();
}
