use libpwquality::{PWQError, PWQuality};
use serial_test::serial;

#[test]
#[serial]
fn test_read_config() -> Result<(), PWQError> {
    let pwq = PWQuality::new()?;
    let ret = pwq.read_config("/invalid/path/pwquality.conf");

    assert!(ret.is_err());

    Ok(())
}

#[test]
#[serial]
fn test_generate() -> Result<(), PWQError> {
    let pwq = PWQuality::new()?;
    let password = pwq.generate(32)?;

    assert!(!password.is_empty());

    Ok(())
}

#[test]
#[serial]
fn test_check() -> Result<(), PWQError> {
    let pwq = PWQuality::new()?;
    let score = pwq.check("p@s5w0rD!", None, None)?;

    assert!(score >= 0);

    Ok(())
}

#[test]
#[serial]
fn test_helper() -> Result<(), PWQError> {
    let pwq = PWQuality::new()?;

    let value = 1;
    pwq.min_diff(value);
    assert_eq!(pwq.get_min_diff(), value);

    let value = 12;
    pwq.min_length(value);
    assert_eq!(pwq.get_min_length(), value);

    let value = 2;
    pwq.digit_credit(value);
    assert_eq!(pwq.get_digit_credit(), value);

    let value = 3;
    pwq.uppercase_credit(value);
    assert_eq!(pwq.get_uppercase_credit(), value);

    let value = 6;
    pwq.lowercase_credit(value);
    assert_eq!(pwq.get_lowercase_credit(), value);

    let value = 5;
    pwq.other_credit(value);
    assert_eq!(pwq.get_other_credit(), value);

    let value = 4;
    pwq.min_class(value);
    assert_eq!(pwq.get_min_class(), value);

    let value = 7;
    pwq.max_repeat(value);
    assert_eq!(pwq.get_max_repeat(), value);

    let value = 8;
    pwq.max_sequence(value);
    assert_eq!(pwq.get_max_sequence(), value);

    let value = 9;
    pwq.max_class_repeat(value);
    assert_eq!(pwq.get_max_class_repeat(), value);

    pwq.gecos_check(true);
    assert!(pwq.get_gecos_check());

    pwq.dict_check(true);
    assert!(pwq.get_dict_check());

    pwq.user_check(true);
    assert!(pwq.get_user_check());

    let value = 10;
    pwq.user_substr(value);
    assert_eq!(pwq.get_user_substr(), value);

    pwq.enforcing(true);
    assert!(pwq.get_enforcing());

    pwq.bad_words(["bad", "words"])?;
    let value = pwq.get_bad_words()?;
    assert_eq!(value, vec!["bad".to_string(), "words".to_string()]);

    #[cfg(feature = "crack")]
    {
        let path = "/path/to/dict";
        pwq.dict_path(path)?;

        let s = pwq.get_dict_path()?;

        assert!(s.eq(path));
    }

    let value = 11;
    pwq.retry_times(value);
    assert_eq!(pwq.get_retry_times(), value);

    pwq.enforce_for_root(true);
    assert!(pwq.get_enforce_for_root());

    pwq.local_users_only(true);
    assert!(pwq.get_local_users_only());

    Ok(())
}
