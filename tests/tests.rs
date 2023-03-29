#[cfg(test)]
mod tests {
    use libpwquality::{Error, PWQuality, Setting};
    use serial_test::serial;

    const SETTINGS_INT: &[(Setting, i32)] = &[
        (Setting::DiffOk, 1),
        (Setting::MinLength, 6),
        (Setting::DigCredit, 3),
        (Setting::UpCredit, 4),
        (Setting::LowCredit, 5),
        (Setting::OthCredit, 6),
        (Setting::MinClass, 4),
        (Setting::MaxRepeat, 8),
        (Setting::MaxClassRepeat, 9),
        (Setting::MaxSequence, 10),
        (Setting::GecosCheck, 11),
        (Setting::DictCheck, 12),
        (Setting::UserCheck, 13),
        (Setting::UserSubstr, 14),
        (Setting::Enforcing, 15),
        (Setting::RetryTimes, 16),
        (Setting::EnforceRoot, 1),
        (Setting::LocalUsers, 1),
    ];

    const SETTINGS_STR: &[(Setting, &str)] = &[
        (Setting::BadWords, "badpassword"),
        #[cfg(feature = "crack")]
        (Setting::DictPath, "/path/to/dict"),
    ];

    #[test]
    #[serial]
    fn test_read_config() -> Result<(), Error> {
        let pwq = PWQuality::new()?;
        let ret = pwq.read_config("/invalid/path/pwquality.conf");

        assert!(ret.is_err());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_option() -> Result<(), Error> {
        let pwq = PWQuality::new()?;

        for setting in SETTINGS_INT {
            let key = setting.0;
            let val = setting.1;
            let option = format!("{}={}", key.as_str(), val);

            pwq.set_option(&option)?;

            let value = pwq.get_int_value(key);

            assert_eq!(value, val);
        }

        for setting in SETTINGS_STR {
            let key = setting.0;
            let val = setting.1;
            let option = format!("{}={}", key.as_str(), val);

            pwq.set_option(&option)?;

            let value = pwq.get_str_value(key)?;

            assert!(value.eq(val));
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_int_val() -> Result<(), Error> {
        let pwq = PWQuality::new()?;

        for setting in SETTINGS_INT {
            let key = setting.0;
            let val = setting.1;

            pwq.set_int_value(key, val);

            let value = pwq.get_int_value(key);

            assert_eq!(value, val);
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_str_val() -> Result<(), Error> {
        let pwq = PWQuality::new()?;

        for setting in SETTINGS_STR {
            let key = setting.0;
            let val = setting.1;

            pwq.set_str_value(key, val)?;

            let value = pwq.get_str_value(key)?;

            assert!(value.eq(val));
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_generate() -> Result<(), Error> {
        let pwq = PWQuality::new()?;
        let password = pwq.generate(32)?;

        assert!(!password.is_empty());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_check() -> Result<(), Error> {
        let pwq = PWQuality::new()?;
        let score = pwq.check("p@s5w0rD!", None, None)?;

        assert!(score >= 0);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_helper() -> Result<(), Error> {
        let pwq = PWQuality::new()?;

        let value = 1;
        pwq.set_difok(value);
        assert_eq!(pwq.get_difok(), value);

        let value = 12;
        pwq.set_min_length(value);
        assert_eq!(pwq.get_min_length(), value);

        let value = 2;
        pwq.set_digit_credit(value);
        assert_eq!(pwq.get_digit_credit(), value);

        let value = 3;
        pwq.set_uppercase_credit(value);
        assert_eq!(pwq.get_uppercase_credit(), value);

        let value = 6;
        pwq.set_lowercase_credit(value);
        assert_eq!(pwq.get_lowercase_credit(), value);

        let value = 5;
        pwq.set_other_credit(value);
        assert_eq!(pwq.get_other_credit(), value);

        let value = 4;
        pwq.set_min_class(value);
        assert_eq!(pwq.get_min_class(), value);

        let value = 7;
        pwq.set_max_repeat(value);
        assert_eq!(pwq.get_max_repeat(), value);

        let value = 8;
        pwq.set_max_seqeunce(value);
        assert_eq!(pwq.get_max_seqeunce(), value);

        let value = 9;
        pwq.set_max_class_repeat(value);
        assert_eq!(pwq.get_max_class_repeat(), value);

        pwq.set_gecos_check(true);
        assert!(pwq.get_gecos_check());

        pwq.set_dict_check(true);
        assert!(pwq.get_dict_check());

        pwq.set_user_check(true);
        assert!(pwq.get_user_check());

        let value = 10;
        pwq.set_user_substr(value);
        assert_eq!(pwq.get_user_substr(), value);

        pwq.set_enforcing(true);
        assert!(pwq.get_enforcing());

        pwq.set_bad_words(["bad", "words"])?;
        let value = pwq.get_bad_words()?;
        assert_eq!(value, vec!["bad".to_string(), "words".to_string()]);

        #[cfg(feature = "crack")]
        {
            let path = "/path/to/dict";
            pwq.set_dict_path(path)?;

            let s = pwq.get_dict_path()?;

            assert!(s.eq(path));
        }

        let value = 11;
        pwq.set_retry_times(value);
        assert_eq!(pwq.get_retry_times(), value);

        pwq.enable_enforce_root();
        assert!(pwq.enforce_root_enabled());

        pwq.enable_local_users_only();
        assert!(pwq.local_users_only_enabled());

        Ok(())
    }
}
