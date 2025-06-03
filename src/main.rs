mod compress;
mod edit_reg;
mod file_dialog;
use anyhow::{Result, anyhow};
use assert2::assert;
use clap::{Parser, Subcommand};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    process::Command,
};

use crate::compress::{decompress_tar_zst_to_folder, decompress_zst_to_file};

const DEFAULT_COMPRESSION_LEVEL: i32 = 22; // Zstd 最高压缩级别

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 卸载右键菜单项
    Uninstall,
    /// 压缩文件或文件夹
    #[command(visible_alias = "c")]
    Compress {
        /// 输入文件或文件夹路径
        input: PathBuf,
        /// 输出文件路径 (可选)。如果未提供，则自动生成。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 压缩级别 (1-22, 默认 19)
        #[arg(short, long, default_value_t = DEFAULT_COMPRESSION_LEVEL)]
        compression_level: i32,
        /// 使用的线程数 (默认使用所有可用核心)
        #[arg(short, long)]
        threads: Option<u32>,
        /// 交互式选择文件/文件夹将被压缩到的位置
        #[arg(short, long)]
        interactive: bool,
    },
    /// 解压文件或文件夹
    #[command(visible_alias = "d")]
    Decompress {
        /// 输入文件路径
        input: PathBuf,
        /// 输出文件或文件夹路径 (可选)。如果未提供，则自动生成。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 交互式选择解压文件将要保存到的位置
        #[arg(short, long)]
        interactive: bool,
    },
}

impl Cli {
    fn verify(&self) {
        if let Some(Commands::Compress {
            input,
            compression_level,
            ..
        }) = &self.command
        {
            assert!(input.is_dir() || input.is_file());
            assert!(
                *compression_level > 0 && *compression_level <= 22,
                "压缩级别必须在 1-22 之间"
            );
        }
    }
}

struct PauseGuard;

impl Drop for PauseGuard {
    fn drop(&mut self) {
        // 这个方法会在 PauseGuard 实例离开作用域时调用
        // 无论是因为正常结束还是因为 panic
        pause();
    }
}

fn main() -> Result<()> {
    let _guard = PauseGuard;
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    cli.verify();

    match cli.command {
        Some(Commands::Uninstall) => {
            edit_reg::remove_context_menu_entries()?;
        }
        Some(Commands::Compress {
            input,
            output,
            compression_level,
            threads,
            interactive,
        }) => {
            if interactive {
                let output = file_dialog::save_file_dialog(
                    &["*.tar.zst", "*.zst"],
                    path_compress_to(input.clone())
                        .file_name()
                        .expect("Internal error: Failed to get file name from path that will be compress to").to_string_lossy().as_ref(),
                );
                assert!(output.is_some(), "用户取消了文件选择操作");
                handle_compress(input, output, compression_level, threads)?;
            } else {
                handle_compress(input, output, compression_level, threads)?;
            }
        }
        Some(Commands::Decompress {
            input,
            output,
            interactive,
        }) => {
            if interactive {
                let output = file_dialog::save_file_dialog(&["*.*"], path_decompress_to(input.clone()).0
                        .file_name()
                        .expect("Internal error: Failed to get file name from path that will be decompress to").to_string_lossy().as_ref(),);
                assert!(output.is_some(), "用户取消了文件选择操作");
                handle_decompress(input, output)?;
            } else {
                handle_decompress(input, output)?;
            }
        }
        None => {
            // 不带参数执行时，添加右键菜单项
            edit_reg::add_context_menu_entries()?;
        }
    }

    Ok(())
}

fn handle_compress(
    input: PathBuf,
    output: Option<PathBuf>,
    compression_level: i32,
    threads: Option<u32>,
) -> Result<()> {
    let input_path = input.as_path();
    let is_dir = input_path.is_dir();
    let final_output_path = output.unwrap_or_else(|| path_compress_to(input_path.to_path_buf()));

    if is_dir {
        compress::compress_folder_to_tar_zst(
            input_path,
            final_output_path.as_ref(),
            compression_level,
            threads,
        )?;
    } else if input_path.is_file() {
        compress::compress_file_to_zst(
            input_path,
            final_output_path.as_ref(),
            compression_level,
            threads,
        )?;
    } else {
        return Err(anyhow!(
            "输入路径 {:?} 既不是文件也不是文件夹。",
            input_path
        ));
    }

    Ok(())
}

fn handle_decompress(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let input_path = input.as_path();
    assert!(input_path.is_file(), "输入路径 {:?} 不是文件", input_path);
    assert!(
        input_path.extension() == Some(OsStr::new("zst")),
        "输入路径 {:?} 不是 .zst 文件",
        input_path
    );
    let (output_path, is_dir) = path_decompress_to(input_path.to_path_buf());

    if is_dir {
        decompress_tar_zst_to_folder(input_path, output.unwrap_or(output_path).as_path())?;
    } else {
        decompress_zst_to_file(input_path, output.unwrap_or(output_path).as_path())?;
    }
    Ok(())
}

/// 根据输入路径是目录还是文件，在其末尾直接追加相应的压缩后缀。该函数会访问文件系统以获取文件类型。
///
/// - 如果 `input_path` 是一个目录 (directory)，则在其末尾添加 ".tar.zst"。
/// - 否则 (例如文件、不存在的路径等)，在其末尾添加 ".zst"。
///
/// 注意：此函数直接操作路径的 OsString 表示，不使用 with_extension 等方法。
fn path_compress_to(input_path: PathBuf) -> PathBuf {
    // 首先检查路径是否存在以及是否为目录
    // is_dir() 会查询文件系统
    let is_directory = input_path.is_dir();

    // 将 PathBuf 转换为 OsString 以便进行字符串操作
    // input_path 在这里被消耗 (moved)
    let mut os_string: OsString = input_path.into_os_string();

    if is_directory {
        os_string.push(".tar.zst"); // OsString::push 可以接受 &str, 它会内部转换为 OsStr
    } else {
        os_string.push(".zst");
    }

    // 将修改后的 OsString 转换回 PathBuf
    PathBuf::from(os_string)
}

/// 将输入路径转换为解压后的路径。该函数不会访问文件系统。
///
/// # Returns
///
/// - `PathBuf`: 解压后的路径。
/// - `bool`: 是否是 .tar.zst 文件。
fn path_decompress_to(input_path: PathBuf) -> (PathBuf, bool) {
    let os_string: OsString = input_path.into_os_string();
    let os_string_str = os_string.to_str().unwrap();
    let mut ty = false;
    let trimmed = os_string_str.trim_end_matches(".tar.zst");
    let new_path = if trimmed.len() == os_string_str.len() - 8 {
        ty = true;
        trimmed
    } else {
        os_string_str.trim_end_matches(".zst")
    };

    let output_path = PathBuf::from(new_path);
    (output_path, ty)
}

fn pause() {
    Command::new("cmd")
        .arg("/k")
        .arg("pause")
        .spawn()
        .expect("无法暂停命令行")
        .wait()
        .expect("等待命令行暂停失败");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_path_decompress_to() {
        let tempdir = tempdir().expect("Failed to create temp dir");
        let input_path = tempdir.path().join("my_test_file.txt.zst");
        let output_path = path_decompress_to(input_path.clone());
        assert_eq!(
            output_path,
            (tempdir.path().join("my_test_file.txt"), false)
        );

        let input_path = tempdir.path().join("my_test_dir.tar.zst");
        let output_path = path_decompress_to(input_path.clone());

        assert_eq!(output_path, (tempdir.path().join("my_test_dir"), true));
    }
}
