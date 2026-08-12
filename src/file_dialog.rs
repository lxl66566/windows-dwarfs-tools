use std::path::PathBuf;

use rfd::FileDialog;

/// 构造过滤器显示名，例如 `"DWARFS Files (*.dwarfs)"`。
fn make_filter_name(extensions: &[&str]) -> String {
    if extensions.len() == 1 {
        format!(
            "{} Files (*.{})",
            extensions[0].to_uppercase(),
            extensions[0]
        )
    } else {
        let exts_display = extensions
            .iter()
            .map(|e| format!("*.{e}"))
            .collect::<Vec<_>>()
            .join(";");
        format!(
            "{} Files ({})",
            extensions.join(", ").to_uppercase(),
            exts_display
        )
    }
}

/// 为对话框添加扩展名过滤器；`extensions` 为空时不做任何过滤。
///
/// 注意：传入的是纯扩展名（如 `"dwarfs"`），不带 `*.` 前缀，rfd 会自行拼接。
fn apply_filter(dialog: FileDialog, extensions: &[&str]) -> FileDialog {
    if extensions.is_empty() {
        dialog
    } else {
        dialog.add_filter(make_filter_name(extensions), extensions)
    }
}

/// Opens a file selection dialog using the Windows API, allowing the user to choose a file.
///
/// # Arguments
///
/// * `extensions`: A slice of strings containing allowed file extensions (e.g., `&["txt", "doc"]`).
///   If empty, no extension filtering is applied.
///
/// # Returns
///
/// * `Option<PathBuf>`: Returns `Some(PathBuf)` with the file path if the user selected a file;
///   returns `None` if the user cancelled the selection.
#[allow(unused)]
pub fn open_file_dialog(extensions: &[&str]) -> Option<PathBuf> {
    apply_filter(FileDialog::new(), extensions).pick_file()
}

/// Opens a "Save As" file dialog using the Windows API, allowing the user to choose the location
/// and name for saving a file.
///
/// # Arguments
///
/// * `extensions`: A slice of strings containing allowed file extensions (e.g., `&["txt", "doc"]`).
///   If empty, no extension filtering is applied. The dialog usually automatically appends the
///   selected extension.
/// * `default_filename`: A string slice representing the default name displayed in the filename
///   input box.
///
/// # Returns
///
/// * `Option<PathBuf>`: Returns `Some(PathBuf)` with the full path if the user confirmed the save
///   location and filename; returns `None` if the user cancelled the operation.
pub fn save_file_dialog(extensions: &[&str], default_filename: &str) -> Option<PathBuf> {
    apply_filter(FileDialog::new(), extensions)
        .set_file_name(default_filename)
        .save_file()
}
