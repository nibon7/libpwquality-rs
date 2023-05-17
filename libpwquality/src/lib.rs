//! libpwquality bindings for Rust
//!
//! ## Example
//!
//! ```
//! use libpwquality::{PWQError, PWQuality};
//!
//! fn main() -> Result<(), PWQError> {
//!     let pwq = PWQuality::new()?;
//!
//!     pwq.read_default_config()?
//!         .min_length(9)
//!         .max_repeat(2)
//!         .bad_words(["bad", "password"])?;
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

/// `PWQuality` Error.
#[derive(Debug)]
pub struct PWQError(String);

impl PWQError {
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

impl std::error::Error for PWQError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PWQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = std::result::Result<T, PWQError>;

/// `PWQuality` instance that holds the underlying `pwquality_settings_t`.
pub struct PWQuality {
    pwq: *mut pwquality_settings_t,
}

impl PWQuality {
    /// Create a new `PWQuality` instance.
    pub fn new() -> Result<Self> {
        let pwq = unsafe { pwquality_default_settings() };

        if pwq.is_null() {
            Err(PWQError::from_aux(PWQ_ERROR_MEM_ALLOC, None))
        } else {
            Ok(Self { pwq })
        }
    }

    /// Parse the default configuration file.
    pub fn read_default_config(&self) -> Result<&Self> {
        self.read_optional_config::<&str>(None)
    }

    /// Parse the given configuration file.
    pub fn read_config<P: AsRef<Path>>(&self, path: P) -> Result<&Self> {
        self.read_optional_config(Some(path))
    }

    /// Parse the configuration file.
    fn read_optional_config<P: AsRef<Path>>(&self, path: Option<P>) -> Result<&Self> {
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
            0 => Ok(self),
            _ => Err(PWQError::from_aux(ret, Some(ptr_err))),
        }
    }

    /// Set value of an integer setting.
    fn set_int_value(&self, setting: Setting, value: i32) -> &Self {
        let ret = unsafe { pwquality_set_int_value(self.pwq, setting as c_int, value) };

        debug_assert!(ret == 0);

        self
    }

    /// Get value of an integer setting.
    fn get_int_value(&self, setting: Setting) -> i32 {
        let mut value: i32 = 0;
        let ret = unsafe { pwquality_get_int_value(self.pwq, setting as c_int, &mut value) };

        debug_assert!(ret == 0);

        value
    }

    /// Set value of a string setting.
    fn set_str_value(&self, setting: Setting, value: &str) -> Result<&Self> {
        let value = CString::new(value).unwrap();
        let ret = unsafe { pwquality_set_str_value(self.pwq, setting as c_int, value.as_ptr()) };

        match ret {
            0 => Ok(self),
            _ => Err(PWQError::from_aux(ret, None)),
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
            _ => Err(PWQError::from_aux(ret, None)),
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
            _ => Err(PWQError::from_aux(ret, None)),
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
            Err(PWQError::from_aux(ret, Some(ptr_err)))
        } else {
            Ok(ret)
        }
    }

    /// Set the minimum number of characters in the new password that must not be present in the
    /// old password.
    ///
    /// The special value of 0 disables all checks of similarity of the new password with
    /// the old password except the new password being exactly the same as the old one.
    pub fn min_diff(&self, value: i32) -> &Self {
        self.set_int_value(Setting::DiffOk, value)
    }

    /// Get the minimum number of characters in the new password that must not be present in the
    /// old password.
    pub fn get_min_diff(&self) -> i32 {
        self.get_int_value(Setting::DiffOk)
    }

    /// Set the minimum acceptable size for the new password (plus one if credits are not
    /// disabled which is the default).
    ///
    /// Any number less than 6 will be replaced by 6.
    pub fn min_length(&self, value: i32) -> &Self {
        self.set_int_value(Setting::MinLength, value)
    }

    /// Get the minimum acceptable size for the new password.
    pub fn get_min_length(&self) -> i32 {
        self.get_int_value(Setting::MinLength)
    }

