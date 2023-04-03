#[cfg(test)]
mod tests {
    use libpwquality::{Error, PWQuality};
    use serial_test::serial;

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
        pwq.set_min_diff(value);
        assert_eq!(pwq.get_min_diff(), value);

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
