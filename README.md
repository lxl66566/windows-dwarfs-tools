# windows-dwarfs-tools

[English](README.md) | 简体中文

A tool dedicated to making it more convenient for Windows users to use [dwarfs](https://github.com/mhx/dwarfs) for compression and decompression. It adds a right-click context menu item to the Windows system, allowing users to directly perform zstd/lzma-based dwarfs format compression, decompression, and mounting operations by right-clicking on files/folders.

windows-dwarfs-tools has built-in 64-bit dwarfs executables and a winfsp dll, allowing it to be used directly on Windows systems without the hassle of installing dependencies. Of course, if you need to mount dwarfs files, you will need to install WinFsp.

## Main Features

- **Efficient Compression**: Utilizes dwarfs for high-compression ratio archiving, balancing performance and compression ratio.
- **Right-Click Menu Integration**: Easily perform compression and decompression through the right-click context menu.
- **Mountable**: Dwarfs files can be mounted as a drive letter or a folder, making them easy to use.

## Installation

Choose one of the following:

- Download the executable file from the [Release](https://github.com/lxl66566/windows-dwarfs-tools/releases) page and place it in `C:\Windows\System32` or any directory included in your `PATH`.
- Install using [bpm](https://github.com/lxl66566/bpm): `bpm i lxl66566/windows-dwarfs-tools`.

After installation, simply click the exe to run it or run it from the command line without any arguments to add the right-click context menu item.

## Notes

- In addition to the right-click menu, this project also provides a command-line interface. Please run `windows-dwarfs-tools -h` to view the help information.
  - Uninstalling the menu requires running a command line.

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