    /// Set the maximum credit for having digits in the new password.
    ///
    /// If less than 0 it is the minimum number of digits in the new password.
    pub fn digit_credit(&self, value: i32) -> &Self {
        self.set_int_value(Setting::DigCredit, value)
    }

    /// Get the maximum credit for having digits in the new password.
    pub fn get_digit_credit(&self) -> i32 {
        self.get_int_value(Setting::DigCredit)
    }

    /// Set the maximum credit for having uppercase characters in the new password.
    ///
    /// If less than 0 it is the minimum number of uppercase characters in the new
    /// password.
    pub fn uppercase_credit(&self, value: i32) -> &Self {
        self.set_int_value(Setting::UpCredit, value)
    }

    /// Get the maximum credit for having uppercase characters in the new password.
    pub fn get_uppercase_credit(&self) -> i32 {
        self.get_int_value(Setting::UpCredit)
    }

    /// Set the maximum credit for having lowercase characters in the new password.
    ///
    /// If less than 0 it is the minimum number of lowercase characters in the new
    /// password.
    pub fn lowercase_credit(&self, value: i32) -> &Self {
        self.set_int_value(Setting::LowCredit, value)
    }

    /// Get the maximum credit for having lowercase characters in the new password.
    pub fn get_lowercase_credit(&self) -> i32 {
        self.get_int_value(Setting::LowCredit)
    }

    /// Set the maximum credit for having other characters in the new password.
    ///
    /// If less than 0 it is the minimum number of other characters in the new
    /// password.
    pub fn other_credit(&self, value: i32) -> &Self {
        self.set_int_value(Setting::OthCredit, value)
    }

    /// Get the maximum credit for having other characters in the new password.
    pub fn get_other_credit(&self) -> i32 {
        self.get_int_value(Setting::OthCredit)
    }

    /// Set the minimum number of required classes of characters for the new
    /// password (digits, uppercase, lowercase, others).
    pub fn min_class(&self, value: i32) -> &Self {
        self.set_int_value(Setting::MinClass, value)
    }

    /// Get the minimum number of required classes of characters for the new
    /// password (digits, uppercase, lowercase, others).
    pub fn get_min_class(&self) -> i32 {
        self.get_int_value(Setting::MinClass)
    }

    /// Set the maximum number of allowed consecutive same characters in the new password.
    ///
    /// The check is disabled if the value is 0.
    pub fn max_repeat(&self, value: i32) -> &Self {
        self.set_int_value(Setting::MaxRepeat, value)
    }

    /// Get the maximum number of allowed consecutive same characters in the new password.
    pub fn get_max_repeat(&self) -> i32 {
        self.get_int_value(Setting::MaxRepeat)
    }

    /// Set the maximum length of monotonic character sequences in the new password.
    ///
    /// Examples of such sequence are '12345' or 'fedcb'.
    /// The check is disabled if the value is 0.
    pub fn max_sequence(&self, value: i32) -> &Self {
        self.set_int_value(Setting::MaxSequence, value)
    }

    /// Get the maximum length of monotonic character sequences in the new password.
    pub fn get_max_sequence(&self) -> i32 {
        self.get_int_value(Setting::MaxSequence)
    }

    /// Set the maximum number of allowed consecutive characters of the same class in the
    /// new password.
    ///
    /// The check is disabled if the value is 0.
    pub fn max_class_repeat(&self, value: i32) -> &Self {
        self.set_int_value(Setting::MaxClassRepeat, value)
    }

    /// Get the maximum number of allowed consecutive characters of the same class in the
    /// new password.
    pub fn get_max_class_repeat(&self) -> i32 {
        self.get_int_value(Setting::MaxClassRepeat)
    }

    /// Set whether to check for the words from the passwd entry GECOS string of the user.
    ///
    /// The check is enabled if the value is not 0.
    pub fn gecos_check(&self, check: bool) -> &Self {
        self.set_int_value(Setting::GecosCheck, i32::from(check))
    }

