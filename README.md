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

If you need [cracklib](https://github.com/cracklib/cracklib) support, enable `crack` feature and install cracklib development library.

```sh
cargo add libpwquality --features crack
sudo pacman -S cracklib
```

## Examples

```rust
use libpwquality::{Error, PWQuality};

fn main() -> Result<(), Error> {
    let pwq = PWQuality::new()?;

    pwq.read_default_config()?;

    pwq.set_min_length(9);
    let minlen = pwq.get_min_length();
    println!("minlen={}", minlen);

    pwq.set_bad_words(["bad", "password"])?;
    let badwords = pwq.get_bad_words()?;
    println!("badwords={:?}", badwords);

    pwq.set_option("maxrepeat=2")?;
    let maxrepeat = pwq.get_max_repeat();
    println!("maxrepeat={}", maxrepeat);

    let password = pwq.generate(32)?;
    println!("password={}", password);

    let score = pwq.check(&password, None, None)?;
    println!("score={}", score);

    let score = pwq.check(&password, Some("password!"), None)?;
    println!("score1={}", score);

    let score = pwq.check(&password, Some("password!"), Some("root"))?;
    println!("score2={}", score);

    Ok(())
}
```
