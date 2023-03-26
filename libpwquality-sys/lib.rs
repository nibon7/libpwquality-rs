//! Native bindings to the libpwquality library

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_void};

pub const PWQ_SETTING_DIFF_OK: u32 = 1;
pub const PWQ_SETTING_MIN_LENGTH: u32 = 3;
pub const PWQ_SETTING_DIG_CREDIT: u32 = 4;
pub const PWQ_SETTING_UP_CREDIT: u32 = 5;
pub const PWQ_SETTING_LOW_CREDIT: u32 = 6;
pub const PWQ_SETTING_OTH_CREDIT: u32 = 7;
pub const PWQ_SETTING_MIN_CLASS: u32 = 8;
pub const PWQ_SETTING_MAX_REPEAT: u32 = 9;
pub const PWQ_SETTING_DICT_PATH: u32 = 10;
pub const PWQ_SETTING_MAX_CLASS_REPEAT: u32 = 11;
pub const PWQ_SETTING_GECOS_CHECK: u32 = 12;
pub const PWQ_SETTING_BAD_WORDS: u32 = 13;
pub const PWQ_SETTING_MAX_SEQUENCE: u32 = 14;
pub const PWQ_SETTING_DICT_CHECK: u32 = 15;
pub const PWQ_SETTING_USER_CHECK: u32 = 16;
pub const PWQ_SETTING_ENFORCING: u32 = 17;
pub const PWQ_SETTING_RETRY_TIMES: u32 = 18;
pub const PWQ_SETTING_ENFORCE_ROOT: u32 = 19;
pub const PWQ_SETTING_LOCAL_USERS: u32 = 20;
pub const PWQ_SETTING_USER_SUBSTR: u32 = 21;

pub const PWQ_MAX_ENTROPY_BITS: u32 = 256;
pub const PWQ_MIN_ENTROPY_BITS: u32 = 56;
pub const PWQ_MAX_ERROR_MESSAGE_LEN: u32 = 256;

pub const PWQ_ERROR_SUCCESS: i32 = 0;
pub const PWQ_ERROR_FATAL_FAILURE: i32 = -1;
pub const PWQ_ERROR_INTEGER: i32 = -2;
pub const PWQ_ERROR_CFGFILE_OPEN: i32 = -3;
pub const PWQ_ERROR_CFGFILE_MALFORMED: i32 = -4;
pub const PWQ_ERROR_UNKNOWN_SETTING: i32 = -5;
pub const PWQ_ERROR_NON_INT_SETTING: i32 = -6;
pub const PWQ_ERROR_NON_STR_SETTING: i32 = -7;
pub const PWQ_ERROR_MEM_ALLOC: i32 = -8;
pub const PWQ_ERROR_TOO_SIMILAR: i32 = -9;
pub const PWQ_ERROR_MIN_DIGITS: i32 = -10;
pub const PWQ_ERROR_MIN_UPPERS: i32 = -11;
pub const PWQ_ERROR_MIN_LOWERS: i32 = -12;
pub const PWQ_ERROR_MIN_OTHERS: i32 = -13;
pub const PWQ_ERROR_MIN_LENGTH: i32 = -14;
pub const PWQ_ERROR_PALINDROME: i32 = -15;
pub const PWQ_ERROR_CASE_CHANGES_ONLY: i32 = -16;
pub const PWQ_ERROR_ROTATED: i32 = -17;
pub const PWQ_ERROR_MIN_CLASSES: i32 = -18;
pub const PWQ_ERROR_MAX_CONSECUTIVE: i32 = -19;
pub const PWQ_ERROR_EMPTY_PASSWORD: i32 = -20;
pub const PWQ_ERROR_SAME_PASSWORD: i32 = -21;
pub const PWQ_ERROR_CRACKLIB_CHECK: i32 = -22;
pub const PWQ_ERROR_RNG: i32 = -23;
pub const PWQ_ERROR_GENERATION_FAILED: i32 = -24;
pub const PWQ_ERROR_USER_CHECK: i32 = -25;
pub const PWQ_ERROR_GECOS_CHECK: i32 = -26;
pub const PWQ_ERROR_MAX_CLASS_REPEAT: i32 = -27;
pub const PWQ_ERROR_BAD_WORDS: i32 = -28;
pub const PWQ_ERROR_MAX_SEQUENCE: i32 = -29;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pwquality_settings {
    _unused: [u8; 0],
}

pub type pwquality_settings_t = pwquality_settings;
extern "C" {
    pub fn pwquality_default_settings() -> *mut pwquality_settings_t;

    pub fn pwquality_free_settings(pwq: *mut pwquality_settings_t);

    pub fn pwquality_read_config(
        pwq: *mut pwquality_settings_t,
        cfgfile: *const c_char,
        auxerror: *mut *mut c_void,
    ) -> c_int;

    pub fn pwquality_set_option(pwq: *mut pwquality_settings_t, option: *const c_char) -> c_int;

    pub fn pwquality_set_int_value(
        pwq: *mut pwquality_settings_t,
        setting: c_int,
        value: c_int,
    ) -> c_int;

    pub fn pwquality_set_str_value(
        pwq: *mut pwquality_settings_t,
        setting: c_int,
        value: *const c_char,
    ) -> c_int;

    pub fn pwquality_get_int_value(
        pwq: *mut pwquality_settings_t,
        setting: c_int,
        value: *mut c_int,
    ) -> c_int;

    pub fn pwquality_get_str_value(
        pwq: *mut pwquality_settings_t,
        setting: c_int,
        value: *mut *const c_char,
    ) -> c_int;

    pub fn pwquality_generate(
        pwq: *mut pwquality_settings_t,
        entropy_bits: c_int,
        password: *mut *mut c_char,
    ) -> c_int;

    pub fn pwquality_check(
        pwq: *mut pwquality_settings_t,
        password: *const c_char,
        oldpassword: *const c_char,
        user: *const c_char,
        auxerror: *mut *mut c_void,
    ) -> c_int;

    pub fn pwquality_strerror(
        buf: *mut c_char,
        len: usize,
        errcode: c_int,
        auxerror: *mut c_void,
    ) -> *const c_char;
}
