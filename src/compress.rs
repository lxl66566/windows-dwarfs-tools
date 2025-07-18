use anyhow::Result;
use assert2::assert;
use once_fn::once;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[once]
pub fn temp_dir() -> PathBuf {
    let path = env::temp_dir().join(env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&path).expect("create temp dir failed");
    path
}

/// decompress the prebuilt zst file and write to a temp file.
macro_rules! write_prebuilt_zstd {
    ($zst_filename:expr, $output_path:expr) => {{
        let compressed_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/", $zst_filename));
        let file = std::fs::File::create(&$output_path).expect("create temp file failed");
        let mut decoder = zstd::stream::Decoder::new(std::io::Cursor::new(compressed_bytes))
            .expect("zstd decoder create failed");
        let mut writer = std::io::BufWriter::new(file);
        std::io::copy(&mut decoder, &mut writer).map(|_| $output_path)
    }};
}

pub fn unpack_all() -> Result<()> {
    let path1 = temp_dir().join("dwarfs.exe");
    let path2 = temp_dir().join("winfsp-x64.dll");
    let path3 = temp_dir().join("mkdwarfs.exe");
    let path4 = temp_dir().join("dwarfsextract.exe");
    if !path1.exists() {
        write_prebuilt_zstd!("dwarfs.exe.zst", &path1)?;
    }
    if !path2.exists() {
        write_prebuilt_zstd!("winfsp-x64.dll.zst", path2)?;
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
pub fn compress_folder_to_dwarfs<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
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
pub fn decompress_dwarfs_to_folder<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    unpack_all()?;
    assert!(input_path.is_file());
    fs::create_dir_all(output_path)?;
    let mut command = Command::new(temp_dir().join("dwarfsextract.exe"));
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);
    command.spawn()?.wait()?;
    Ok(())
}
