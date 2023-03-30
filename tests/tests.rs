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

            let value = pwq.get_int_value(key)?;

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

            pwq.set_int_value(key, val)?;

            let value = pwq.get_int_value(key)?;

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
}
