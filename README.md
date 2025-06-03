# windows-simple-zstd

一个致力于让 Windows 用户更便捷地使用 Zstd 进行压缩和解压的工具。它通过为 Windows 系统添加右键菜单项，使用户能够直接通过右键点击文件或文件夹来执行 zstd 压缩和解压操作。

## 主要特点

- **右键菜单集成**: 轻松通过右键菜单进行压缩和解压。
- **流式处理**: 支持流式压缩与解压，无需先将 `.tar.zst` 解压为 `.tar` 文件，再进行 `tar` 解压，简化了操作流程。
- **多核并行**: 压缩时尽可能利用多核 CPU 进行并行处理。
- **默认最高压缩级别**: 默认使用 Zstd 的最高压缩级别 (22) 进行压缩，以获得最佳压缩比。

## 安装

任选其一：

- 从 [Release](https://github.com/lxl66566/windows-simple-zstd/releases) 下载可执行文件，并将其放置在 `C:\Windows\System32` 或任意包含在 `PATH` 的目录下。
- 使用 [bpm](https://github.com/lxl66566/bpm) 安装：`bpm i lxl66566/windows-simple-zstd`。

安装后，不带参数直接执行，即可添加右键菜单项。

## CLI 使用

除了右键菜单，本项目也提供了命令行接口。请运行 `windows-simple-zstd -h` 查看帮助信息。
