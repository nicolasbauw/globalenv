//! Globally set or unset environment variables (and not just for the current process).
//! Example:
//! ```rust
//! use globalenv::{set_var, unset_var};
//! set_var("ENVTEST", "TESTVALUE").unwrap();
//! unset_var("ENVTEST").unwrap();
//! ```

use std::{io, env};
#[cfg(target_os = "windows")]
use winreg::{ enums::*, RegKey };

#[cfg(target_family = "unix")]
use std::{ fs, io::prelude::*, path::PathBuf, fs::OpenOptions };

/// Sets a global environment variable, usable also in current process without reload. Support for Windows, zsh and bash.
#[cfg(target_os = "windows")]
pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    // Setting the variable globally
    key.set_value(var, &value)?;
    // Additionnaly, we set the env for current shell
    env::set_var(var, value);
    Ok(())
}

#[cfg(target_family = "unix")]
pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    // Getting env and building env file path
    let homedir = env::var("HOME").unwrap();
    let shell = env::var("SHELL").unwrap();
    let envfile = match shell.as_str() {
        "/bin/zsh" => ".zshenv",
        "/bin/bash" => ".bashrc",
        _ => "TDB"
    };

    let mut envfilepath = PathBuf::from(homedir);
    envfilepath.push(envfile);
    println!("{:?}", envfilepath);

    // Reading the env file
    let env = fs::read_to_string(&envfilepath)?;

    // Building the "export" line according to requested parameters
    let mut export = String::from("export ");
    export.push_str(var);
    export.push_str("=");
    export.push_str(value);
    export.push_str("\n");

    // Already present ? we just set the variable for current process
    if env.contains(&export) { println!("Already set in env file"); env::set_var(var, value); return Ok(()); }

    // Not present ? we append the env file to set it globally
    let mut env_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(envfilepath)?;
    env_file.write(export.as_bytes())?;
    println!("Set in env file");

    // Additionnaly, we set the env for current process
    env::set_var(var, value);
            
    Ok(())
}

/// Unsets both global and local (process) environment variable. Support for Windows, zsh and bash.
#[cfg(target_os = "windows")]
pub fn unset_var(var: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.delete_value(var)?;
    env::remove_var(var);
    Ok(())
}

#[cfg(target_family = "unix")]
pub fn unset_var(var: &str, value: &str) -> io::Result<()> {
    // Getting env and building env file path
    let homedir = env::var("HOME").unwrap();
    let shell = env::var("SHELL").unwrap();
    let envfile = match shell.as_str() {
        "/bin/zsh" => ".zshenv",
        "/bin/bash" => ".bashrc",
        _ => "TDB"
    };

    let mut envfilepath = PathBuf::from(homedir);
    envfilepath.push(envfile);
    println!("{:?}", envfilepath);

    // Reading the env file
    let env = fs::read_to_string(&envfilepath)?;

    // Building the "export" line according to requested parameters
    let mut export = String::from("export ");
    export.push_str(var);
    export.push_str("=");
    export.push_str(value);
    export.push_str("\n");

    // Variable not present in env file ? we just unset the variable for current process
    if !env.contains(&export) { println!("Not present in env file"); env::remove_var(var); return Ok(()); }

    // Present ? we remove it from the env file to unset it globally
    println!("{}", env);
    let updated_env = env.replace(&export, "\x08");
    println!("{}", updated_env);
    fs::write(envfilepath, updated_env)?;
    println!("Removed from env file");

    // Additionnaly, we unset the env for current process
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

#[cfg(target_family = "unix")]
mod tests {

}
