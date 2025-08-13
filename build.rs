//! Build script for the `xsd_parser` crate.

use std::env::var;
use std::fs::{read, read_to_string, write};
use std::path::PathBuf;

use base64::{prelude::BASE64_STANDARD, Engine};
use regex::{Captures, Regex};

fn main() {
    let cargo_dir =
        var("CARGO_MANIFEST_DIR").expect("Missing `CARGO_MANIFEST_DIR` environment variable!");
    let cargo_dir = PathBuf::from(cargo_dir);

    let readme = cargo_dir.join("README.md");
    let logo_png = cargo_dir.join("doc/logo.png");

    println!("cargo:rerun-if-changed={}", readme.display());
    println!("cargo:rerun-if-changed={}", logo_png.display());

    let rx = Regex::new(r"\[!([A-Z]+)\]").unwrap();

    let logo_png = read(logo_png).expect("Unable to load `doc/logo.png`");
    let logo_png = BASE64_STANDARD.encode(logo_png);
    let logo_png = format!("data:image/png;base64,{logo_png}");

    let readme = read_to_string(readme).expect("Unable to read `README.md`");
    let readme = readme.replace("doc/logo.png", &logo_png);
    let readme = rx.replace_all(&readme, |c: &Captures<'_>| {
        let keyword = &c[1];

        format!("**{keyword}**")
    });

    let out_dir = var("OUT_DIR").expect("Missing `OUT_DIR` environment variable!");
    let out_dir = PathBuf::from(out_dir);

    write(out_dir.join("README.md"), &*readme).expect("Unable to write `README.md`");
}
