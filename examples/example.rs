use libpwquality::{Error, PWQuality, Setting};

fn main() -> Result<(), Error> {
    let pwq = PWQuality::new()?;

    pwq.read_default_config()?;

    pwq.set_int_value(Setting::MinLength, 9);
    let minlen = pwq.get_int_value(Setting::MinLength);
    println!("minlen={}", minlen);

    pwq.set_str_value(Setting::BadWords, "badpassword")?;
    let badwords = pwq.get_str_value(Setting::BadWords)?;
    println!("badwords=\"{}\"", badwords);

    pwq.set_option("maxrepeat=2")?;
    let maxrepeat = pwq.get_int_value(Setting::MaxRepeat);
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
