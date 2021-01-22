//! Globally set or unset environment variables (and not just for the current process).
//! Example:
//! ```rust
//! use globalenv::{set_var, unset_var};
//! set_var("ENVTEST", "TESTVALUE").unwrap();
//! unset_var("ENVTEST").unwrap();
//! ```

#[cfg(target_os = "windows")]
use std::{io, env};
#[cfg(target_os = "windows")]
use winreg::{ enums::*, RegKey };

#[cfg(target_os = "macos")]
use std::{ fs, io, io::prelude::*, path::Path, fs::OpenOptions };

/// Sets a global environment variable, usable in current process without reload. Support for Windows. Linux support TBD.
#[cfg(target_os = "windows")]
pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.set_value(var, &value)?;
    env::set_var(var, value);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    // Reading the env file
    let env = fs::read_to_string("/Users/nicolasb/.zshenv").unwrap();

    // Building the "export" line according to requested parameters
    let mut v = String::from("export ");
    v.push_str(var);
    v.push_str("=");
    v.push_str(value);
    v.push_str("\n");

    // Already present ? we don't do anything
    if env.contains(&v) { return Ok(()); }

    // Not present ? we add it to the env file
    let f = Path::new("/Users/nicolasb/.zshenv");
    let mut l = OpenOptions::new()
        .append(true)
        .create(true)
        .open(f)?;
    l.write(v.as_bytes())?;
            
    Ok(())
}

/// Unsets both global and local (process) environment variable. Support for Windows. Linux support TBD.
#[cfg(target_os = "windows")]
pub fn unset_var(var: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.delete_value(var)?;
    env::remove_var(var);
    Ok(())
}

/* Run the tests in a single thread context !
$env:RUST_TEST_THREADS=1; cargo test */

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use winreg::enums::*;
    use winreg::RegKey;
    use std::env;
    #[test]
    fn is_set_globally() {
        crate::set_var("ENVTEST", "TESTVALUE").unwrap();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .unwrap();
        let var: String = key.get_value("ENVTEST").unwrap();
        assert_eq!(String::from("TESTVALUE"), var);
    }

    #[test]
    fn is_set_locally() {
        assert_eq!(String::from("TESTVALUE"), env::var("ENVTEST").unwrap());
    }

    #[test]
    #[should_panic]
    fn is_unset_globally() {
        crate::unset_var("ENVTEST").unwrap();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .unwrap();
        let _: String = key.get_value("ENVTEST").unwrap();
    }

    #[test]
    #[should_panic]
    fn is_unset_locally() {
        env::var("ENVTEST").unwrap();
    }
}

#[cfg(target_os = "macos")]
mod tests {

}
