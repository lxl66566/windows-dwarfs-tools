mod compress;
mod edit_reg;
mod file_dialog;
use anyhow::Result;
use assert2::assert;
use clap::{Parser, Subcommand};
use std::{ffi::OsString, io::Read, path::PathBuf};

use crate::compress::{compress_folder_to_dwarfs, decompress_dwarfs_to_folder};

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
        /// 压缩级别 (0-9, 默认 7)
        #[arg(short, long, value_parser = clap::value_parser!(i32).range(0..=9))]
        compression_level: Option<i32>,
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

struct PauseGuard;

impl Drop for PauseGuard {
    fn drop(&mut self) {
        // 这个方法会在 PauseGuard 实例离开作用域时调用
        // 无论是因为正常结束还是因为 panic
        println!("请按任意键继续");
        std::io::stdin().read_exact(&mut [0; 1]).unwrap();
    }
}

trait PathExt: Sized {
    fn add_ext(self) -> Self;
    fn rm_ext(self) -> Self;
}

impl PathExt for PathBuf {
    fn add_ext(self) -> Self {
        let mut os_string: OsString = self.into_os_string();
        os_string.push(".dwarfs");
        PathBuf::from(os_string)
    }

    fn rm_ext(self) -> Self {
        let os_string: OsString = self.into_os_string();
        let os_string_str = os_string.to_str().unwrap();
        let trimmed = os_string_str.trim_end_matches(".dwarfs");
        PathBuf::from(trimmed)
    }
}

fn main() -> Result<()> {
    let _guard = PauseGuard;
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Uninstall) => {
            edit_reg::remove_context_menu_entries()?;
        }
        Some(Commands::Compress {
            input,
            mut output,
            compression_level,
            interactive,
        }) => {
            if interactive {
                output = file_dialog::save_file_dialog(
                    &["*.dwarfs"],
                    input.clone().add_ext()
                        .file_name()
                        .expect("Internal error: Failed to get file name from path that will be compress to").to_string_lossy().as_ref(),
                );
                assert!(output.is_some(), "用户取消了文件选择操作");
            }
            compress_folder_to_dwarfs(
                input.clone(),
                output.unwrap_or_else(|| input.add_ext()),
                compression_level,
            )?;
        }
        Some(Commands::Decompress {
            input,
            mut output,
            interactive,
        }) => {
            if interactive {
                output = file_dialog::save_file_dialog(&[], input.clone().rm_ext()
                        .file_name()
                        .expect("Internal error: Failed to get file name from path that will be decompress to").to_string_lossy().as_ref(),);
                assert!(output.is_some(), "用户取消了文件选择操作");
                println!("解压到 {:?}", output.as_ref().unwrap());
            }
            decompress_dwarfs_to_folder(input.clone(), output.unwrap_or_else(|| input.rm_ext()))?;
        }
        None => {
            // 不带参数执行时，添加右键菜单项
            edit_reg::add_context_menu_entries()?;
        }
    }

    Ok(())
}
