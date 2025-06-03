use rfd::FileDialog;
use std::path::PathBuf;

/// 使用 Windows API 打开一个文件选择窗，用户可以选择一个文件。
///
/// # 参数
///
/// * `extensions`: 一个字符串切片，包含允许的文件后缀名 (例如, &["txt", "doc"])。如果为空，则不进行后缀过滤。
///
/// # 返回
///
/// * `Option<PathBuf>`: 如果用户选择了一个文件，则返回 `Some(PathBuf)` 包含文件路径；如果用户取消了选择，则返回 `None`。
#[allow(unused)]
pub fn open_file_dialog(extensions: &[&str]) -> Option<PathBuf> {
    let mut dialog = FileDialog::new();

    if !extensions.is_empty() {
        // 构建过滤器名称，例如 "Text and Word files"
        let filter_name = if extensions.len() == 1 {
            format!("{} 文件", extensions[0].to_uppercase())
        } else {
            let exts_joined = extensions.join(", ");
            format!("{} 文件", exts_joined)
        };
        dialog = dialog.add_filter(&filter_name, extensions);
    }

    dialog.pick_file()
}

/// 使用 Windows API 打开一个“另存为”文件对话框，用户可以选择保存文件的位置和名称。
///
/// # 参数
///
/// * `extensions`: 一个字符串切片，包含允许的文件后缀名 (例如, &["txt", "doc"])。如果为空，则不进行后缀过滤。对话框通常会自动附加所选的后缀。
/// * `default_filename`: 一个字符串切片，表示默认显示在文件名输入框中的名称。
///
/// # 返回
///
/// * `Option<PathBuf>`: 如果用户确认了保存位置和文件名，则返回 `Some(PathBuf)` 包含完整路径；如果用户取消了操作，则返回 `None`。
pub fn save_file_dialog(extensions: &[&str], default_filename: &str) -> Option<PathBuf> {
    let mut dialog = FileDialog::new();

    if !extensions.is_empty() {
        // 为 "另存为" 对话框创建过滤器描述
        // 通常，用户选择一个过滤器后，对话框会自动处理后缀的添加
        let filter_name = if extensions.len() == 1 {
            format!(
                "{} 文件 (*.{})",
                extensions[0].to_uppercase(),
                extensions[0]
            )
        } else {
            // 对于多个后缀，可以创建一个更通用的描述
            // 或者为每个后缀创建单独的过滤器条目，rfd 支持多个 add_filter 调用
            let exts_display = extensions
                .iter()
                .map(|e| format!("*.{}", e))
                .collect::<Vec<String>>()
                .join(";");
            let exts_joined = extensions.join(", ");
            format!("{} 文件 ({})", exts_joined.to_uppercase(), exts_display)
        };
        dialog = dialog.add_filter(&filter_name, extensions);
    }

    dialog = dialog.set_file_name(default_filename);

    dialog.save_file()
}
