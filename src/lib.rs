use std::{process::Command, io};

pub fn set_var(var: &str, value: &str) -> io::Result<()> {
    Command::new("cmd")
        .args(&["/C", "setx", var, value])
        .output()?;
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
        let key = hkcu
            .open_subkey_with_flags("Environment", KEY_SET_VALUE)
            .unwrap();
        key.delete_value("ENVTEST").unwrap();
    }
}
