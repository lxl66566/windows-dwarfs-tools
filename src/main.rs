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
    /// Uninstall context menu entries
    Uninstall,
    /// Compress file or folder
    #[command(visible_alias = "c")]
    Compress {
        /// Input file or folder path
        input: PathBuf,
        /// Output file path (optional). If not provided, it will be generated automatically.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Compression level (0-9, default 7)
        #[arg(short, long, value_parser = clap::value_parser!(i32).range(0..=9))]
        compression_level: Option<i32>,
        /// Interactively select where the file/folder will be compressed to
        #[arg(short, long)]
        interactive: bool,
    },
    /// Decompress file or folder
    #[command(visible_alias = "d")]
    Decompress {
        /// Input file path
        input: PathBuf,
        /// Output file or folder path (optional). If not provided, it will be generated automatically.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Interactively select where the decompressed file will be saved
        #[arg(short, long)]
        interactive: bool,
    },
}

struct PauseGuard;

impl Drop for PauseGuard {
    fn drop(&mut self) {
        // This method is called when the PauseGuard instance goes out of scope
        // whether due to normal completion or a panic
        println!("Press any key to continue...");
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
                assert!(output.is_some(), "User cancelled file selection operation");
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
                assert!(output.is_some(), "User cancelled file selection operation");
                println!("Decompressing to {:?}", output.as_ref().unwrap());
            }
            decompress_dwarfs_to_folder(input.clone(), output.unwrap_or_else(|| input.rm_ext()))?;
        }
        None => {
            // When executed without arguments, add context menu entries
            edit_reg::add_context_menu_entries()?;
        }
    }

    Ok(())
}
