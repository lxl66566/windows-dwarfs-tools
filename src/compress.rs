use std::{
    env, fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use assert2::assert;
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
        unpack_zstd_to(include_bytes!(concat!(env!("OUT_DIR"), "/dwarfs.exe.zst")), &path1)?;
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

/// 压缩文件夹到 .dwarfs 文件。
pub fn compress_folder_to_dwarfs(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    compression_level: Option<i32>,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    unpack_all()?;
    assert!(input_path.is_dir());
    assert!(!output_path.exists());
    let mut command = Command::new(temp_dir().join("mkdwarfs.exe"));
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);
    if let Some(level) = compression_level {
        command.arg("-l").arg(level.to_string());
    }
    command.spawn()?.wait()?;
    Ok(())
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
    assert!(input_path.is_file());
    fs::create_dir_all(output_path)?;
    let mut command = Command::new(temp_dir().join("dwarfsextract.exe"));
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);
    command.spawn()?.wait()?;
    Ok(())
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
    assert!(input_path_ref.is_file() || input_path_ref.is_dir());

    if input_path_ref.is_file() {
        let file_name = input_path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("temp_file");
        let temp_folder_path = input_path_ref
            .parent()
            .unwrap_or_else(|| panic!("can't get parent path of {}", input_path_ref.display()))
            .join(file_name);
        fs::create_dir_all(&temp_folder_path)?;
        let dest_path =
            temp_folder_path.join(input_path_ref.file_name().expect("file name is empty"));
        fs::rename(input_path_ref, &dest_path)?;
        // dbg!(&input_path_ref, &temp_folder_path, &dest_path);
        compress_folder_to_dwarfs(
            temp_folder_path.clone(),
            output_path_ref.to_path_buf(),
            compression_level,
        )?;
        fs::rename(dest_path, input_path_ref)?;
        fs::remove_dir_all(temp_folder_path)?; // 清理临时文件夹
    } else if input_path_ref.is_dir() {
        compress_folder_to_dwarfs(
            input_path_ref.to_path_buf(),
            output_path_ref.to_path_buf(),
            compression_level,
        )?;
    } else if input_path_ref.exists() {
        panic!("Unsupported input path type: {}", input_path_ref.display());
    } else {
        panic!("Input path does not exist: {}", input_path_ref.display());
    }
    Ok(())
}
