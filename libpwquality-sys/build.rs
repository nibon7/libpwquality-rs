use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "vendored")]
mod vendor {
    use super::{Path, Result};
    use cc::Build;
    use pkg_config::probe_library;
    use std::{fs::File, process::Command};

    fn is_docsrs() -> bool {
        std::env::var("DOCS_RS").is_ok()
    }

    fn update_submodule(module: impl AsRef<Path>) -> Result<()> {
        if is_docsrs() {
            return Ok(());
        }

        if [".git", "src"]
            .iter()
            .all(|s| !module.as_ref().join(s).exists())
        {
            Command::new("git")
                .args(["submodule", "update", "--init", "--recursive"])
                .status()?;
        }

        Ok(())
    }

    fn default_dict_path() -> Result<String> {
        if is_docsrs() {
            return Ok("/path/to/cracklib_dict".into());
        }

        if let Ok(path) = std::env::var("DEFAULT_CRACKLIB_DICT") {
            return Ok(path);
        }

        let msg ="cracklib dictionaries not found, install cracklib-runtime or set DEFAULT_CRACKLIB_DICT environment";
        let path = [
            // default
            "/usr/share/cracklib/pw_dict",
            // ubuntu
            "/var/cache/cracklib/cracklib_dict",
            // gentoo
            "/usr/lib/cracklib_dict",
            // freebsd
            "/usr/local/libdata/cracklib/cracklib-words",
        ]
        .iter()
        .find_map(|s| {
            Path::new(s)
                .with_extension("pwd")
                .exists()
                .then_some(s.to_string())
        })
        .ok_or(msg)?;

        Ok(path)
    }

    fn link_zlib(build: &mut Build) {
        let Ok(zlib) = probe_library("zlib") else {
            println!(
                "cargo::warning=cracklib: zlib not found, disable compressed dictionaries support"
            );
            return;
        };

        build
            .define("HAVE_ZLIB_H", None)
            .includes(&zlib.include_paths);
    }

    fn build_cracklib(src_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> Result<()> {
        println!("cargo:rerun-if-env-changed=DEFAULT_CRACKLIB_DICT");
        println!("cargo:rerun-if-changed={}", src_dir.as_ref().display());

        update_submodule("cracklib")?;

        let dict_path = format!("\"{}\"", default_dict_path()?);
        let files =
            ["fascist.c", "packlib.c", "rules.c", "stringlib.c"].map(|f| src_dir.as_ref().join(f));

        let mut build = Build::new();

        build
            .files(files)
            .include(&out_dir)
            .include(src_dir)
            .define("HAVE_UNISTD_H", None)
            .define("IN_CRACKLIB", None)
            .define("DEFAULT_CRACKLIB_DICT", Some(dict_path.as_str()))
            .warnings(false)
            .out_dir(&out_dir);

        link_zlib(&mut build);

        build.try_compile("crack")?;

        Ok(())
    }

    fn build_libpwquality(src_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> Result<()> {
        println!("cargo:rerun-if-changed={}", src_dir.as_ref().display());

        update_submodule("libpwquality")?;

        let files =
            ["check.c", "error.c", "generate.c", "settings.c"].map(|f| src_dir.as_ref().join(f));

        let mut build = cc::Build::new();

        build
            .files(files)
            .include(&out_dir)
            .include(src_dir)
            .define("HAVE_CRACK_H", None)
            .define("_(msgid)", "(msgid)")
            .define("_GNU_SOURCE", None)
            .warnings(false)
            .out_dir(&out_dir);

        if cfg!(feature = "vendored-cracklib") {
            let cracklib_src_dir = Path::new("cracklib/src/lib");

            build.include(cracklib_src_dir);
            build_cracklib(cracklib_src_dir, &out_dir)?;
        } else {
            println!("cargo:rerun-if-env-changed=CRACKLIB_INCLUDE_PATH");
            println!("cargo:rerun-if-env-changed=CRACKLIB_LIBRARY_PATH");
            println!("cargo:rerun-if-env-changed=CRACKLIB_STATIC");

            if let Ok(include_path) = std::env::var("CRACKLIB_INCLUDE_PATH") {
                build.include(include_path);
            }

            if let Ok(library_path) = std::env::var("CRACKLIB_LIBRARY_PATH") {
                println!("cargo:rustc-link-search={library_path}");
            }

            let link_str = std::env::var("CRACKLIB_STATIC").map_or("", |_| "static=");
            println!("cargo:rustc-link-lib={link_str}crack");
        }

        build.try_compile("pwquality")?;

        Ok(())
    }

    pub(super) fn header_path(out_dir: impl AsRef<Path>) -> Result<String> {
        File::create(out_dir.as_ref().join("config.h"))?;

        let src_dir = Path::new("libpwquality/src");
        build_libpwquality(src_dir, &out_dir)?;

        Ok("libpwquality/src/pwquality.h".into())
    }
}

#[cfg(not(feature = "vendored"))]
mod system {
    use super::{Path, Result};
    use pkg_config::Config;

    fn features_to_version() -> &'static str {
        if cfg!(feature = "v1_4_5") {
            "1.4.5"
        } else if cfg!(feature = "v1_4_3") {
            "1.4.3"
        } else if cfg!(feature = "v1_4_1") {
            "1.4.1"
        } else if cfg!(feature = "v1_4") {
            "1.4.0"
        } else if cfg!(feature = "v1_3") {
            "1.3.0"
        } else if cfg!(feature = "v1_2") {
            "1.2.0"
        } else {
            "1.0.0"
        }
    }

    pub(super) fn header_path(_out_dir: impl AsRef<Path>) -> Result<String> {
        let header = Config::new()
            .atleast_version(features_to_version())
            .probe("pwquality")?
            .include_paths
            .iter()
            .find_map(|p| {
                let header = p.join("pwquality.h");
                header.exists().then_some(header)
            })
            .ok_or("Could not find pwquality.h")?
            .to_string_lossy()
            .to_string();

        Ok(header)
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    #[cfg(feature = "vendored")]
    let header_path = vendor::header_path(out_dir)?;

    #[cfg(not(feature = "vendored"))]
    let header_path = system::header_path(out_dir)?;

    let path = Path::new(&out_dir).join("bindings.rs");
    bindgen::builder()
        .allowlist_var("PWQ_.*")
        .allowlist_type("pwquality_.*")
        .allowlist_function("pwquality_.*")
        .header(header_path)
        .generate()?
        .write_to_file(path)?;

    Ok(())
}
