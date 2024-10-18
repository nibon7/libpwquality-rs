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
    ($($(#[$meta:meta])? $setting:ident),* $(,)?) => {
        paste! {
            /// `PWQuality` Setting.
            #[derive(Copy, Clone, Debug)]
            #[non_exhaustive]
            enum Setting {
                $(
                    $(#[$meta])?
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
    #[cfg(any(feature = "v1_2", feature = "vendored", feature = "vendored-cracklib"))]
    MaxSequence,
    #[cfg(any(feature = "v1_3", feature = "vendored", feature = "vendored-cracklib"))]
    DictCheck,
    #[cfg(any(feature = "v1_4", feature = "vendored", feature = "vendored-cracklib"))]
    UserCheck,
    #[cfg(any(feature = "v1_4", feature = "vendored", feature = "vendored-cracklib"))]
    Enforcing,
    #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
    RetryTimes,
    #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
    EnforceRoot,
    #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
    LocalUsers,
    #[cfg(any(feature = "v1_4_3", feature = "vendored", feature = "vendored-cracklib"))]
    UserSubstr,
}

/// `PWQuality` Error.
#[derive(Debug)]
pub struct PWQError {
    code: i32,
    message: String,
}

impl PWQError {
    unsafe fn new_aux(error_code: i32, aux_error: Option<*mut c_void>) -> Self {
        unsafe {
            let s =
                sys::pwquality_strerror(null_mut(), 0, error_code, aux_error.unwrap_or(null_mut()))
                    .as_ref()
                    .map(|p| CStr::from_ptr(p).to_string_lossy().to_string())
                    .unwrap_or("Unknown error".into());

            Self {
                code: error_code,
                message: s,
            }
        }
    }

    fn new(error_code: i32) -> Self {
        unsafe { Self::new_aux(error_code, None) }
    }
}

impl std::error::Error for PWQError {}

impl std::fmt::Display for PWQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (error code {})", self.message, self.code)
    }
}

type Result<T> = std::result::Result<T, PWQError>;

macro_rules! define_getseters {
    (#[$doc_meta:meta] $(#[$feat_meta:meta])? $func:ident, $setting:ident) => {
        paste! {
            $(#[$feat_meta])?
            #[doc = " Get"]
            #[$doc_meta]
            pub fn [<get_ $func>] (&self) -> i32 {
                self.get_int_value($crate::Setting::$setting)
            }

            $(#[$feat_meta])?
            #[doc = " Set"]
            #[$doc_meta]
            pub fn $func(&self, value: i32) -> &Self {
                self.set_int_value($crate::Setting::$setting, value)
            }
        }
    };
    (#[$doc_meta:meta] $(#[$feat_meta:meta])? $func:ident, $setting:ident, bool) => {
        paste! {
            $(#[$feat_meta])?
            #[doc = " Get"]
            #[$doc_meta]
            pub fn [<get_ $func>] (&self) -> bool {
                self.get_int_value($crate::Setting::$setting) != 0
            }

            $(#[$feat_meta])?
            #[doc = " Set"]
            #[$doc_meta]
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
        unsafe {
            let ptr = sys::pwquality_default_settings();

            ptr.as_ref()
                .ok_or(PWQError::new(sys::PWQ_ERROR_MEM_ALLOC))
                .map(|_| Self { pwq: ptr })
        }
    }

    #[cfg(any(feature = "vendored", feature = "vendored-cracklib"))]
    /// Set the default configuration file name.
    pub fn config_name(&self, cfgname: Option<&str>) -> Result<&Self> {
        let c_cfgname = cfgname.map(CString::new).transpose().unwrap();

        let ret = unsafe {
            sys::pwquality_set_config_name(
                self.pwq,
                c_cfgname.as_ref().map(|s| s.as_ptr()).unwrap_or(null()),
            )
        };

        if ret == 0 {
            Ok(self)
        } else {
            Err(PWQError::new(ret))
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
        let c_path = path
            .map(|p| CString::new(p.as_ref().to_string_lossy().to_string()))
            .transpose()
            .unwrap();

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
            Err(unsafe { PWQError::new_aux(ret, Some(aux_error)) })
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
            let s = unsafe {
                ptr.as_ref()
                    .map(|p| CStr::from_ptr(p).to_string_lossy().to_string())
                    .unwrap_or_default()
            };

            Ok(s)
        } else {
            Err(PWQError::new(ret))
        }
    }

    /// Generate a random password of entropy_bits entropy and check it according to the settings.
    pub fn generate(&self, bits: i32) -> Result<String> {
        let mut ptr: *mut c_char = null_mut();
        unsafe {
            let ret = sys::pwquality_generate(self.pwq, bits, &mut ptr);

            ptr.as_ref().ok_or(PWQError::new(ret)).map(|p| {
                let s = CStr::from_ptr(p).to_string_lossy().to_string();

                // free the memory allocated in the C library
                libc::free(ptr.cast());

                s
            })
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

        let c_old_password = old_password.map(CString::new).transpose().unwrap();
        let c_user = user.map(CString::new).transpose().unwrap();

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
            Err(unsafe { PWQError::new_aux(ret, Some(aux_error)) })
        } else {
            Ok(ret)
        }
    }

    define_getseters! {
        #[doc = " the minimum number of characters in the new password that must not be present in the old password."]
        min_diff,
        DiffOk
    }

    define_getseters! {
        #[doc = " the minimum acceptable size for the new password."]
        min_length,
        MinLength
    }

    define_getseters! {
        #[doc = " the maximum credit for having digits in the new password."]
        digit_credit,
        DigCredit
    }

    define_getseters! {
        #[doc = " the maximum credit for having uppercase characters in the new password."]
        uppercase_credit,
        UpCredit
    }

    define_getseters! {
        #[doc = " the maximum credit for having lowercase characters in the new password."]
        lowercase_credit,
        LowCredit
    }

    define_getseters! {
        #[doc = " the maximum credit for having other characters in the new password."]
        other_credit,
        OthCredit
    }

    define_getseters! {
        #[doc = " the minimum number of required classes of characters for the new password."]
        min_class,
        MinClass
    }

    define_getseters! {
        #[doc = " the maximum number of allowed same consecutive characters in the new password."]
        max_repeat,
        MaxRepeat
    }

    define_getseters! {
        #[doc = " the maximum number of allowed consecutive characters of the same class in the new password."]
        max_class_repeat,
        MaxClassRepeat
    }

    define_getseters! {
        #[doc = " the maximum length of monotonic character sequences in the new password."]
        #[cfg(any(feature = "v1_2", feature = "vendored", feature = "vendored-cracklib"))]
        max_sequence,
        MaxSequence
    }

    define_getseters! {
        #[doc = " whether to perform the passwd GECOS field check."]
        gecos_check,
        GecosCheck,
        bool
    }

    define_getseters! {
        #[doc = " whether to perform the dictionary check."]
        #[cfg(any(feature = "v1_3", feature = "vendored", feature = "vendored-cracklib"))]
        dict_check,
        DictCheck,
        bool
    }

    define_getseters! {
        #[doc = " whether to perform the user name check."]
        #[cfg(any(feature = "v1_4", feature = "vendored", feature = "vendored-cracklib"))]
        user_check,
        UserCheck,
        bool
    }

    define_getseters! {
        #[doc = " whether the check is enforced."]
        #[cfg(any(feature = "v1_4", feature = "vendored", feature = "vendored-cracklib"))]
        enforcing,
        Enforcing,
        bool
    }

    define_getseters! {
        #[doc = " maximum retries for the password change should be allowed."]
        #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
        retry_times,
        RetryTimes
    }

    define_getseters! {
        #[doc = " whether the check is enforced for root."]
        #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
        enforce_for_root,
        EnforceRoot,
        bool
    }

    define_getseters! {
        #[doc = " whether to check local users only."]
        #[cfg(any(feature = "v1_4_1", feature = "vendored", feature = "vendored-cracklib"))]
        local_users_only,
        LocalUsers,
        bool
    }

    #[cfg(any(
        feature = "v1_4_5",
        feature = "vendored",
        feature = "vendored-cracklib"
    ))]
    /// Get the length of substrings of the user name to check.
    pub fn get_user_substr(&self) -> i32 {
        self.get_int_value(crate::Setting::UserSubstr)
    }

    #[cfg(any(
        feature = "v1_4_3",
        feature = "vendored",
        feature = "vendored-cracklib"
    ))]
    /// Set the length of substrings of the user name to check.
    pub fn user_substr(&self, value: i32) -> &Self {
        self.set_int_value(crate::Setting::UserSubstr, value)
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
