use anyhow::{Result, anyhow};
use std::env;
use winreg::{RegKey, enums::*};

const MENU_NAME: &str = env!("CARGO_PKG_NAME"); // Main menu item name
const FILE_SHELL_PATH: &str = ".dwarfs\\shell"; // Applies to files
const DIRECTORY_SHELL_PATH: &str = "Directory\\shell"; // Applies to folders themselves and folder background
const FOLDER_SHELL_PATH: &str = "Folder\\shell"; // Primarily applies to folder items themselves

// Subcommand definition
struct SubCommandInfo<'a> {
    key_name: &'a str,     // Key name used for the submenu item in the registry
    display_name: &'a str, // Name displayed in the context menu
    arg_template: &'a str, // Command argument template, {} will be replaced by exe_path
}

// Subcommand list
const SUB_COMMANDS: [SubCommandInfo; 4] = [
    SubCommandInfo {
        key_name: "CompressQuick",
        display_name: "Quick Compress",
        arg_template: "\"{}\" c \"%1\"", // Note quotes for path and arguments
    },
    SubCommandInfo {
        key_name: "CompressTo",
        display_name: "Compress to...",
        arg_template: "\"{}\" c -i \"%1\"",
    },
    SubCommandInfo {
        key_name: "DecompressQuick",
        display_name: "Quick Decompress",
        arg_template: "\"{}\" d \"%1\"",
    },
    SubCommandInfo {
        key_name: "DecompressTo",
        display_name: "Decompress to...",
        arg_template: "\"{}\" d -i \"%1\"",
    },
];

/// Adds context menu entries.
///
/// Adds an expandable context menu for files and folders, with subcommands defined directly under the main menu's shell subkey.
pub fn add_context_menu_entries() -> Result<()> {
    let current_exe = env::current_exe()?;
    let exe_path = current_exe
        .to_str()
        .ok_or_else(|| anyhow!("Invalid executable path"))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes_key = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .or_else(|_| hkcu.create_subkey("Software\\Classes").map(|x| x.0))?;

    // Add menus for different association types
    add_menu_for_shell_path_prefix(&classes_key, FILE_SHELL_PATH, exe_path)?;
    // "Directory" for folders and folder background
    add_menu_for_shell_path_prefix(&classes_key, DIRECTORY_SHELL_PATH, exe_path)?;
    // "Folder" is also usually recommended to ensure coverage for folder items
    add_menu_for_shell_path_prefix(&classes_key, FOLDER_SHELL_PATH, exe_path)?;

    println!("Successfully added context menu entries: {MENU_NAME}");
    Ok(())
}

/// Adds the main menu and all its submenu items for the specified shell path prefix (e.g., "*\\shell").
fn add_menu_for_shell_path_prefix(
    classes_key: &RegKey,    // HKCU\Software\Classes
    shell_path_prefix: &str, // E.g., "*\\shell", "Directory\\shell"
    exe_path: &str,
) -> Result<()> {
    // 1. Create the main menu item key, e.g., HKCU\Software\Classes\*\shell\Zstd Tool
    let (main_menu_key, _) =
        classes_key.create_subkey(format!("{shell_path_prefix}\\{MENU_NAME}"))?;

    // Set the display name for the main menu
    main_menu_key.set_value("MUIVerb", &MENU_NAME)?;
    // (Optional) Set an icon for the main menu, pointing to your program and icon index (0 is usually the first)
    main_menu_key.set_value("Icon", &format!("\"{exe_path}\",0"))?;

    // (Optional but recommended) Set SubCommands to an empty string to explicitly indicate this is a menu with subcommands.
    // Even if subcommands are defined directly under its "shell" subkey.
    main_menu_key.set_value("SubCommands", &"")?;

    // 2. Create a "shell" subkey under the main menu item to hold all subcommand items
    // E.g., HKCU\Software\Classes\*\shell\Zstd Tool\shell
    let (sub_menu_container_key, _) = main_menu_key.create_subkey("shell")?;

    // 3. Add each subcommand item under this "shell" subkey
    for sc_info in SUB_COMMANDS.iter() {
        // Create the subcommand item key, e.g., HKCU\Software\Classes\*\shell\Zstd Tool\shell\CompressQuick
        let (sub_command_entry_key, _) = sub_menu_container_key.create_subkey(sc_info.key_name)?;

        // Set the display name for the subcommand item
        sub_command_entry_key.set_value("MUIVerb", &sc_info.display_name)?;
        // (Optional) Set an icon for the subcommand item
        // sub_command_entry_key.set_value("Icon", &format!("\"{}\",0", exe_path))?;

        // Create the command subkey to store the actual command to execute
        let (command_key, _) = sub_command_entry_key.create_subkey("command")?;
        let command_str = sc_info.arg_template.replace("{}", exe_path);
        command_key.set_value("", &command_str)?; // Command string as the default value of the command key
    }

    Ok(())
}

/// Removes context menu entries.
pub fn remove_context_menu_entries() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(classes_key) = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE) {
        // Since all subcommands are under the main menu item, simply recursively delete the main menu item
        let paths_to_delete = [FILE_SHELL_PATH, DIRECTORY_SHELL_PATH, FOLDER_SHELL_PATH];
        for path_prefix in paths_to_delete.iter() {
            let _ = classes_key.delete_subkey_all(format!("{path_prefix}\\{MENU_NAME}"));
        }
    }

    println!("Successfully removed context menu entries");
    Ok(())
}
