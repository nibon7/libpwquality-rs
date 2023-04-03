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

    pwq.set_max_repeat(2);
    let maxrepeat = pwq.get_max_repeat();
    println!("maxrepeat={}", maxrepeat);

    let password = pwq.generate(32)?;
    println!("password={:?}", password);

    let score = pwq.check(&password, None, None)?;
    println!("score={}", score);

    let score = pwq.check(&password, Some("password!"), None)?;
    println!("score1={}", score);

    let score = pwq.check(&password, Some("password!"), Some("root"))?;
    println!("score2={}", score);

    Ok(())
}
