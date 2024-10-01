//! libpwquality bindings for Rust
//!
//! ## Example
//!
//! ```
#![doc = include_str!("../examples/example.rs")]
//! ```
//!
//! ## Cargo features
//!
//! * `vX_Y_Z`:  Build with system libpwquality version X.Y.Z.
//!   v1_0 *Enabled by default*
//!
//! * `vendored`: Build with vendored libpwquality.
//!   This requires cracklib to be installed.
//!   You can also set `CRACKLIB_INCLUDE_PATH` and `CRACKLIB_LIBRARY_PATH`
//!   environment variables to specify the include path and library path.
//!   *Disabled by default*
//!
//! * `vendored-cracklib`: Build with vendored libpwquality and cracklib.
//!   The build script will try to guess the path of cracklib dictionaries,
//!   but you can set `DEFAULT_CRACKLIB_DICT` environment variable to override it.
//!   *Disabled by default*

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use libpwquality_sys as sys;
use paste::paste;
use std::os::raw::{c_char, c_int, c_void};
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::{null, null_mut},
};

macro_rules! define_settings {
    ($($setting:ident $(:$feature:literal)?,)*) => {
        paste! {
            /// `PWQuality` Setting.
            #[derive(Copy, Clone, Debug)]
            #[non_exhaustive]
            enum Setting {
                $(
                    $(#[cfg(any(feature = $feature, feature = "vendored", feature = "vendored-cracklib"))])?
                    $setting = sys::[<PWQ_SETTING_ $setting:snake:upper>] as isize,
                )*
            }
        }
    };
}

define_settings! {
    DiffOk,
    MinLength,
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
    MaxSequence: "v1_2",
    DictCheck: "v1_3",
    UserCheck: "v1_4",
    Enforcing: "v1_4",
    RetryTimes: "v1_4_1",
    EnforceRoot: "v1_4_1",
    LocalUsers: "v1_4_1",
    UserSubstr: "v1_4_3",
}

/// `PWQuality` Error.
pub struct PWQError(String);

impl PWQError {
    fn new_aux(error_code: i32, aux_error: Option<*mut c_void>) -> Self {
        let error = aux_error.unwrap_or(null_mut());
        let ret = unsafe { sys::pwquality_strerror(null_mut(), 0, error_code, error) };

        let s = if ret.is_null() {
            format!("Unknown error: errcode={error_code}")
        } else {
            unsafe { CStr::from_ptr(ret).to_string_lossy().to_string() }
        };

        Self(s)
    }

    fn new(error_code: i32) -> Self {
        Self::new_aux(error_code, None)
    }
}

impl std::error::Error for PWQError {}

impl std::fmt::Debug for PWQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PWQError: {}", self.0)
    }
}

impl std::fmt::Display for PWQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = std::result::Result<T, PWQError>;

