fn main() -> Result<(), Box<dyn std::error::Error>> {
    let header = system_deps::Config::new()
        .probe()?
        .all_include_paths()
        .iter()
        .find_map(|p| {
            let header = p.join("pwquality.h");
            header.exists().then_some(header)
        })
        .ok_or("Could not find pwquality.h")?;

    let out_dir = std::env::var("OUT_DIR")?;
    let path = std::path::PathBuf::from(out_dir).join("bindings.rs");

    bindgen::builder()
        .allowlist_var("PWQ_.*")
        .allowlist_type("pwquality_.*")
        .allowlist_function("pwquality_.*")
        .header(header.to_string_lossy())
        .generate()?
        .write_to_file(path)?;

    Ok(())
}
