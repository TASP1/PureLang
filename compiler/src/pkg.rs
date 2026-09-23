//! Minimal PureLang package manager (path dependencies)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

pub fn run(args: &[String]) {
    if args.is_empty() {
        print_pkg_usage();
        process::exit(1);
    }
    match args[0].as_str() {
        "init" => cmd_init(args.get(1).map(|s| s.as_str())),
        "add" => {
            if args.len() < 2 {
                eprintln!("Usage: purec pkg add path:<dir>");
                process::exit(1);
            }
            cmd_add(&args[1]);
        }
        "build" => cmd_build(),
        "list" => cmd_list(),
        "help" | "--help" | "-h" => print_pkg_usage(),
        other => {
            eprintln!("Unknown pkg command: {}", other);
            print_pkg_usage();
            process::exit(1);
        }
    }
}

fn print_pkg_usage() {
    eprintln!("PureLang package manager");
    eprintln!();
    eprintln!("  purec pkg init [name]     Create Pure.toml in current directory");
    eprintln!("  purec pkg add path:DIR    Add a path dependency");
    eprintln!("  purec pkg list            Show package + dependencies");
    eprintln!("  purec pkg build           Type-check/compile src/main.pure (or main.pure)");
}

fn cmd_init(name: Option<&str>) {
    let name = name.unwrap_or("myapp");
    let path = Path::new("Pure.toml");
    if path.exists() {
        eprintln!("Pure.toml already exists");
        process::exit(1);
    }
    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
entry = "src/main.pure"

[dependencies]
# path dependencies, e.g.:
# utils = {{ path = "../utils" }}
"#
    );
    fs::write(path, content).expect("write Pure.toml");
    let _ = fs::create_dir_all("src");
    let main = Path::new("src/main.pure");
    if !main.exists() {
        fs::write(
            main,
            "fn main() {\n    print \"Hello from PureLang package\"\n}\n",
        )
        .expect("write main");
    }
    println!("Created Pure.toml and src/main.pure");
}

fn cmd_add(spec: &str) {
    let path = if let Some(p) = spec.strip_prefix("path:") {
        p
    } else {
        eprintln!("Only path dependencies supported: purec pkg add path:../lib");
        process::exit(1);
    };
    let dep_name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("dep");
    let mut toml = fs::read_to_string("Pure.toml").unwrap_or_else(|_| {
        eprintln!("No Pure.toml — run purec pkg init first");
        process::exit(1);
    });
    if !toml.contains("[dependencies]") {
        toml.push_str("\n[dependencies]\n");
    }
    let line = format!("{dep_name} = {{ path = \"{path}\" }}\n");
    if toml.contains(&format!("{dep_name} =")) {
        eprintln!("Dependency '{dep_name}' already listed");
        return;
    }
    toml.push_str(&line);
    fs::write("Pure.toml", toml).expect("update Pure.toml");
    println!("Added dependency {dep_name} (path: {path})");
}

fn cmd_list() {
    let toml = fs::read_to_string("Pure.toml").unwrap_or_else(|_| {
        eprintln!("No Pure.toml");
        process::exit(1);
    });
    println!("{}", toml);
}

fn parse_entry(toml: &str) -> String {
    for line in toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("entry") {
            if let Some(eq) = rest.find('=') {
                let v = rest[eq + 1..].trim().trim_matches('"');
                return v.to_string();
            }
        }
    }
    "src/main.pure".into()
}

fn cmd_build() {
    let toml = fs::read_to_string("Pure.toml").unwrap_or_else(|_| {
        eprintln!("No Pure.toml — run purec pkg init");
        process::exit(1);
    });
    let entry = parse_entry(&toml);
    if !Path::new(&entry).exists() {
        eprintln!("Entry not found: {}", entry);
        process::exit(1);
    }
    // Collect path deps (informational + future include path)
    let mut dep_paths: Vec<PathBuf> = Vec::new();
    let mut in_deps = false;
    for line in toml.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t.starts_with('[') {
            in_deps = t == "[dependencies]";
            continue;
        }
        if in_deps && t.contains("path") && t.contains('=') {
            if let Some(start) = t.find('"') {
                if let Some(end) = t[start + 1..].find('"') {
                    dep_paths.push(PathBuf::from(&t[start + 1..start + 1 + end]));
                }
            }
        }
    }
    let purec = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("purec"));
    for d in &dep_paths {
        println!("dependency path: {}", d.display());
        let dep_toml = d.join("Pure.toml");
        if dep_toml.exists() {
            if let Ok(txt) = fs::read_to_string(&dep_toml) {
                let dep_entry = parse_entry(&txt);
                let dep_file = d.join(&dep_entry);
                if dep_file.exists() {
                    println!("  type-check {}", dep_file.display());
                    let st = Command::new(&purec)
                        .arg(dep_file.to_string_lossy().as_ref())
                        .status()
                        .expect("check dep");
                    if !st.success() {
                        eprintln!("Dependency failed: {}", d.display());
                        process::exit(1);
                    }
                }
            }
        }
    }
    let status = Command::new(&purec)
        .args(["--compile", "-o", "app", &entry])
        .status()
        .expect("run purec");
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
    println!("Package build OK → ./app");
}
