# [libpwquality](https://github.com/libpwquality/libpwquality) bindings for Rust

[![Crates.io](https://img.shields.io/crates/v/libpwquality)](https://crates.io/crates/libpwquality)
[![Crates.io](https://img.shields.io/crates/d/libpwquality)](https://crates.io/crates/libpwquality)
[![License](https://img.shields.io/github/license/nibon7/libpwquality-rs)](LICENSE)
[![docs.rs](https://img.shields.io/docsrs/libpwquality)](https://docs.rs/libpwquality)
[![Build Status](https://img.shields.io/github/actions/workflow/status/nibon7/libpwquality-rs/ci.yml)](https://github.com/nibon7/libpwquality-rs/actions/workflows/ci.yml?query=branch%3Amain)

## Usage

```sh
cargo add libpwquality
```

libpwquality-rs links system libpwquality library by default, you can
optionally enable `vendored` feature and install cracklib dictionaries
to build libpwquality. When the `vendored` feature is enabled, you can
export `DEFAULT_CRACKLIB_DICT` environment variable to specify the path
of the dictionaries if you encounter problems with the dictionary path.

```sh
cargo add libpwquality --features vendored
sudo apt-get install cracklib-runtime
```

## Example

```rust
use libpwquality::{PWQError, PWQuality};

fn main() -> Result<(), PWQError> {
    let pwq = PWQuality::new()?;

    pwq.read_default_config()?
        .min_length(9)
        .max_repeat(2)
        .bad_words(["bad", "password"])?;

    let minlen = pwq.get_min_length();
    println!("minlen={}", minlen);

    let badwords = pwq.get_bad_words()?;
    println!("badwords={:?}", badwords);

    let maxrepeat = pwq.get_max_repeat();
    println!("maxrepeat={}", maxrepeat);

    let password = pwq.generate(32)?;
    println!("password={:?}", password);

    let score = pwq.check(&password, Some("password!"), None)?;
    println!("score={}", score);

    Ok(())
}
```
