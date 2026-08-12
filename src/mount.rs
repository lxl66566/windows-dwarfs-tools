use std::process::Command;

use anyhow::{Result, bail};
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

use crate::compress::{temp_dir, unpack_all};

/// 获取从 Z: 到 A: 的首个未使用的盘符。
///
/// # Returns
///
/// 如果找到未使用的盘符，则返回一个包含该盘符的 `Some(String)`，例如 "Z:"。
/// 如果所有盘符都已被使用，则返回 `None`。
pub fn get_first_unused_drive_letter() -> Option<String> {
    let drives_mask = unsafe { GetLogicalDrives() };

    for drive_char in ('A'..='Z').rev() {
        let drive_bit = 1 << (drive_char as u32 - 'A' as u32);
        if (drives_mask & drive_bit) == 0 {
            return Some(format!("{drive_char}:"));
        }
    }

    None
}

/// 挂载 dwarfs 文件为盘符或文件夹。
pub fn mount_dwarfs(input: std::path::PathBuf, dest: Option<String>) -> Result<()> {
    unpack_all()?;
    let dest =
        dest.unwrap_or_else(|| get_first_unused_drive_letter().expect("No available drive letter"));
    println!("Mount {} to `{dest}`", input.display());
    let mut cmd = Command::new(temp_dir().join("dwarfs.exe"));
    let output = cmd.arg(input).arg(dest).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to mount dwarfs file: {stderr}");
        if stderr.contains("FSD not found") {
            eprintln!(
                "Mounting dwarfs needs WinFsp to be installed. Please install it first: https://github.com/winfsp/winfsp/releases"
            );
        }
        bail!("dwarfs exited with {}", output.status);
    }
    Ok(())
}
