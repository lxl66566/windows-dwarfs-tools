use std::{
    env, fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, ensure};
use once_fn::once;
use tempfile::NamedTempFile;

#[once]
pub fn temp_dir() -> PathBuf {
    let path = env::temp_dir().join(env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&path).expect("create temp dir failed");
    path
}

/// Decompress prebuilt zst bytes and atomically persist to `target`,
/// so a killed process never leaves a partially written binary behind.
fn unpack_zstd_to(compressed: &[u8], target: &Path) -> Result<()> {
    let mut tmp_file = NamedTempFile::new_in(temp_dir())?;
    let mut decoder = zstd::stream::Decoder::new(Cursor::new(compressed))?;
    io::copy(&mut decoder, &mut tmp_file)?;
    tmp_file.persist(target).map_err(|e| e.error)?;
    Ok(())
}

pub fn unpack_all() -> Result<()> {
    let path1 = temp_dir().join("dwarfs.exe");
    let path2 = temp_dir().join("winfsp-x64.dll");
    let path3 = temp_dir().join("mkdwarfs.exe");
    let path4 = temp_dir().join("dwarfsextract.exe");
    if !path1.exists() {
        unpack_zstd_to(
            include_bytes!(concat!(env!("OUT_DIR"), "/dwarfs.exe.zst")),
            &path1,
        )?;
    }
    if !path2.exists() {
        unpack_zstd_to(
            include_bytes!(concat!(env!("OUT_DIR"), "/winfsp-x64.dll.zst")),
            &path2,
        )?;
    }
    if !path3.exists() {
        fs::hard_link(&path1, path3)?;
    }
    if !path4.exists() {
        fs::hard_link(path1, path4)?;
    }
    Ok(())
}

/// 运行子进程并检查退出码，非零退出视为错误。
fn run_checked(command: &mut Command) -> Result<()> {
    let status = command.spawn()?.wait()?;
    ensure!(
        status.success(),
        "`{}` exited with {status}",
        command.get_program().to_string_lossy()
    );
    Ok(())
}

/// 压缩文件夹到 .dwarfs 文件。
pub fn compress_folder_to_dwarfs(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    compression_level: Option<i32>,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    unpack_all()?;
    ensure!(
        input_path.is_dir(),
        "Input path is not a directory: {}",
        input_path.display()
    );
    ensure!(
        !output_path.exists(),
        "Output path already exists: {}",
        output_path.display()
    );
    let mut command = Command::new(temp_dir().join("mkdwarfs.exe"));
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);
    if let Some(level) = compression_level {
        command.arg("-l").arg(level.to_string());
    }
    run_checked(&mut command)
}

/// 解压 dwarfs 文件到指定文件夹。
pub fn decompress_dwarfs_to_folder(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    println!(
        "Decompressing  {} to {}",
        input_path.display(),
        output_path.display()
    );
    unpack_all()?;
    ensure!(
        input_path.is_file(),
        "Input path is not a file: {}",
        input_path.display()
    );
    fs::create_dir_all(output_path)?;
    let mut command = Command::new(temp_dir().join("dwarfsextract.exe"));
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);
    run_checked(&mut command)
}

/// 将移入临时文件夹的文件恢复原位、并清理临时文件夹的 RAII guard。
/// 无论压缩成功还是失败，都保证输入文件不丢。
struct RestoreGuard {
    moved_to: PathBuf,
    original: PathBuf,
    temp_folder: PathBuf,
}

impl Drop for RestoreGuard {
    fn drop(&mut self) {
        if let Err(e) = fs::rename(&self.moved_to, &self.original) {
            eprintln!(
                "Failed to restore input file to {}: {e}",
                self.original.display()
            );
            return;
        }
        // 文件移走后临时文件夹应为空，用 remove_dir 避免误删用户已有目录
        if let Err(e) = fs::remove_dir(&self.temp_folder) {
            eprintln!(
                "Failed to remove temp folder {}: {e}",
                self.temp_folder.display()
            );
        }
    }
}

/// 压缩文件或文件夹到 .dwarfs 文件。
/// 如果输入是文件，会先创建一个与文件名相同的临时文件夹，将文件移动进去再压缩。
/// 压缩结束后，会将临时文件夹中的文件移动回原来的位置。
pub fn compress_path_to_dwarfs(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    compression_level: Option<i32>,
) -> Result<()> {
    let input_path_ref = input_path.as_ref();
    let output_path_ref = output_path.as_ref();
    unpack_all()?;

    if input_path_ref.is_file() {
        let file_name = input_path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("temp_file");
        let parent = input_path_ref
            .parent()
            .with_context(|| format!("can't get parent path of {}", input_path_ref.display()))?;
        let temp_folder_path = parent.join(file_name);
        ensure!(
            !temp_folder_path.exists(),
            "Temporary folder already exists, refusing to overwrite: {}",
            temp_folder_path.display()
        );
        fs::create_dir(&temp_folder_path)?;
        let dest_path = temp_folder_path.join(
            input_path_ref
                .file_name()
                .with_context(|| format!("file name is empty: {}", input_path_ref.display()))?,
        );
        fs::rename(input_path_ref, &dest_path)?;
        // 无论压缩成功与否，guard 都会把文件移回原位并清理临时文件夹
        let _guard = RestoreGuard {
            moved_to: dest_path,
            original: input_path_ref.to_path_buf(),
            temp_folder: temp_folder_path.clone(),
        };
        compress_folder_to_dwarfs(&temp_folder_path, output_path_ref, compression_level)?;
    } else if input_path_ref.is_dir() {
        compress_folder_to_dwarfs(input_path_ref, output_path_ref, compression_level)?;
    } else if input_path_ref.exists() {
        anyhow::bail!("Unsupported input path type: {}", input_path_ref.display());
    } else {
        anyhow::bail!("Input path does not exist: {}", input_path_ref.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_checked_succeeds_on_zero_exit() {
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "exit 0"]);
        assert!(run_checked(&mut cmd).is_ok());
    }

    #[test]
    fn run_checked_fails_on_nonzero_exit() {
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "exit 3"]);
        assert!(run_checked(&mut cmd).is_err());
    }

    #[test]
    fn run_checked_fails_on_missing_program() {
        let mut cmd = Command::new("definitely-not-existing-program.exe");
        assert!(run_checked(&mut cmd).is_err());
    }
}