    /// Get whether to check for the words from the passwd entry GECOS string of the user.
    pub fn get_gecos_check(&self) -> bool {
        self.get_int_value(Setting::GecosCheck) != 0
    }

    /// Set whether to check for the words from the cracklib dictionary.
    ///
    /// The check is enabled if the value is not 0.
    pub fn dict_check(&self, check: bool) -> &Self {
        self.set_int_value(Setting::DictCheck, i32::from(check))
    }

    /// Get whether to check for the words from the cracklib dictionary.
    pub fn get_dict_check(&self) -> bool {
        self.get_int_value(Setting::DictCheck) != 0
    }

    /// Set whether to check if it contains the user name in some form.
    ///
    /// The check is enabled if the value is not 0.
    pub fn user_check(&self, check: bool) -> &Self {
        self.set_int_value(Setting::UserCheck, i32::from(check))
    }

    /// Get whether to check if it contains the user name in some form.
    pub fn get_user_check(&self) -> bool {
        self.get_int_value(Setting::UserCheck) != 0
    }

    /// Set the length of substrings from the username to check for in the password.
    ///
    /// The check is enabled if the value is greater than 0 and usercheck is enabled.
    pub fn user_substr(&self, value: i32) -> &Self {
        self.set_int_value(Setting::UserSubstr, value)
    }

    /// Get the length of substrings from the username to check for in the password.
    pub fn get_user_substr(&self) -> i32 {
        self.get_int_value(Setting::UserSubstr)
    }

    /// Set whether the check is enforced by the PAM module and possibly other
    /// applications.
    pub fn enforcing(&self, enforced: bool) -> &Self {
        self.set_int_value(Setting::Enforcing, i32::from(enforced))
    }

    /// Get whether the check is enforced by the PAM module and possibly other
    /// applications.
    pub fn get_enforcing(&self) -> bool {
        self.get_int_value(Setting::Enforcing) != 0
    }

    /// Set the list of words more than 3 characters long that are forbidden.
    pub fn bad_words<W>(&self, words: W) -> Result<&Self>
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

    /// Get the list of words more than 3 characters long that are forbidden.
    pub fn get_bad_words(&self) -> Result<Vec<String>> {
        self.get_str_value(Setting::BadWords)
            .map(|s| s.split_whitespace().map(String::from).collect())
    }

    /// Set the path to the cracklib dictionaries.
    pub fn dict_path(&self, path: &str) -> Result<&Self> {
        self.set_str_value(Setting::DictPath, path)
    }

    /// Get the path to the cracklib dictionaries.
    pub fn get_dict_path(&self) -> Result<String> {
        self.get_str_value(Setting::DictPath)
    }

    /// Set the maximum times to prompt user before returning with error.
    pub fn retry_times(&self, value: i32) -> &Self {
        self.set_int_value(Setting::RetryTimes, value)
    }

    /// Get the maximum times to prompt user before returning with error.
    pub fn get_retry_times(&self) -> i32 {
        self.get_int_value(Setting::RetryTimes)
    }

    /// Set whether to enforce pwquality checks on the root user password.
    pub fn enforce_for_root(&self, enforced: bool) -> &Self {
        self.set_int_value(Setting::EnforceRoot, i32::from(enforced))
    }

    /// Get whether to enforce pwquality checks on the root user password.
    pub fn get_enforce_for_root(&self) -> bool {
        self.get_int_value(Setting::EnforceRoot) != 0
    }

    /// Set whether to skip testing the password quality for users that are not present in the
    /// /etc/passwd file.
    pub fn local_users_only(&self, local_users_only: bool) -> &Self {
        self.set_int_value(Setting::LocalUsers, i32::from(local_users_only))
    }

    /// Get whether to skip testing the password quality for users that are not present in the
    /// /etc/passwd file.
    pub fn get_local_users_only(&self) -> bool {
        self.get_int_value(Setting::LocalUsers) != 0
    }
}

impl Drop for PWQuality {
    /// Free pwquality settings data.
    fn drop(&mut self) {
        unsafe { pwquality_free_settings(self.pwq) }
    }
}
