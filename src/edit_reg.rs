use anyhow::{Result, anyhow};
use std::env;
use winreg::{RegKey, enums::*};

const MENU_NAME: &str = "Zstd 工具"; // 主菜单项名称
const FILE_SHELL_PATH: &str = "*\\shell"; // 应用于所有文件
const DIRECTORY_SHELL_PATH: &str = "Directory\\shell"; // 应用于文件夹本身和文件夹背景空白处
const FOLDER_SHELL_PATH: &str = "Folder\\shell"; // 主要应用于文件夹项本身

// 子命令定义
struct SubCommandInfo<'a> {
    key_name: &'a str,     // 在注册表中用作子菜单项的键名
    display_name: &'a str, // 在右键菜单中显示的名称
    arg_template: &'a str, // 命令参数模板, {} 会被 exe_path 替换
}

// 子命令列表
const SUB_COMMANDS: [SubCommandInfo; 4] = [
    SubCommandInfo {
        key_name: "CompressQuick",
        display_name: "快速压缩",
        arg_template: "\"{}\" c \"%1\"", // 注意路径和参数的引号
    },
    SubCommandInfo {
        key_name: "CompressTo",
        display_name: "压缩到...",
        arg_template: "\"{}\" c -i \"%1\"",
    },
    SubCommandInfo {
        key_name: "DecompressQuick",
        display_name: "快速解压",
        arg_template: "\"{}\" d \"%1\"",
    },
    SubCommandInfo {
        key_name: "DecompressTo",
        display_name: "解压到...",
        arg_template: "\"{}\" d -i \"%1\"",
    },
];

/// 添加右键菜单项。
///
/// 为文件和文件夹添加一个可展开的右键菜单，子命令直接定义在主菜单的 shell 子键下。
pub fn add_context_menu_entries() -> Result<()> {
    let current_exe = env::current_exe()?;
    let exe_path = current_exe
        .to_str()
        .ok_or_else(|| anyhow!("无效的可执行文件路径"))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes_key = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .or_else(|_| hkcu.create_subkey("Software\\Classes").map(|x| x.0))?;

    // 为不同的关联类型添加菜单
    // "*" 用于文件
    add_menu_for_shell_path_prefix(&classes_key, FILE_SHELL_PATH, exe_path)?;
    // "Directory" 用于文件夹和文件夹背景
    add_menu_for_shell_path_prefix(&classes_key, DIRECTORY_SHELL_PATH, exe_path)?;
    // "Folder" 通常也建议添加，以确保覆盖文件夹项
    add_menu_for_shell_path_prefix(&classes_key, FOLDER_SHELL_PATH, exe_path)?;

    println!("成功添加右键菜单项：Zstd 工具");
    Ok(())
}

/// 为指定的 shell 路径前缀（如 "*\\shell"）添加主菜单及其所有子菜单项。
fn add_menu_for_shell_path_prefix(
    classes_key: &RegKey,    // HKCU\Software\Classes
    shell_path_prefix: &str, // 例如 "*\\shell", "Directory\\shell"
    exe_path: &str,
) -> Result<()> {
    // 1. 创建主菜单项的键，例如 HKCU\Software\Classes\*\shell\Zstd 工具
    let (main_menu_key, _) =
        classes_key.create_subkey(format!("{}\\{}", shell_path_prefix, MENU_NAME))?;

    // 设置主菜单的显示名称
    main_menu_key.set_value("MUIVerb", &MENU_NAME)?;
    // (可选) 为主菜单设置图标，指向你的程序和图标索引 (0通常是第一个)
    main_menu_key.set_value("Icon", &format!("\"{}\",0", exe_path))?;

    // (可选但推荐) 设置 SubCommands 为空字符串，以明确指示这是一个包含子命令的菜单。
    // 即使子命令是直接在其下的 "shell" 子键中定义的。
    main_menu_key.set_value("SubCommands", &"")?;

    // 2. 在主菜单项下创建 "shell" 子键，用于存放所有子命令项
    // 例如 HKCU\Software\Classes\*\shell\Zstd 工具\shell
    let (sub_menu_container_key, _) = main_menu_key.create_subkey("shell")?;

    // 3. 在这个 "shell" 子键下添加每一个子命令项
    for sc_info in SUB_COMMANDS.iter() {
        // 创建子菜单项的键，例如 HKCU\Software\Classes\*\shell\Zstd 工具\shell\CompressQuick
        let (sub_command_entry_key, _) = sub_menu_container_key.create_subkey(sc_info.key_name)?;

        // 设置子菜单项的显示名称
        sub_command_entry_key.set_value("MUIVerb", &sc_info.display_name)?;
        // (可选) 为子菜单项设置图标
        // sub_command_entry_key.set_value("Icon", &format!("\"{}\",0", exe_path))?;

        // 创建 command 子键来存储实际执行的命令
        let (command_key, _) = sub_command_entry_key.create_subkey("command")?;
        let command_str = sc_info.arg_template.replace("{}", exe_path);
        command_key.set_value("", &command_str)?; // 命令字符串作为 command 键的默认值
    }

    Ok(())
}

/// 移除右键菜单项。
pub fn remove_context_menu_entries() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(classes_key) = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE) {
        // 由于所有子命令都在主菜单项之下，只需递归删除主菜单项即可
        let paths_to_delete = [FILE_SHELL_PATH, DIRECTORY_SHELL_PATH, FOLDER_SHELL_PATH];
        for path_prefix in paths_to_delete.iter() {
            let _ = classes_key.delete_subkey_all(format!("{}\\{}", path_prefix, MENU_NAME));
        }
    }

    println!("成功移除右键菜单项");
    Ok(())
}
