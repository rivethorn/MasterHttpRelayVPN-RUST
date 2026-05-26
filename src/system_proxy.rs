use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;

use crate::config::Config;

const LINUX_SCRIPT: &str = include_str!("../scripts/proxy_set_linux_sh");
const MACOS_SCRIPT: &str = include_str!("../scripts/proxy_set_osx_sh");

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
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

    #[cfg(target_os = "macos")]
    {
        let socks = cfg.socks5_port.unwrap_or(cfg.listen_port);
        run_script(
            MACOS_SCRIPT,
            &[
                "set",
                &cfg.listen_host,
                &cfg.listen_port.to_string(),
                &socks.to_string(),
            ],
        )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to enable system proxy: {}", e),
            )
        })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let socks = cfg.socks5_port.unwrap_or(cfg.listen_port);
        run_script(
            LINUX_SCRIPT,
            &[
                "manual",
                &cfg.listen_host,
                &cfg.listen_port.to_string(),
                &socks.to_string(),
            ],
        )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to enable system proxy: {}", e),
            )
        })
    }
}

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
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

    #[cfg(target_os = "macos")]
    {
        run_script(MACOS_SCRIPT, &["clear"]).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to disable system proxy: {}", e),
            )
        })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        run_script(LINUX_SCRIPT, &["none"]).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to enable system proxy: {}", e),
            )
        })
    }
}

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
fn run_script(script_content: &str, args: &[&str]) -> std::io::Result<()> {
    let script_path = std::env::temp_dir().join("proxy_script.sh");
    fs::write(&script_path, script_content)?;

    let output = Command::new("/bin/bash")
        .arg(&script_path)
        .args(args)
        .output()?;

    if !output.status.success() {
        eprintln!("Script error: {}", String::from_utf8_lossy(&output.stderr));
        let _ = fs::remove_file(script_path);
        return Err(Error::new(ErrorKind::Other, "Failed to run proxy script"));
    }

    let _ = fs::remove_file(script_path);
    Ok(())
}
