//! libpwquality bindings for Rust
//!
//! ## Example
//!
//! ```
//! use libpwquality::{Error, PWQuality};
//!
//! fn main() -> Result<(), Error> {
//!     let pwq = PWQuality::new()?;
//!
//!     pwq.read_default_config()?;
//!
//!     pwq.set_min_length(9);
//!
//!     let password = pwq.generate(32)?;
//!
//!     println!("password={:?}", password);
//!
//!     let score = pwq.check(&password, None, None)?;
//!
//!     println!("score={}", score);
//!
//!     Ok(())
//! }
//! ```

use libpwquality_sys::*;
use std::os::raw::{c_char, c_int, c_void};
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::{null, null_mut},
};

/// PWQuality Setting.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
enum Setting {
    DiffOk = PWQ_SETTING_DIFF_OK as isize,
    MinLength = PWQ_SETTING_MIN_LENGTH as isize,
    DigCredit,
    UpCredit,
    LowCredit,
    OthCredit,
    MinClass,
    MaxRepeat,
    DictPath,
    MaxClassRepeat,
    GecosCheck,
    BadWords,
    MaxSequence,
    DictCheck,
    UserCheck,
    Enforcing,
    RetryTimes,
    EnforceRoot,
    LocalUsers,
    UserSubstr,
}

