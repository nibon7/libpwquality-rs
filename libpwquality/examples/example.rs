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
