//! Globally set or unset environment variables (and not just for the current process).
//! Support for Windows, zsh and bash (MacOS and most Linux distros).
//! Example:
//! ```rust
//! use globalenv::{set_var, unset_var};
//! set_var("ENVTEST", "TESTVALUE").unwrap();
//! unset_var("ENVTEST").unwrap();
//! ```

use std::{env, fmt, error};
#[cfg(target_os = "windows")]
use winreg::{ enums::*, RegKey };

#[cfg(target_family = "unix")]
use std::{ fs, io::prelude::*, path::PathBuf, fs::OpenOptions };

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnvError {
    /// Unsupported shell
    UnsupportedShell,
    /// IO Error (file or registry operation)
    IOError,
    /// ENV error (can't get or set variable)
    VarError
}

impl error::Error for EnvError {}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ENV operation error : ")?;
        f.write_str(match self {
            EnvError::UnsupportedShell => "Unsupported shell",
            EnvError::IOError => "I/O error",
            EnvError::VarError => "error while getting or setting env",
        })
    }
}

impl From<std::io::Error> for EnvError {
    fn from(_e: std::io::Error) -> EnvError {
        EnvError::IOError
    }
}

impl From<std::env::VarError> for EnvError {
    fn from(_e: std::env::VarError) -> EnvError {
        EnvError::VarError
    }
}

#[cfg(target_os = "windows")]
/// Sets a global environment variable, usable also in current process without reload.
pub fn set_var(var: &str, value: &str) -> Result<(), EnvError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    // Setting the variable globally
    key.set_value(var, &value)?;
    // Additionnaly, we set the env for current shell
    env::set_var(var, value);
    Ok(())
}

#[cfg(target_family = "unix")]
/// Sets a global environment variable, usable also in current process without reload.
pub fn set_var(var: &str, value: &str) -> Result<(), EnvError> {
    // Getting env and building env file path
    let homedir = env::var("HOME")?;
    let shell = env::var("SHELL")?;
    let envfile = match shell.as_str() {
        "/bin/zsh" => ".zshenv",
        "/bin/bash" => ".bashrc",
        _ => return Err(EnvError::UnsupportedShell)
    };

    let mut envfilepath = PathBuf::from(homedir);
    envfilepath.push(envfile);

    // Reading the env file
    let env = fs::read_to_string(&envfilepath)?;

    // Building the "export" line according to requested parameters
    let mut export = String::from("export ");
    export.push_str(var);
    export.push_str("=");
    export.push_str(value);
    export.push_str("\n");

    // Already present ? we just set the variable for current process
    if env.contains(&export) { env::set_var(var, value); return Ok(()); }

    // Not present ? we append the env file to set it globally
    let mut env_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(envfilepath)?;
    env_file.write(export.as_bytes())?;

    // Additionnaly, we set the env for current process
    env::set_var(var, value);
            
    Ok(())
}

#[cfg(target_os = "windows")]
/// Unsets both global and local (process) environment variable.
pub fn unset_var(var: &str) -> Result<(), EnvError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.delete_value(var)?;
    env::remove_var(var);
    Ok(())
}

#[cfg(target_family = "unix")]
/// Unsets both global and local (process) environment variable.
pub fn unset_var(var: &str) -> Result<(), EnvError> {
    // Getting env and building env file path
    let homedir = env::var("HOME")?;
    let shell = env::var("SHELL")?;
    let envfile = match shell.as_str() {
        "/bin/zsh" => ".zshenv",
        "/bin/bash" => ".bashrc",
        _ => return Err(EnvError::UnsupportedShell)
    };

    let mut envfilepath = PathBuf::from(homedir);
    envfilepath.push(envfile);

    // Reading the env file
    let env = fs::read_to_string(&envfilepath)?;

    // Building the "export" line according to requested parameters
    let mut export = String::from("export ");
    export.push_str(var);
    export.push_str("=");

    // Variable not present in env file ? we just unset the variable for current process
    if !env.contains(&export) { env::remove_var(var); return Ok(()); }

    // Present ? we remove it from the env file to unset it globally
    let mut updated_env = String::new();
    for l in env.lines() { if !l.contains(var) { updated_env.push_str(l); updated_env.push_str("\n") } }
    fs::write(envfilepath, updated_env)?;

    // Additionnaly, we unset the env for current process
    env::remove_var(var);
            
    Ok(())
}

/* Run the tests in a single thread context !
$env:RUST_TEST_THREADS=1; cargo test
RUST_TEST_THREADS=1 cargo test */

#[cfg(target_os = "windows")]
#[cfg(test)]
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

#[cfg(target_family = "unix")]
mod tests {
    #[test]
    fn is_set_globally() {
        crate::set_var("ENVTEST", "TESTVALUE").unwrap();
        // Getting env and building env file path
        let homedir = crate::env::var("HOME").unwrap();
        let shell = crate::env::var("SHELL").unwrap();
        let envfile = match shell.as_str() {
            "/bin/zsh" => ".zshenv",
            "/bin/bash" => ".bashrc",
            _ => panic!("Unsupported shell")
        };

        let mut envfilepath = crate::PathBuf::from(homedir);
        envfilepath.push(envfile);

        // Reading the env file
        let env = crate::fs::read_to_string(&envfilepath).unwrap();

        assert_eq!(env.contains("export ENVTEST=TESTVALUE\n"), true);
    }

    #[test]
    fn is_set_locally() {
        assert_eq!(String::from("TESTVALUE"), crate::env::var("ENVTEST").unwrap());
    }

    #[test]
    fn is_unset_globally() {
        crate::unset_var("ENVTEST").unwrap();
        // Getting env and building env file path
        let homedir = crate::env::var("HOME").unwrap();
        let shell = crate::env::var("SHELL").unwrap();
        let envfile = match shell.as_str() {
            "/bin/zsh" => ".zshenv",
            "/bin/bash" => ".bashrc",
            _ => panic!("Unsupported shell")
        };

        let mut envfilepath = crate::PathBuf::from(homedir);
        envfilepath.push(envfile);

        // Reading the env file
        let env = crate::fs::read_to_string(&envfilepath).unwrap();

        assert_eq!(env.contains("export ENVTEST=TESTVALUE\n"), false);
    }

    #[test]
    #[should_panic]
    fn is_unset_locally() {
        crate::env::var("ENVTEST").unwrap();
    }
}

    
