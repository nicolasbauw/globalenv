//! Globally set or unset environment variables (and not just for the current process).
//! Example:
//! ```rust
//! use globalenv::{set_var, unset_var};
//! set_var("ENVTEST", "TESTVALUE").unwrap();
//! unset_var("ENVTEST").unwrap();
//! ```

use std::io;
use winreg::enums::*;
use winreg::RegKey;

/// Sets a global environment variable. Support for Windows. Linux support TBD.
pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.set_value(var, &value)?;
    Ok(())
}

/// Unsets an environment variable. Support for Windows. Linux support TBD.
pub fn unset_var(var: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.delete_value(var)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use winreg::enums::*;
    use winreg::RegKey;
    #[test]
    fn set() {
        crate::set_var("ENVTEST", "TESTVALUE").unwrap();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .unwrap();
        let var: String = key.get_value("ENVTEST").unwrap();
        assert_eq!(String::from("TESTVALUE"), var);
    }

    #[test]
    #[should_panic]
    fn unset() {
        crate::unset_var("ENVTEST").unwrap();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .unwrap();
        let _: String = key.get_value("ENVTEST").unwrap();
    }
}
