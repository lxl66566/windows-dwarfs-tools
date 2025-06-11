# windows-dwarfs-tools

[English](README.md) | 简体中文

一个致力于让 Windows 用户更便捷地使用 [dwarfs](https://github.com/mhx/dwarfs) 进行压缩和解压的工具。它通过为 Windows 系统添加右键菜单项，使用户能够直接通过右键点击文件或文件夹来执行 zstd 压缩和解压操作。

windows-dwarfs-tools 内置了 64 位的 dwarfs 可执行文件和 winfsp dll，可以在 Windows 系统上直接使用，免去了安装依赖项的麻烦。

## 主要特点

- **高效压缩**: 利用 dwarfs 进行高压缩率的存档，兼顾性能和压缩率。
- **右键菜单集成**: 轻松通过右键菜单进行压缩和解压。

## 安装

任选其一：

- 从 [Release](https://github.com/lxl66566/windows-dwarfs-tools/releases) 下载可执行文件，并将其放置在 `C:\Windows\System32` 或任意包含在 `PATH` 的目录下。
- 使用 [bpm](https://github.com/lxl66566/bpm) 安装：`bpm i lxl66566/windows-dwarfs-tools`。

安装后，直接点击 exe 运行或命令行不带参数运行，即可添加右键菜单项。

## 提示

- dwarfs 仅能压缩文件夹，不支持压缩单个文件。
- 除了右键菜单，本项目也提供了命令行接口。请运行 `windows-dwarfs-tools -h` 查看帮助信息。

关于压缩等级的说明，可以参考 dwarfs 的文档：

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
