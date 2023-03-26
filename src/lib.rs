//! libpwquality bindings for Rust

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
pub enum Setting {
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

impl Setting {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::DiffOk => "difok",
            Self::MinLength => "minlen",
            Self::DigCredit => "dcredit",
            Self::UpCredit => "ucredit",
            Self::LowCredit => "lcredit",
            Self::OthCredit => "ocredit",
            Self::MinClass => "minclass",
            Self::MaxRepeat => "maxrepeat",
            Self::DictPath => "dictpath",
            Self::MaxClassRepeat => "maxclassrepeat",
            Self::GecosCheck => "gecoscheck",
            Self::BadWords => "badwords",
            Self::MaxSequence => "maxsequence",
            Self::DictCheck => "dictcheck",
            Self::UserCheck => "usercheck",
            Self::Enforcing => "enforcing",
            Self::RetryTimes => "retry",
            Self::EnforceRoot => "enforce_for_root",
            Self::LocalUsers => "local_users_only",
            Self::UserSubstr => "usersubstr",
        }
    }
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

    /// Useful for setting the options as configured on a pam module command line in form of `<opt>=<val>`.
    pub fn set_option(&self, option: &str) -> Result<()> {
        let c_option = CString::new(option).unwrap();
        let ret = unsafe { pwquality_set_option(self.pwq, c_option.as_ptr()) };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Set value of an integer setting.
    pub fn set_int_value(&self, setting: Setting, value: i32) -> Result<()> {
        let ret = unsafe { pwquality_set_int_value(self.pwq, setting as c_int, value) };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Set value of a string setting.
    pub fn set_str_value(&self, setting: Setting, value: &str) -> Result<()> {
        let value = CString::new(value).unwrap();
        let ret = unsafe { pwquality_set_str_value(self.pwq, setting as c_int, value.as_ptr()) };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Get value of an integer setting.
    pub fn get_int_value(&self, setting: Setting) -> Result<i32> {
        let mut value: i32 = 0;
        let ret = unsafe { pwquality_get_int_value(self.pwq, setting as c_int, &mut value) };
        match ret {
            0 => Ok(value),
            _ => Err(Error::from_aux(ret, None)),
        }
    }

    /// Get value of a string setting.
    pub fn get_str_value(&self, setting: Setting) -> Result<String> {
        let mut ptr: *const c_char = null_mut();

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
}

impl Drop for PWQuality {
    /// Free pwquality settings data.
    fn drop(&mut self) {
        unsafe { pwquality_free_settings(self.pwq) }
    }
}
