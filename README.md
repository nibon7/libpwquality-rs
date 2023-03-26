# [libpwquality](https://github.com/libpwquality/libpwquality) bindings for Rust

[![Crates.io](https://img.shields.io/crates/v/libpwquality)](https://crates.io/crates/libpwquality)
[![Crates.io](https://img.shields.io/crates/d/libpwquality)](https://crates.io/crates/libpwquality)
[![License](https://img.shields.io/github/license/nibon7/libpwquality-rs)](LICENSE)
[![docs.rs](https://img.shields.io/docsrs/libpwquality)](https://docs.rs/libpwquality)
[![Build Status](https://img.shields.io/github/actions/workflow/status/nibon7/libpwquality-rs/ci.yml)](https://github.com/nibon7/libpwquality-rs/actions/workflows/ci.yml?query=branch%3Amain)

## Examples

```rust
use libpwquality::{Error, PWQuality, Setting};

fn main() -> Result<(), Error> {
    let pwq = PWQuality::new()?;

    pwq.read_default_config()?;

    pwq.set_int_value(Setting::MinLength, 9)?;
    let minlen = pwq.get_int_value(Setting::MinLength)?;
    println!("minlen={}", minlen);

    pwq.set_str_value(Setting::BadWords, "badpassword")?;
    let badwords = pwq.get_str_value(Setting::BadWords)?;
    println!("badwords=\"{}\"", badwords);

    pwq.set_option("maxrepeat=2")?;
    let maxrepeat = pwq.get_int_value(Setting::MaxRepeat)?;
    println!("maxrepeat={}", maxrepeat);

    let password = pwq.generate(32)?;
    println!("password={}", password);

    let score = pwq.check("p@s5w0rD!", None, None)?;
    println!("score={}", score);

    let score = pwq.check("p@s5w0rD!", Some("password!"), None)?;
    println!("score1={}", score);

    let score = pwq.check("p@s5w0rD!", Some("password!"), Some("root"))?;
    println!("score2={}", score);

    Ok(())
}
```
