//! Configuration persistence: atomic saves, reads, legacy Electron paths.

use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Local;

use super::StoredConfig;

pub fn save(path: &Path, config: &StoredConfig) -> Result<()> {
    let parent = path.parent().context("configuration path has no parent")?;
    fs::create_dir_all(parent).context("create config directory")?;
    let data = serde_json::to_vec_pretty(config).context("serialize configuration")?;
    let temporary = path.with_extension("json.tmp");
    write_temporary_config(&temporary, &data)?;
    replace_config_file(&temporary, path)?;
    Ok(())
}

fn write_temporary_config(path: &Path, data: &[u8]) -> Result<()> {
    let mut file = File::create(path).context("create temporary configuration")?;
    file.write_all(data)
        .context("write temporary configuration")?;
    file.sync_all().context("sync temporary configuration")?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_config_file(temporary: &Path, target: &Path) -> Result<()> {
    fs::rename(temporary, target).context("replace configuration atomically")
}

#[cfg(target_os = "windows")]
fn replace_config_file(temporary: &Path, target: &Path) -> Result<()> {
    if !target.exists() {
        return fs::rename(temporary, target).context("create configuration");
    }

    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let target = wide(target);
    let temporary = wide(temporary);
    // ReplaceFileW atomically swaps the new file into place when the destination already
    // exists. `rename` alone has inconsistent replacement semantics across Windows versions.
    let result = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            temporary.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error()).context("replace configuration atomically");
    }
    Ok(())
}

pub(super) fn read_config(path: &Path) -> Result<StoredConfig> {
    let source = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&source).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn backup_legacy(path: &PathBuf) -> Result<()> {
    let stamp = Local::now().format("%Y%m%d%H%M%S");
    let backup = path.with_extension(format!("json.{stamp}.bak"));
    match fs::copy(path, backup) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("backup legacy configuration"),
    }
}

pub(super) fn legacy_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let home = env::var_os("HOME").map(PathBuf::from);

    #[cfg(target_os = "macos")]
    if let Some(home) = &home {
        paths.push(home.join("Library/Application Support/neko/neko-config.json"));
        paths.push(home.join("Library/Application Support/Neko/neko-config.json"));
        paths.push(home.join("Library/Application Support/com.neko.app/neko-config.json"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = env::var_os("APPDATA") {
            paths.push(PathBuf::from(&app_data).join("neko/neko-config.json"));
            paths.push(PathBuf::from(app_data).join("Neko/neko-config.json"));
        }
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            paths.push(PathBuf::from(local_app_data).join("neko/neko-config.json"));
        }
        if let Some(program_data) = env::var_os("PROGRAMDATA") {
            paths.push(PathBuf::from(program_data).join("com.neko.app/neko-config.json"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
            paths.push(PathBuf::from(config_home).join("neko/neko-config.json"));
        }
        if let Some(home) = home {
            paths.push(home.join(".config/neko/neko-config.json"));
            paths.push(home.join(".config/Neko/neko-config.json"));
            paths.push(home.join(".config/com.neko.app/neko-config.json"));
        }
    }

    paths
}
