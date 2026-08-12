mod compress;
mod edit_reg;
mod file_dialog;
mod mount;
use std::{io::Read, path::{Path, PathBuf}};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::compress::{compress_path_to_dwarfs, decompress_dwarfs_to_folder};

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
        /// Output file or folder path (optional). If not provided, it will be generated
        /// automatically.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Interactively select where the decompressed file will be saved
        #[arg(short, long)]
        interactive: bool,
    },
    /// Mount dwarfs file as drive or folder
    #[command(visible_alias = "m")]
    Mount {
        /// Input file path
        input: PathBuf,
        /// Output drive letter (ends with ':') or folder path (optional). If not provided, it will
        /// be a usable drive letter.
        dest: Option<String>,
    },
}

struct PauseGuard;

impl Drop for PauseGuard {
    fn drop(&mut self) {
        // This method is called when the PauseGuard instance goes out of scope
        // whether due to normal completion or a panic
        println!("Press any key to continue...");
        // stdin may already be closed; never panic in a destructor
        let _ = std::io::stdin().read_exact(&mut [0; 1]);
    }
}

trait PathExt {
    fn add_ext(&self) -> PathBuf;
    fn rm_ext(&self) -> PathBuf;
}

impl PathExt for Path {
    fn add_ext(&self) -> PathBuf {
        let mut os_string = self.as_os_str().to_os_string();
        os_string.push(".dwarfs");
        PathBuf::from(os_string)
    }

    fn rm_ext(&self) -> PathBuf {
        if self.extension().is_some_and(|ext| ext == "dwarfs") {
            self.with_extension("")
        } else {
            self.to_path_buf()
        }
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
            edit_reg::remove_context_menu_entries();
        },
        Some(Commands::Compress {
            input,
            mut output,
            compression_level,
            interactive,
        }) => {
            if interactive {
                let default_output = input.add_ext();
                let Some(selected) = file_dialog::save_file_dialog(
                    &["*.dwarfs"],
                    default_output
                        .file_name()
                        .expect(
                            "Internal error: Failed to get file name from path that will be \
                             compress to",
                        )
                        .to_string_lossy()
                        .as_ref(),
                ) else {
                    println!("Operation cancelled by user");
                    return Ok(());
                };
                output = Some(selected);
            }
            compress_path_to_dwarfs(
                &input,
                output.unwrap_or_else(|| input.add_ext()),
                compression_level,
            )?;
        },
        Some(Commands::Decompress {
            input,
            mut output,
            interactive,
        }) => {
            if interactive {
                let default_output = input.rm_ext();
                let Some(selected) = file_dialog::save_file_dialog(
                    &[],
                    default_output
                        .file_name()
                        .expect(
                            "Internal error: Failed to get file name from path that will be \
                             decompress to",
                        )
                        .to_string_lossy()
                        .as_ref(),
                ) else {
                    println!("Operation cancelled by user");
                    return Ok(());
                };
                output = Some(selected);
            }
            decompress_dwarfs_to_folder(&input, output.unwrap_or_else(|| input.rm_ext()))?;
        },
        None => {
            // When executed without arguments, add context menu entries
            edit_reg::add_context_menu_entries()?;
        },
        Some(Commands::Mount { input, dest }) => {
            mount::mount_dwarfs(input, dest)?;
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rm_ext_strips_exactly_one_dwarfs_suffix() {
        assert2::assert!(Path::new("a.dwarfs").rm_ext() == Path::new("a"));
        assert2::assert!(Path::new("a.dwarfs.dwarfs").rm_ext() == Path::new("a.dwarfs"));
        assert2::assert!(Path::new("dir/b.tar.dwarfs").rm_ext() == Path::new("dir/b.tar"));
    }

    #[test]
    fn rm_ext_keeps_path_without_dwarfs_extension() {
        assert2::assert!(Path::new("folder").rm_ext() == Path::new("folder"));
        // 扩展名不是严格的 "dwarfs" 时不应剥离
        assert2::assert!(Path::new("my.dwarfsfolder").rm_ext() == Path::new("my.dwarfsfolder"));
        assert2::assert!(Path::new("my.DWARFS").rm_ext() == Path::new("my.DWARFS"));
    }

    #[test]
    fn add_ext_appends_dwarfs_suffix() {
        assert2::assert!(Path::new("a").add_ext() == Path::new("a.dwarfs"));
        assert2::assert!(Path::new("a.tar").add_ext() == Path::new("a.tar.dwarfs"));
    }
}
