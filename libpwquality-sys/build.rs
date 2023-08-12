use cc::Build;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

fn default_dict_path() -> &'static str {
    [
        // ubuntu
        "/var/cache/cracklib/cracklib_dict",
        // gentoo
        "/usr/lib/cracklib_dict",
        // freebsd
        "/usr/local/libdata/cracklib/cracklib-words",
        // default
        "/usr/share/cracklib/pw_dict",
    ]
    .iter()
    .find(|s| Path::new(s).with_extension("pwd").exists())
    .expect("cracklib dictionaries not found, please install cracklib dictionaries or set DEFAULT_CRACKLIB_DICT environment")
}

fn build_cracklib<P: AsRef<Path>>(out_dir: P) -> bool {
    if cfg!(docsrs) {
        return false;
    }

    println!("cargo:rerun-if-env-changed=DEFAULT_CRACKLIB_DICT");

    let mut cfg = Build::new();
    let files = ["fascist.c", "packlib.c", "rules.c", "stringlib.c"];
    let mut dict_path =
        std::env::var("DEFAULT_CRACKLIB_DICT").unwrap_or(default_dict_path().into());

    dict_path.insert(0, '"');
    dict_path.push('"');

    cfg.files(files.map(|f| Path::new("cracklib/src/lib").join(f)))
        .include(&out_dir)
        .include("cracklib/src/lib")
        .define("HAVE_UNISTD_H", None)
        .define("IN_CRACKLIB", None)
        .define("DEFAULT_CRACKLIB_DICT", dict_path.as_str())
        .warnings(false)
        .out_dir(&out_dir)
        .compile("crack");

    true
}

fn build_pwquality<P: AsRef<Path>>(out_dir: P, enable_crack: bool) {
    let mut cfg = Build::new();
    let files = ["check.c", "error.c", "generate.c", "settings.c"];

    cfg.files(files.map(|f| Path::new("libpwquality/src").join(f)))
        .include(&out_dir)
        .include("libpwquality/src")
        .define("_(msgid)", "(msgid)")
        .define("_GNU_SOURCE", None)
        .warnings(false)
        .out_dir(&out_dir);

    if enable_crack {
        cfg.include("cracklib/src/lib").define("HAVE_CRACK_H", None);
    }

    cfg.compile("pwquality");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let vendored = std::env::var("CARGO_FEATURE_VENDORED").is_ok();

    // Try to find system libpwquality
    if !vendored
        && pkg_config::Config::new()
            .atleast_version("1.4.4")
            .probe("pwquality")
            .is_ok()
    {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    File::create(out_dir.join("config.h")).expect("Failed to create config.h");

    if ["libpwquality", "cracklib"]
        .iter()
        .any(|s| !Path::new(s).join("src").exists())
    {
        Command::new("git")
            .args(["submodule", "update", "--init", "--recursive"])
            .status()
            .expect("Failed to initialize libpwquality");
    }

    let enable_crack = build_cracklib(&out_dir);
    build_pwquality(&out_dir, enable_crack);
}
