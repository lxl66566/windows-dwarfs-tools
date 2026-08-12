use std::process::Command;

use anyhow::{Context, Result, bail};
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

use crate::compress::{temp_dir, unpack_all};

/// 从盘符位掩码中选出 Z→A 方向首个未使用的盘符。
fn first_unused_from_mask(drives_mask: u32) -> Option<char> {
    ('A'..='Z')
        .rev()
        .find(|&c| drives_mask & (1 << (u32::from(c) - u32::from('A'))) == 0)
}

/// 获取从 Z: 到 A: 的首个未使用的盘符。
///
/// # Returns
///
/// 如果找到未使用的盘符，则返回一个包含该盘符的 `Some(String)`，例如 "Z:"。
/// 如果所有盘符都已被使用，则返回 `None`。
pub fn get_first_unused_drive_letter() -> Option<String> {
    let drives_mask = unsafe { GetLogicalDrives() };
    first_unused_from_mask(drives_mask).map(|c| format!("{c}:"))
}

/// 挂载 dwarfs 文件为盘符或文件夹。
pub fn mount_dwarfs(input: std::path::PathBuf, dest: Option<String>) -> Result<()> {
    unpack_all()?;
    let dest = dest
        .or_else(get_first_unused_drive_letter)
        .context("No available drive letter")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_z_when_nothing_used() {
        assert!(first_unused_from_mask(0) == Some('Z'));
    }

    #[test]
    fn skips_used_letters_from_z_to_a() {
        // Z 和 Y 被占用时应选 X
        let mask =
            (1 << (u32::from('Z') - u32::from('A'))) | (1 << (u32::from('Y') - u32::from('A')));
        assert!(first_unused_from_mask(mask) == Some('X'));
    }

    #[test]
    fn returns_none_when_all_letters_used() {
        assert!(first_unused_from_mask(0x03ff_ffff).is_none());
    }
}
