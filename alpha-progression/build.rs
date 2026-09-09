use std::{env, fs, path::PathBuf};

/// Loads `.env` from the crate dir or any ancestor and forwards its entries to
/// `env!()`. Real environment variables win, so CI can supply the keys without a file.
fn main() {
    let Some(path) = find_dotenv() else { return };
    println!("cargo:rerun-if-changed={}", path.display());

    let Ok(contents) = fs::read_to_string(&path) else { return };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        let value = value.trim().trim_matches(['"', '\'']);

        println!("cargo:rerun-if-env-changed={key}");
        if env::var_os(key).is_none() {
            println!("cargo:rustc-env={key}={value}");
        }
    }
}

fn find_dotenv() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").ok()?);
    loop {
        let candidate = dir.join(".env");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
