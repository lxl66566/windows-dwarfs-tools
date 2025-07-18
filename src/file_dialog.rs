use rfd::FileDialog;
use std::path::PathBuf;

/// Opens a file selection dialog using the Windows API, allowing the user to choose a file.
///
/// # Arguments
///
/// * `extensions`: A slice of strings containing allowed file extensions (e.g., &["txt", "doc"]). If empty, no extension filtering is applied.
///
/// # Returns
///
/// * `Option<PathBuf>`: Returns `Some(PathBuf)` with the file path if the user selected a file; returns `None` if the user cancelled the selection.
#[allow(unused)]
pub fn open_file_dialog(extensions: &[&str]) -> Option<PathBuf> {
    let mut dialog = FileDialog::new();

    if !extensions.is_empty() {
        // Build filter name, e.g., "Text and Word files"
        let filter_name = if extensions.len() == 1 {
            format!("{} Files", extensions[0].to_uppercase())
        } else {
            let exts_joined = extensions.join(", ");
            format!("{exts_joined} Files")
        };
        dialog = dialog.add_filter(&filter_name, extensions);
    }

    dialog.pick_file()
}

/// Opens a "Save As" file dialog using the Windows API, allowing the user to choose the location and name for saving a file.
///
/// # Arguments
///
/// * `extensions`: A slice of strings containing allowed file extensions (e.g., &["txt", "doc"]). If empty, no extension filtering is applied. The dialog usually automatically appends the selected extension.
/// * `default_filename`: A string slice representing the default name displayed in the filename input box.
///
/// # Returns
///
/// * `Option<PathBuf>`: Returns `Some(PathBuf)` with the full path if the user confirmed the save location and filename; returns `None` if the user cancelled the operation.
pub fn save_file_dialog(extensions: &[&str], default_filename: &str) -> Option<PathBuf> {
    let mut dialog = FileDialog::new();

    if !extensions.is_empty() {
        // Create filter description for "Save As" dialog
        // Typically, after the user selects a filter, the dialog automatically handles appending the extension
        let filter_name = if extensions.len() == 1 {
            format!(
                "{} Files (*.{})",
                extensions[0].to_uppercase(),
                extensions[0]
            )
        } else {
            // For multiple extensions, a more general description can be created
            // Or separate filter entries can be created for each extension, rfd supports multiple add_filter calls
            let exts_display = extensions
                .iter()
                .map(|e| format!("*.{e}"))
                .collect::<Vec<String>>()
                .join(";");
            let exts_joined = extensions.join(", ");
            format!("{} Files ({})", exts_joined.to_uppercase(), exts_display)
        };
        dialog = dialog.add_filter(&filter_name, extensions);
    }

    dialog = dialog.set_file_name(default_filename);

    dialog.save_file()
}