macro_rules! define_getseters {
    ($func:ident, $setting:ident, $doc:literal $(,$getter_feature:literal, $setter_feature:literal)?) => {
        paste! {
            $(#[cfg(any(feature = $getter_feature, feature = "vendored", feature = "vendored-cracklib"))])?
            #[doc = "Get " $doc ""]
            pub fn [<get_ $func>] (&self) -> i32 {
                self.get_int_value($crate::Setting::$setting)
            }

            $(#[cfg(any(feature = $setter_feature, feature = "vendored", feature = "vendored-cracklib"))])?
            #[doc = "Set " $doc ""]
            pub fn $func(&self, value: i32) -> &Self {
                self.set_int_value($crate::Setting::$setting, value)
            }
        }
    };
    ($func:ident, $setting:ident, bool, $doc:literal $(,$getter_feature:literal, $setter_feature:literal)?) => {
        paste! {
            $(#[cfg(any(feature = $getter_feature, feature = "vendored", feature = "vendored-cracklib"))])?
            #[doc = "Get " $doc ""]
            pub fn [<get_ $func>] (&self) -> bool {
                self.get_int_value($crate::Setting::$setting) != 0
            }

            $(#[cfg(any(feature = $setter_feature, feature = "vendored", feature = "vendored-cracklib"))])?
            #[doc = "Set " $doc ""]
            pub fn $func(&self, value: bool) -> &Self {
                self.set_int_value($crate::Setting::$setting, i32::from(value))
            }
        }
    };
}

/// `PWQuality` instance that holds the underlying [pwquality_settings_t](sys::pwquality_settings_t).
pub struct PWQuality {
    pwq: *mut sys::pwquality_settings_t,
}

impl PWQuality {
    /// Create a new `PWQuality` instance.
    pub fn new() -> Result<Self> {
        let pwq = unsafe { sys::pwquality_default_settings() };

        if pwq.is_null() {
            Err(PWQError::new(sys::PWQ_ERROR_MEM_ALLOC))
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
        let mut aux_error = null_mut();
        let c_path = path.map(|p| {
            let s = p.as_ref().to_string_lossy().to_string();
            CString::new(s).unwrap()
        });

        let ret = unsafe {
            sys::pwquality_read_config(
                self.pwq,
                c_path.as_ref().map(|s| s.as_ptr()).unwrap_or(null()),
                &mut aux_error,
            )
        };

        if ret == 0 {
            Ok(self)
        } else {
            Err(PWQError::new_aux(ret, Some(aux_error)))
        }
    }

    /// Set value of an integer setting.
    fn set_int_value(&self, setting: Setting, value: i32) -> &Self {
        let ret = unsafe { sys::pwquality_set_int_value(self.pwq, setting as c_int, value) };

        debug_assert!(ret == 0);

        self
    }

    /// Get value of an integer setting.
    fn get_int_value(&self, setting: Setting) -> i32 {
        let mut value: i32 = 0;
        let ret = unsafe { sys::pwquality_get_int_value(self.pwq, setting as c_int, &mut value) };

        debug_assert!(ret == 0);

        value
    }

    /// Set value of a string setting.
    fn set_str_value(&self, setting: Setting, value: &str) -> Result<&Self> {
        let value = CString::new(value).unwrap();
        let ret =
            unsafe { sys::pwquality_set_str_value(self.pwq, setting as c_int, value.as_ptr()) };

        if ret == 0 {
            Ok(self)
        } else {
            Err(PWQError::new(ret))
        }
    }

    /// Get value of a string setting.
    fn get_str_value(&self, setting: Setting) -> Result<String> {
        let mut ptr: *const c_char = null();

        let ret = unsafe { sys::pwquality_get_str_value(self.pwq, setting as c_int, &mut ptr) };
        if ret == 0 {
            let s = if ptr.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() }
            };

            Ok(s)
        } else {
            Err(PWQError::new(ret))
        }
    }

    /// Generate a random password of entropy_bits entropy and check it according to the settings.
    pub fn generate(&self, bits: i32) -> Result<String> {
        let mut ptr: *mut c_char = null_mut();
        let ret = unsafe { sys::pwquality_generate(self.pwq, bits, &mut ptr) };

        if ptr.is_null() {
            Err(PWQError::new(ret))
        } else {
            let password = unsafe {
                let str_password = CStr::from_ptr(ptr).to_string_lossy().to_string();

                // free the memory allocated in the C library
                libc::free(ptr as *mut c_void);

                str_password
            };

            Ok(password)
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
        let mut aux_error = null_mut();

        let c_old_password = old_password.map(|s| CString::new(s).unwrap());
        let c_user = user.map(|s| CString::new(s).unwrap());

        let ret = unsafe {
            sys::pwquality_check(
                self.pwq,
                c_password.as_ptr(),
                c_old_password
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(null()),
                c_user.as_ref().map(|s| s.as_ptr()).unwrap_or(null()),
                &mut aux_error,
            )
        };

        if ret < 0 {
            Err(PWQError::new_aux(ret, Some(aux_error)))
        } else {
            Ok(ret)
        }
    }

    define_getseters! {
        min_diff,
        DiffOk,
        "the minimum number of characters in the new password that must not be present in the old password."
    }

    define_getseters! {
        min_length,
        MinLength,
        "the minimum acceptable size for the new password."
    }

    define_getseters! {
        digit_credit,
        DigCredit,
        "the maximum credit for having digits in the new password."
    }

    define_getseters! {
        uppercase_credit,
        UpCredit,
        "the maximum credit for having uppercase characters in the new password."
    }

    define_getseters! {
        lowercase_credit,
        LowCredit,
        "the maximum credit for having lowercase characters in the new password."
    }

    define_getseters! {
        other_credit,
        OthCredit,
        "the maximum credit for having other characters in the new password."
    }

    define_getseters! {
        min_class,
        MinClass,
        "the minimum number of required classes of characters for the new password."
    }

    define_getseters! {
        max_repeat,
        MaxRepeat,
        "the maximum number of allowed same consecutive characters in the new password."
    }

    define_getseters! {
        max_class_repeat,
        MaxClassRepeat,
        "the maximum number of allowed consecutive characters of the same class in the new password."
    }

    define_getseters! {
        max_sequence,
        MaxSequence,
        "the maximum length of monotonic character sequences in the new password.",
        "v1_2",
        "v1_2"
    }

    define_getseters! {
        gecos_check,
        GecosCheck,
        bool,
        "whether to perform the passwd GECOS field check."
    }

    define_getseters! {
        dict_check,
        DictCheck,
        bool,
        "whether to perform the dictionary check.",
        "v1_3",
        "v1_3"
    }

    define_getseters! {
        user_check,
        UserCheck,
        bool,
        "whether to perform the user name check.",
        "v1_4",
        "v1_4"
    }

    define_getseters! {
        enforcing,
        Enforcing,
        bool,
        "whether the check is enforced.",
        "v1_4",
        "v1_4"
    }

    define_getseters! {
        retry_times,
        RetryTimes,
        "maximum retries for the password change should be allowed.",
        "v1_4_1",
        "v1_4_1"
    }

    define_getseters! {
        enforce_for_root,
        EnforceRoot,
        bool,
        "whether the check is enforced for root.",
        "v1_4_1",
        "v1_4_1"
    }

    define_getseters! {
        local_users_only,
        LocalUsers,
        bool,
        "whether to check local users only.",
        "v1_4_1",
        "v1_4_1"
    }

    define_getseters! {
        user_substr,
        UserSubstr,
        "the length of substrings of the user name to check.",
        "v1_4_5",
        "v1_4_3"
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
}

impl Drop for PWQuality {
    /// Free pwquality settings data.
    fn drop(&mut self) {
        unsafe { sys::pwquality_free_settings(self.pwq) }
    }
}
