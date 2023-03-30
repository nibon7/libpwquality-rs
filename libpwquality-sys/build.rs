use cc::Build;
use std::fs::File;
use std::{path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libpwquality");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    if !PathBuf::from("libpwquality/src").exists() {
        Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status()
            .expect("Failed to initialize libpwquality");
    }

    File::create(out_dir.join("config.h")).expect("Failed to create config.h");

    let mut cfg = Build::new();
    let files = ["check.c", "error.c", "generate.c", "settings.c"];

    cfg.files(files.map(|f| PathBuf::from("libpwquality/src").join(f)))
        .include(&out_dir)
        .include("libpwquality/src")
        .define("_(msgid)", "(msgid)")
        .define("_GNU_SOURCE", None)
        .warnings(false)
        .out_dir(&out_dir);

    if cfg!(feature = "crack") {
        cfg.define("HAVE_CRACK_H", None);
        println!("cargo:rustc-link-lib=crack");
    }

    cfg.compile("pwquality");

    println!("cargo:rustc-link-lib=pwquality");
}
