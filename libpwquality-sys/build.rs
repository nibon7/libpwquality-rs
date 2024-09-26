use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "vendored")]
mod vendor {
    use super::{Path, Result};
    use cc::Build;
    use std::{fs::File, process::Command};

    fn update_submodule(module: &str) -> Result<()> {
        if std::env::var("DOCS_RS").is_ok() {
            return Ok(());
        }

        if !Path::new(module).join(".git").exists() {
            Command::new("git")
                .args(["submodule", "update", "--init", "--recursive"])
                .status()?;
        }

        Ok(())
    }

    fn default_dict_path() -> Result<String> {
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

    fn build_cracklib(out_dir: impl AsRef<Path>) -> Result<()> {
        println!("cargo:rerun-if-env-changed=DEFAULT_CRACKLIB_DICT");

        let dict_path = format!("\"{}\"", default_dict_path()?);
        let src_dir = Path::new("cracklib/src/lib");
        let files = ["fascist.c", "packlib.c", "rules.c", "stringlib.c"].map(|f| src_dir.join(f));

        Build::new()
            .files(files)
            .include(&out_dir)
            .include(src_dir)
            .define("HAVE_UNISTD_H", None)
            .define("IN_CRACKLIB", None)
            .define("DEFAULT_CRACKLIB_DICT", Some(dict_path.as_str()))
            .warnings(false)
            .out_dir(&out_dir)
            .try_compile("crack")?;

        Ok(())
    }

    fn build_libpwquality(out_dir: impl AsRef<Path>) -> Result<()> {
        let src_dir = Path::new("libpwquality/src");
        let files = ["check.c", "error.c", "generate.c", "settings.c"].map(|f| src_dir.join(f));

        cc::Build::new()
            .files(files)
            .include(&out_dir)
            .include(src_dir)
            .include("cracklib/src/lib")
            .define("HAVE_CRACK_H", None)
            .define("_(msgid)", "(msgid)")
            .define("_GNU_SOURCE", None)
            .warnings(false)
            .out_dir(out_dir)
            .try_compile("pwquality")?;

        Ok(())
    }

    pub(super) fn header_path(out_dir: impl AsRef<Path>) -> Result<String> {
        for module in ["cracklib", "libpwquality"] {
            update_submodule(module)?;
        }

        File::create(out_dir.as_ref().join("config.h"))?;

        build_cracklib(out_dir.as_ref())?;
        build_libpwquality(out_dir.as_ref())?;

        Ok("libpwquality/src/pwquality.h".into())
    }
}

#[cfg(not(feature = "vendored"))]
mod system {
    use super::{Path, Result};
    use system_deps::Config;

    pub(super) fn header_path(_out_dir: impl AsRef<Path>) -> Result<String> {
        let header = Config::new()
            .probe()?
            .all_include_paths()
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
