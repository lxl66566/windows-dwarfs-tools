# windows-dwarfs-tools

English | [简体中文](README_zh-CN.md)

A tool dedicated to making it easier for Windows users to compress and decompress files using [dwarfs](https://github.com/mhx/dwarfs). It adds right-click menu options to Windows, allowing users to directly perform zstd/lzma based dwarfs compression and decompression operations by right-clicking on files or folders.

windows-dwarfs-tools comes with a built-in 64-bit dwarfs executable and winfsp dll, making it ready to use on Windows systems without the hassle of installing dependencies.

## Key Features

- **Efficient Compression**: Utilizes dwarfs for high-compression archiving, balancing performance and compression ratio.
- **Right-Click Menu Integration**: Easily compress and decompress via the right-click context menu.

## Installation

Choose one of the following:

- Download the executable from [Release](https://github.com/lxl66566/windows-dwarfs-tools/releases) and place it in `C:\Windows\System32` or any directory included in your `PATH`.
- Install using [bpm](https://github.com/lxl66566/bpm): `bpm i lxl66566/windows-dwarfs-tools`.

After installation, simply run the exe or run it from the command line without arguments to add the right-click menu entries.

## Tips

- dwarfs can only compress folders, not individual files.
- In addition to the right-click menu, this project also provides a command-line interface. Please run `windows-dwarfs-tools -h` for help information.
  - Uninstalling the menu requires running the command line.

For explanations of compression levels, please refer to the dwarfs documentation:

| Level | Block Size | Compression Algorithm (Block Data) | Compression Algorithm (Schema/History) | Compression Algorithm (Metadata) | Window Size/Step | Inode Order |
| ----- | ---------- | ---------------------------------- | -------------------------------------- | -------------------------------- | ---------------- | ----------- |
| 0     | 20         | null                               | null                                   | null                             | 0 / 0            | none        |
| 1     | 20         | lz4                                | zstd:level=16                          | null                             | 0 / 0            | path        |
| 2     | 20         | lz4hc:level=9                      | zstd:level=16                          | null                             | 0 / 0            | path        |
| 3     | 21         | lz4hc:level=9                      | zstd:level=16                          | null                             | 12 / 1           | similarity  |
| 4     | 22         | zstd:level=11                      | zstd:level=16                          | null                             | 12 / 2           | similarity  |
| 5     | 23         | zstd:level=19                      | zstd:level=16                          | null                             | 12 / 2           | similarity  |
| 6     | 24         | zstd:level=22                      | zstd:level=16                          | null                             | 12 / 3           | nilsimsa    |
| 7     | 24         | zstd:level=22                      | zstd:level=16                          | zstd:level=22                    | 12 / 3           | nilsimsa    |
| 8     | 24         | lzma:level=9                       | zstd:level=16                          | lzma:level=9                     | 12 / 4           | nilsimsa    |
| 9     | 26         | lzma:level=9                       | zstd:level=16                          | lzma:level=9                     | 12 / 4           | nilsimsa    |
