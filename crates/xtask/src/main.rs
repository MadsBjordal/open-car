// SPDX-License-Identifier: GPL-3.0-or-later
//! Project chores. `cargo xtask registry` writes `docs/MASTER_FUNCTIONS.md`, the registry of every
//! module and public item, from the source code; `cargo xtask registry --check` fails when the file
//! is out of date or a public item has no `///` comment (ADR-0006).

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CRATES: &[&str] = &["opencar-core", "opencar"];
const OUT: &str = "docs/MASTER_FUNCTIONS.md";

/// One registered item.
struct Item {
    id: String,
    kind: &'static str,
    signature: String,
    purpose: String,
}

/// One source file.
struct Module {
    path: String,
    file: String,
    purpose: String,
    tests: usize,
    items: Vec<Item>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("registry") {
        eprintln!("usage: cargo xtask registry [--check]");
        return ExitCode::from(2);
    }
    let check = args.iter().any(|a| a == "--check");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (modules, undocumented) = scan(&root);
    let text = render(&modules);
    let out = root.join(OUT);
    if !undocumented.is_empty() {
        eprintln!("Public items without a /// comment (its first sentence is what the registry shows):");
        for u in &undocumented {
            eprintln!("  {u}");
        }
        return ExitCode::FAILURE;
    }
    if check {
        if fs::read_to_string(&out).ok().as_deref() != Some(text.as_str()) {
            eprintln!("{OUT} is out of date: run `cargo xtask registry` and commit it.");
            return ExitCode::FAILURE;
        }
        println!("{OUT} is up to date.");
    } else {
        fs::write(&out, text).expect("write registry");
        println!("Wrote {OUT}.");
    }
    ExitCode::SUCCESS
}

fn scan(root: &Path) -> (Vec<Module>, Vec<String>) {
    let mut modules = Vec::new();
    let mut undocumented = Vec::new();
    for krate in CRATES {
        let src = root.join("crates").join(krate).join("src");
        let mut files = Vec::new();
        collect_rs(&src, &mut files);
        files.sort();
        for f in files {
            let rel = f.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
            let path = module_path(krate, &rel);
            let text = fs::read_to_string(&f).expect("read source");
            let file = format!("crates/{krate}/src/{rel}");
            let (m, missing) = parse(&path, &file, &text);
            undocumented.extend(missing);
            modules.push(m);
        }
    }
    (modules, undocumented)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn module_path(krate: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = vec![];
    let krate = krate.replace('-', "_");
    let stem = rel.trim_end_matches(".rs");
    for p in stem.split('/') {
        if !matches!(p, "lib" | "main" | "mod") {
            parts.push(p);
        }
    }
    if parts.is_empty() { krate } else { format!("{krate}::{}", parts.join("::")) }
}

fn first_sentence(doc: &[String]) -> String {
    let mut para = String::new();
    for l in doc {
        if l.trim().is_empty() {
            break;
        }
        if !para.is_empty() {
            para.push(' ');
        }
        para.push_str(l.trim());
    }
    let end = para
        .match_indices(". ")
        .map(|(i, _)| i)
        .find(|&i| !para[..i].ends_with("e.g") && !para[..i].ends_with("i.e"))
        .map(|i| i + 1)
        .unwrap_or(para.len());
    para[..end].replace('|', "\\|")
}

fn parse(path: &str, file: &str, text: &str) -> (Module, Vec<String>) {
    let lines: Vec<&str> = text.lines().collect();
    let module_doc: Vec<String> =
        lines.iter().filter_map(|l| l.strip_prefix("//!")).map(|l| l.trim().to_string()).collect();
    let tests = lines.iter().filter(|l| l.trim() == "#[test]").count();
    let mut items = Vec::new();
    let mut missing = Vec::new();
    let mut doc: Vec<String> = Vec::new();
    let mut owner: Option<String> = None; // the impl or trait we are inside
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        let t = raw.trim();
        if t == "#[cfg(test)]" {
            break; // tests sit at the end of each file
        }
        if let Some(d) = t.strip_prefix("///") {
            doc.push(d.trim().to_string());
            i += 1;
            continue;
        }
        if t.starts_with("#[") {
            i += 1;
            continue;
        }
        if !raw.starts_with(' ') && !t.is_empty() {
            owner = None;
            if let Some(rest) = t.strip_prefix("impl") {
                let rest = rest.trim_start_matches(|c| c != ' ').trim();
                let ty = rest.rsplit(" for ").next().unwrap_or(rest);
                owner = Some(ty.split([' ', '<', '{']).next().unwrap_or("").to_string());
            }
        }
        let item = if let Some(rest) = t.strip_prefix("pub trait ") {
            let name = ident(rest);
            if !raw.starts_with(' ') {
                owner = Some(name.clone());
            }
            Some(("trait", name))
        } else if let Some(rest) = t.strip_prefix("pub fn ").or_else(|| t.strip_prefix("pub const fn ")) {
            Some(("fn", ident(rest)))
        } else if let (Some(rest), true) =
            (t.strip_prefix("fn "), raw.starts_with("    ") && owner.is_some() && is_trait_body(&lines, i))
        {
            Some(("fn", ident(rest)))
        } else if let Some(rest) = t.strip_prefix("pub struct ") {
            Some(("struct", ident(rest)))
        } else if let Some(rest) = t.strip_prefix("pub enum ") {
            Some(("enum", ident(rest)))
        } else if let Some(rest) = t.strip_prefix("pub const ") {
            Some(("const", ident(rest)))
        } else {
            t.strip_prefix("pub type ").map(|rest| ("type", ident(rest)))
        };
        if let Some((kind, name)) = item {
            let nested = raw.starts_with(' ') && kind == "fn";
            let id = match (&owner, nested) {
                (Some(o), true) => format!("{path}::{o}::{name}"),
                _ => format!("{path}::{name}"),
            };
            let signature = if kind == "fn" { signature(&lines, i) } else { String::new() };
            if doc.is_empty() {
                missing.push(format!("{id} ({file}:{})", i + 1));
            }
            items.push(Item { id, kind, signature, purpose: first_sentence(&doc) });
        }
        doc.clear();
        i += 1;
    }
    (
        Module { path: path.to_string(), file: file.to_string(), purpose: first_sentence(&module_doc), tests, items },
        missing,
    )
}

