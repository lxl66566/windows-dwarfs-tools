use anyhow::Result;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use tar::{Archive, Builder};
use zstd::stream::raw::CParameter;
use zstd::stream::{Decoder, Encoder};

/// 压缩文件夹到 .tar.zst 文件。
///
/// # 参数
/// - `folder_path`: 要压缩的文件夹路径。
/// - `output_path`: 输出的 .tar.zst 文件路径。
/// - `compression_level`: Zstd 压缩级别 (1-22)。
pub fn compress_folder_to_tar_zst<P: AsRef<Path>>(
    folder_path: P,
    output_path: P,
    compression_level: i32,
    threads: Option<u32>,
) -> Result<()> {
    let output_file = File::create(&output_path)?;
    let mut encoder = Encoder::new(output_file, compression_level)?;

    if let Some(threads) = threads {
        encoder.set_parameter(CParameter::NbWorkers(threads))?;
    } else if let Ok(num_threads) = std::thread::available_parallelism().map(|n| n.get() as u32) {
        encoder.set_parameter(CParameter::NbWorkers(num_threads))?;
    }

    {
        let mut tar_builder = Builder::new(&mut encoder);
        // append_dir_all 的第一个参数是 tar 内部的路径前缀，这里设置为 "." 表示根目录
        tar_builder.append_dir_all(".", &folder_path)?;
        tar_builder.finish()?;
    }

    encoder.finish()?;
    println!(
        "文件夹 {:?} 已成功压缩到 {:?}",
        folder_path.as_ref(),
        output_path.as_ref()
    );
    Ok(())
}

/// 压缩文件到 .zst 文件。
///
/// # 参数
/// - `file_path`: 要压缩的文件路径。
/// - `output_path`: 输出的 .zst 文件路径。
/// - `compression_level`: Zstd 压缩级别 (1-22)。
pub fn compress_file_to_zst<P: AsRef<Path>>(
    file_path: P,
    output_path: P,
    compression_level: i32,
    threads: Option<u32>,
) -> Result<()> {
    let input_file = File::open(&file_path)?;
    let output_file = File::create(&output_path)?;

    let mut encoder = Encoder::new(output_file, compression_level)?;

    if let Some(threads) = threads {
        encoder.set_parameter(CParameter::NbWorkers(threads))?;
    } else if let Ok(num_threads) = std::thread::available_parallelism().map(|n| n.get() as u32) {
        encoder.set_parameter(CParameter::NbWorkers(num_threads))?;
    }

    let mut reader = BufReader::new(input_file);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;

    println!(
        "文件 {:?} 已成功压缩到 {:?}",
        file_path.as_ref(),
        output_path.as_ref()
    );
    Ok(())
}

/// 解压 .tar.zst 文件到指定文件夹。
///
/// # 参数
/// - `input_path`: 输入的 .tar.zst 文件路径。
/// - `output_folder`: 解压到的目标文件夹路径。
pub fn decompress_tar_zst_to_folder<P: AsRef<Path>>(input_path: P, output_folder: P) -> Result<()> {
    let input_file = File::open(&input_path)?;
    let decoder = Decoder::new(input_file)?;
    let mut archive = Archive::new(decoder);

    // 确保目标文件夹存在
    fs::create_dir_all(&output_folder)?;

    // 解压到指定目录
    archive.unpack(&output_folder)?;

    println!(
        "文件 {:?} 已成功解压到文件夹 {:?}",
        input_path.as_ref(),
        output_folder.as_ref()
    );
    Ok(())
}

/// 解压 .zst 文件到指定文件。
///
/// # 参数
/// - `input_path`: 输入的 .zst 文件路径。
/// - `output_path`: 解压到的目标文件路径。
pub fn decompress_zst_to_file<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let input_file = File::open(&input_path)?;
    let mut decoder = Decoder::new(input_file)?;
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);

    io::copy(&mut decoder, &mut writer)?;

    println!(
        "文件 {:?} 已成功解压到 {:?}",
        input_path.as_ref(),
        output_path.as_ref()
    );
    Ok(())
}
