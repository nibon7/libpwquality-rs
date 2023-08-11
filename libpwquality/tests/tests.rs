use libpwquality::PWQuality;
use serial_test::serial;

const MAX: i32 = 10;

#[test]
#[serial]
fn test_read_config() {
    let pwq = PWQuality::new().unwrap();
    let ret = pwq.read_config("/invalid/path/pwquality.conf");

    assert!(ret.is_err());
}

#[test]
#[serial]
fn test_generate() {
    let pwq = PWQuality::new().unwrap();
    let password = pwq.generate(32).unwrap();

    assert!(!password.is_empty());
}

#[test]
#[serial]
fn test_check() {
    let pwq = PWQuality::new().unwrap();
    let score = pwq.check("p@s5w0rD!", None, None).unwrap();

    assert!(score >= 0);
}

#[test]
#[serial]
fn test_min_diff() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.min_diff(value);
        assert_eq!(pwq.get_min_diff(), value);
    }
}

#[test]
#[serial]
fn test_min_length() {
    const PWQ_BASE_MIN_LENGTH: i32 = 6;
    let pwq = PWQuality::new().unwrap();

    for value in 1..PWQ_BASE_MIN_LENGTH {
        pwq.min_length(value);
        assert_eq!(pwq.get_min_length(), PWQ_BASE_MIN_LENGTH);
    }

    for value in PWQ_BASE_MIN_LENGTH..MAX {
        pwq.min_length(value);
        assert_eq!(pwq.get_min_length(), value);
    }
}

#[test]
#[serial]
fn test_digit_credit() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.digit_credit(value);
        assert_eq!(pwq.get_digit_credit(), value);
    }
}

#[test]
#[serial]
fn test_uppercase_credit() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.uppercase_credit(value);
        assert_eq!(pwq.get_uppercase_credit(), value);
    }
}

#[test]
#[serial]
fn test_lowercase_credit() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.lowercase_credit(value);
        assert_eq!(pwq.get_lowercase_credit(), value);
    }
}

#[test]
#[serial]
fn test_other_credit() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.other_credit(value);
        assert_eq!(pwq.get_other_credit(), value);
    }
}

#[test]
#[serial]
fn test_min_class() {
    const PWQ_NUM_CLASSES: i32 = 4;
    let pwq = PWQuality::new().unwrap();

    for value in 1..=PWQ_NUM_CLASSES {
        pwq.min_class(value);
        assert_eq!(pwq.get_min_class(), value);
    }

    for value in PWQ_NUM_CLASSES..MAX {
        pwq.min_class(value);
        assert_eq!(pwq.get_min_class(), PWQ_NUM_CLASSES);
    }
}

#[test]
#[serial]
fn test_max_repeat() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.max_repeat(value);
        assert_eq!(pwq.get_max_repeat(), value);
    }
}

#[test]
#[serial]
fn test_max_sequence() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.max_sequence(value);
        assert_eq!(pwq.get_max_sequence(), value);
    }
}

#[test]
#[serial]
fn test_max_class_repeat() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.max_class_repeat(value);
        assert_eq!(pwq.get_max_class_repeat(), value);
    }
}

#[test]
#[serial]
fn test_gecos_check() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.gecos_check(value);
        assert_eq!(pwq.get_gecos_check(), value);
    }
}

#[test]
#[serial]
fn test_dict_check() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.dict_check(value);
        assert_eq!(pwq.get_dict_check(), value);
    }
}

#[test]
#[serial]
fn test_user_check() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.user_check(value);
        assert_eq!(pwq.get_user_check(), value);
    }
}

#[test]
#[serial]
fn test_user_substr() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.user_substr(value);
        assert_eq!(pwq.get_user_substr(), value);
    }
}

#[test]
#[serial]
fn test_enforcing() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.enforcing(value);
        assert_eq!(pwq.get_enforcing(), value);
    }
}

#[test]
#[serial]
fn test_bad_words() {
    let pwq = PWQuality::new().unwrap();

    pwq.bad_words(["bad", "words"]).unwrap();
    let value = pwq.get_bad_words().unwrap();
    assert_eq!(value, vec!["bad".to_string(), "words".to_string()]);
}

#[test]
#[serial]
fn test_dict_path() {
    let pwq = PWQuality::new().unwrap();

    for path in ["/path/to/dict", "/path/to/dict2"] {
        pwq.dict_path(path).unwrap();
        let s = pwq.get_dict_path().unwrap();
        assert!(s.eq(path));
    }
}

#[test]
#[serial]
fn test_retry_times() {
    let pwq = PWQuality::new().unwrap();

    for value in 1..MAX {
        pwq.retry_times(value);
        assert_eq!(pwq.get_retry_times(), value);
    }
}

#[test]
#[serial]
fn test_enforce_for_root() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.enforce_for_root(value);
        assert_eq!(pwq.get_enforce_for_root(), value);
    }
}

#[test]
#[serial]
fn test_local_users_only() {
    let pwq = PWQuality::new().unwrap();

    for value in [true, false] {
        pwq.local_users_only(value);
        assert_eq!(pwq.get_local_users_only(), value);
    }
}