/// True when the line at `i` is inside a `trait` block (the nearest unindented line opens one).
fn is_trait_body(lines: &[&str], i: usize) -> bool {
    lines[..i].iter().rev().find(|l| !l.starts_with(' ') && !l.trim().is_empty()).is_some_and(|l| l.contains("trait "))
}

fn ident(rest: &str) -> String {
    rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
}

fn signature(lines: &[&str], start: usize) -> String {
    let mut s = String::new();
    for l in &lines[start..] {
        s.push(' ');
        s.push_str(l.trim());
        if l.contains('{') || l.trim_end().ends_with(';') {
            break;
        }
    }
    let s = s.split(" {").next().unwrap_or(&s).trim_end_matches(';');
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(", )", ")");
    format!("`{}`", s.replace('|', "\\|"))
}

fn render(modules: &[Module]) -> String {
    let mut s = String::new();
    s += "# Function and module registry\n\n";
    s += "**Generated by `cargo xtask registry` from the source code. Do not edit by hand**: CI fails when it is out of date.\n";
    s += "Before writing anything new, search this file (REUSE → EXTEND → REFACTOR → CREATE NEW, see CLAUDE.md).\n";
    s += "An item's purpose is the first sentence of its `///` comment; its status is Implemented unless that sentence says otherwise.\n";
    s += "Tests are in each file's `mod tests`.\n\n";
    s += "## Modules\n\n| Module | File | Purpose | Tests |\n|---|---|---|---|\n";
    for m in modules {
        let _ = writeln!(s, "| `{}` | `{}` | {} | {} |", m.path, m.file, m.purpose, m.tests);
    }
    for m in modules.iter().filter(|m| !m.items.is_empty()) {
        let _ = writeln!(s, "\n## `{}`\n\n| ID | Kind | Purpose | Signature |\n|---|---|---|---|", m.path);
        for it in &m.items {
            let _ = writeln!(s, "| `{}` | {} | {} | {} |", it.id, it.kind, it.purpose, it.signature);
        }
    }
    s
}