/// PWQuality Error.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    fn from_aux(errorkind: i32, auxerror: Option<*mut c_void>) -> Self {
        let ret = match auxerror {
            Some(aux) => unsafe { pwquality_strerror(null_mut(), 0, errorkind, aux) },
            None => unsafe { pwquality_strerror(null_mut(), 0, errorkind, null_mut()) },
        };

        debug_assert!(!ret.is_null());
        let s = unsafe { CStr::from_ptr(ret).to_string_lossy().to_string() };

        Self(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = std::result::Result<T, Error>;

/// `PWQuality` instance that holds the underlying `pwquality_settings_t`.
pub struct PWQuality {
    pwq: *mut pwquality_settings_t,
}

impl PWQuality {
    /// Create a new `PWQuality` instance.
    pub fn new() -> Result<Self> {
        let pwq = unsafe { pwquality_default_settings() };

        if pwq.is_null() {
            Err(Error::from_aux(PWQ_ERROR_MEM_ALLOC, None))
        } else {
            Ok(Self { pwq })
        }
    }

    /// Parse the default configuration file.
    pub fn read_default_config(&self) -> Result<()> {
        self.read_optional_config::<&str>(None)
    }

    /// Parse the given configuration file.
    pub fn read_config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.read_optional_config(Some(path))
    }

    /// Parse the configuration file.
    fn read_optional_config<P: AsRef<Path>>(&self, path: Option<P>) -> Result<()> {
        let mut ptr_err = null_mut();
        let ret = match path {
            Some(path) => {
                let str_path = path.as_ref().to_string_lossy().to_string();
                let c_path = CString::new(str_path).unwrap();

                unsafe { pwquality_read_config(self.pwq, c_path.as_ptr(), &mut ptr_err) }
            }
            None => unsafe { pwquality_read_config(self.pwq, null(), &mut ptr_err) },
        };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from_aux(ret, Some(ptr_err))),
        }
    }

    /// Set value of an integer setting.
    fn set_int_value(&self, setting: Setting, value: i32) {
        let ret = unsafe { pwquality_set_int_value(self.pwq, setting as c_int, value) };

        debug_assert!(ret == 0);
    }

    /// Get value of an integer setting.
    fn get_int_value(&self, setting: Setting) -> i32 {
        let mut value: i32 = 0;
        let ret = unsafe { pwquality_get_int_value(self.pwq, setting as c_int, &mut value) };

        debug_assert!(ret == 0);

        value
    }

    /// Set value of a string setting.
    fn set_str_value(&self, setting: Setting, value: &str) -> Result<()> {
        let value = CString::new(value).unwrap();
        let ret = unsafe { pwquality_set_str_value(self.pwq, setting as c_int, value.as_ptr()) };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Get value of a string setting.
    fn get_str_value(&self, setting: Setting) -> Result<String> {
        let mut ptr: *const c_char = null();

        let ret = unsafe { pwquality_get_str_value(self.pwq, setting as c_int, &mut ptr) };
        match ret {
            0 => {
                let s = if ptr.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() }
                };

                Ok(s)
            }
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Generate a random password of entropy_bits entropy and check it according to the settings.
    pub fn generate(&self, bits: i32) -> Result<String> {
        let mut ptr: *mut c_char = null_mut();
        let ret = unsafe { pwquality_generate(self.pwq, bits, &mut ptr) };
        match ret {
            0 => {
                let password = unsafe {
                    debug_assert!(!ptr.is_null());
                    let str_password = CStr::from_ptr(ptr).to_string_lossy().to_string();

                    // free the memory allocated in the C library
                    libc::free(ptr as *mut c_void);

                    str_password
                };

                Ok(password)
            }
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Check the password according to the settings.
    pub fn check(
        &self,
        password: &str,
        old_password: Option<&str>,
        user: Option<&str>,
    ) -> Result<i32> {
        let c_password = CString::new(password).unwrap();
        let mut ptr_err = null_mut();

        let ret = match (old_password, user) {
            (None, None) => unsafe {
                pwquality_check(self.pwq, c_password.as_ptr(), null(), null(), &mut ptr_err)
            },
            (Some(old_password), None) => {
                let c_old_password = CString::new(old_password).unwrap();

                unsafe {
                    pwquality_check(
                        self.pwq,
                        c_password.as_ptr(),
                        c_old_password.as_ptr(),
                        null(),
                        &mut ptr_err,
                    )
                }
            }
            (None, Some(user)) => {
                let c_user = CString::new(user).unwrap();

                unsafe {
                    pwquality_check(
                        self.pwq,
                        c_password.as_ptr(),
                        null(),
                        c_user.as_ptr(),
                        &mut ptr_err,
                    )
                }
            }
            (Some(old_password), Some(user)) => {
                let c_old_password = CString::new(old_password).unwrap();
                let c_user = CString::new(user).unwrap();

                unsafe {
                    pwquality_check(
                        self.pwq,
                        c_password.as_ptr(),
                        c_old_password.as_ptr(),
                        c_user.as_ptr(),
                        &mut ptr_err,
                    )
                }
            }
        };

        if ret < 0 {
            Err(Error::from_aux(ret, Some(ptr_err)))
        } else {
            Ok(ret)
        }
    }

    /// Set the minimum number of characters in the new password that must not be present in the
    /// old password.
    /// The special value of 0 disables all checks of similarity of the new password with
    /// the old password except the new password being exactly the same as the old one.
    pub fn set_min_diff(&self, value: i32) {
        self.set_int_value(Setting::DiffOk, value)
    }

    /// Get the minimum number of characters in the new password that must not be present in the
    /// old password.
    pub fn get_min_diff(&self) -> i32 {
        self.get_int_value(Setting::DiffOk)
    }

    /// Set the minimum acceptable size for the new password (plus one if credits are not
    /// disabled which is the default).
    /// Any number less than 6 will be replaced by 6.
    pub fn set_min_length(&self, value: i32) {
        self.set_int_value(Setting::MinLength, value)
    }

    /// Get the minimum acceptable size for the new password.
    pub fn get_min_length(&self) -> i32 {
        self.get_int_value(Setting::MinLength)
    }

    /// Set the maximum credit for having digits in the new password.
    /// If less than 0 it is the minimum number of digits in the new password.
    pub fn set_digit_credit(&self, value: i32) {
        self.set_int_value(Setting::DigCredit, value)
    }

    /// Get the maximum credit for having digits in the new password.
    pub fn get_digit_credit(&self) -> i32 {
        self.get_int_value(Setting::DigCredit)
    }

    /// Set maximum credit for having uppercase characters in the new password.
    /// If less than 0 it is the minimum number of uppercase characters in the new
    /// password.
    pub fn set_uppercase_credit(&self, value: i32) {
        self.set_int_value(Setting::UpCredit, value)
    }

    /// Get the maximum credit for having uppercase characters in the new password.
    pub fn get_uppercase_credit(&self) -> i32 {
        self.get_int_value(Setting::UpCredit)
    }

    /// Set the maximum credit for having lowercase characters in the new password.
    /// If less than 0 it is the minimum number of lowercase characters in the new
    /// password.
    pub fn set_lowercase_credit(&self, value: i32) {
        self.set_int_value(Setting::LowCredit, value)
    }

    /// Get the maximum credit for having lowercase characters in the new password.
    pub fn get_lowercase_credit(&self) -> i32 {
        self.get_int_value(Setting::LowCredit)
    }

    /// Set the maximum credit for having other characters in the new password.
    /// If less than 0 it is the minimum number of other characters in the new
    /// password.
    pub fn set_other_credit(&self, value: i32) {
        self.set_int_value(Setting::OthCredit, value)
    }

    /// Get the maximum credit for having other characters in the new password.
    pub fn get_other_credit(&self) -> i32 {
        self.get_int_value(Setting::OthCredit)
    }

    /// Set the minimum number of required classes of characters for the new
    /// password (digits, uppercase, lowercase, others).
    pub fn set_min_class(&self, value: i32) {
        self.set_int_value(Setting::MinClass, value)
    }

    /// Get the minimum number of required classes of characters for the new
    /// password (digits, uppercase, lowercase, others).
    pub fn get_min_class(&self) -> i32 {
        self.get_int_value(Setting::MinClass)
    }

    /// Set the maximum number of allowed consecutive same characters in the new password.
    /// The check is disabled if the value is 0.
    pub fn set_max_repeat(&self, value: i32) {
        self.set_int_value(Setting::MaxRepeat, value)
    }

    /// Get the maximum number of allowed consecutive same characters in the new password.
    pub fn get_max_repeat(&self) -> i32 {
        self.get_int_value(Setting::MaxRepeat)
    }

    /// Set the maximum length of monotonic character sequences in the new password.
    /// Examples of such sequence are '12345' or 'fedcb'.
    /// The check is disabled if the value is 0.
    pub fn set_max_seqeunce(&self, value: i32) {
        self.set_int_value(Setting::MaxSequence, value)
    }

    /// Get the maximum length of monotonic character sequences in the new password.
    pub fn get_max_seqeunce(&self) -> i32 {
        self.get_int_value(Setting::MaxSequence)
    }

    /// Set the maximum number of allowed consecutive characters of the same class in the
    /// new password.
    /// The check is disabled if the value is 0.
    pub fn set_max_class_repeat(&self, value: i32) {
        self.set_int_value(Setting::MaxClassRepeat, value)
    }

    /// Get the maximum number of allowed consecutive characters of the same class in the
    /// new password.
    pub fn get_max_class_repeat(&self) -> i32 {
        self.get_int_value(Setting::MaxClassRepeat)
    }

    /// Set whether to check for the words from the passwd entry GECOS string of the user.
    /// The check is enabled if the value is not 0.
    pub fn set_gecos_check(&self, check: bool) {
        self.set_int_value(Setting::GecosCheck, i32::from(check))
    }

    /// Get whether to check for the words from the passwd entry GECOS string of the user.
    pub fn get_gecos_check(&self) -> bool {
        self.get_int_value(Setting::GecosCheck) != 0
    }

    /// Set whether to check for the words from the cracklib dictionary.
    /// The check is enabled if the value is not 0.
    pub fn set_dict_check(&self, check: bool) {
        self.set_int_value(Setting::DictCheck, i32::from(check))
    }

    /// Get whether to check for the words from the cracklib dictionary.
    pub fn get_dict_check(&self) -> bool {
        self.get_int_value(Setting::DictCheck) != 0
    }

    /// Set whether to check if it contains the user name in some form.
    /// The check is enabled if the value is not 0.
    pub fn set_user_check(&self, check: bool) {
        self.set_int_value(Setting::UserCheck, i32::from(check))
    }

    /// Get whether to check if it contains the user name in some form.
    pub fn get_user_check(&self) -> bool {
        self.get_int_value(Setting::UserCheck) != 0
    }

    /// Set length of substrings from the username to check for in the password.
    /// The check is enabled if the value is greater than 0 and usercheck is enabled.
    pub fn set_user_substr(&self, value: i32) {
        self.set_int_value(Setting::UserSubstr, value)
    }

    /// Get length of substrings from the username to check for in the password.
    pub fn get_user_substr(&self) -> i32 {
        self.get_int_value(Setting::UserSubstr)
    }

    /// Set whether the check is enforced by the PAM module and possibly other
    /// applications.
    pub fn set_enforcing(&self, enforced: bool) {
        self.set_int_value(Setting::Enforcing, i32::from(enforced))
    }

    /// Get whether the check is enforced by the PAM module and possibly other
    /// applications.
    pub fn get_enforcing(&self) -> bool {
        self.get_int_value(Setting::Enforcing) != 0
    }

    /// Set list of words more than 3 characters long that are forbidden.
    pub fn set_bad_words<W>(&self, words: W) -> Result<()>
    where
        W: IntoIterator,
        W::Item: ToString,
    {
        let s = words
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        self.set_str_value(Setting::BadWords, &s)
    }

    /// Get list of words more than 3 characters long that are forbidden.
    pub fn get_bad_words(&self) -> Result<Vec<String>> {
        self.get_str_value(Setting::BadWords)
            .map(|s| s.split_whitespace().map(String::from).collect())
    }
    /// Set the path to the cracklib dictionaries.
    pub fn set_dict_path(&self, path: &str) -> Result<()> {
        self.set_str_value(Setting::DictPath, path)
    }

    /// Get the path to the cracklib dictionaries.
    pub fn get_dict_path(&self) -> Result<String> {
        self.get_str_value(Setting::DictPath)
    }

    /// Set the maximum times to prompt user before returning with error.
    pub fn set_retry_times(&self, value: i32) {
        self.set_int_value(Setting::RetryTimes, value)
    }

    /// Get the maximum times to prompt user before returning with error.
    pub fn get_retry_times(&self) -> i32 {
        self.get_int_value(Setting::RetryTimes)
    }

    /// Enable enforced pwquality checks on the root user password.
    pub fn enable_enforce_root(&self) {
        self.set_int_value(Setting::EnforceRoot, 1)
    }

    /// Return whether enforced pwquality checks on the root user password is enabled.
    pub fn enforce_root_enabled(&self) -> bool {
        self.get_int_value(Setting::EnforceRoot) != 0
    }

    /// Enable testing the password quality for local users only.
    pub fn enable_local_users_only(&self) {
        self.set_int_value(Setting::LocalUsers, 1)
    }

    /// Return whether testing password quality for local users only is enabled.
    pub fn local_users_only_enabled(&self) -> bool {
        self.get_int_value(Setting::LocalUsers) != 0
    }
}

impl Drop for PWQuality {
    /// Free pwquality settings data.
    fn drop(&mut self) {
        unsafe { pwquality_free_settings(self.pwq) }
    }
}
