use std::io::Error;

use crate::config::Config;

pub fn check_system_proxy() -> std::io::Result<bool> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let internet_settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            winreg::enums::KEY_QUERY_VALUE,
        )?;
        let enabled: u32 = internet_settings.get_value("ProxyEnable")?;

        Ok(enabled != 0)
    }
}

pub fn enable_system_proxy(cfg: &Config) -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let internet_settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            winreg::enums::KEY_SET_VALUE,
        );
        match internet_settings {
            Ok(key) => {
                key.set_value("ProxyEnable", &1u32)?;
                let proxy_server = format!("{}:{}", cfg.listen_host, cfg.listen_port);
                key.set_value("ProxyServer", &proxy_server)?;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

pub fn disable_system_proxy() -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let internet_settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            winreg::enums::KEY_SET_VALUE,
        );
        match internet_settings {
            Ok(key) => {
                key.set_value("ProxyEnable", &0u32)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
