use crate::types::{ToolInfo, ToolType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchKind {
    IdeApp,
    Cli,
}

#[derive(Debug, Clone)]
pub struct ResolvedLaunch {
    pub tool: ToolInfo,
    pub command: Vec<String>,
}

pub fn resolve_launch(
    tool: &ToolInfo,
    project_dir: Option<&str>,
    ghostty_exists: bool,
) -> Result<ResolvedLaunch, String> {
    match tool.tool_type {
        ToolType::IDE => resolve_ide_launch(tool),
        ToolType::CLI => resolve_cli_launch(tool, project_dir, ghostty_exists),
    }
}

pub fn launch_tool(tool: &ToolInfo, project_dir: Option<&str>) -> Result<(), String> {
    let ghostty_exists = std::path::Path::new("/Applications/Ghostty.app").exists();
    let resolved = resolve_launch(tool, project_dir, ghostty_exists)?;
    let Some((program, args)) = resolved.command.split_first() else {
        return Err("Launch command is empty".to_string());
    };
    std::process::Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch {}: {e}", tool.name))
}

fn resolve_ide_launch(tool: &ToolInfo) -> Result<ResolvedLaunch, String> {
    let launch_name = tool.launch_as.as_deref().unwrap_or(&tool.name);
    Ok(ResolvedLaunch {
        tool: tool.clone(),
        command: vec![
            "open".to_string(),
            "-a".to_string(),
            launch_name.to_string(),
        ],
    })
}

fn resolve_cli_launch(
    tool: &ToolInfo,
    project_dir: Option<&str>,
    ghostty_exists: bool,
) -> Result<ResolvedLaunch, String> {
    let binary = tool
        .install_path
        .as_deref()
        .ok_or_else(|| format!("{} has no executable path", tool.name))?;
    let binary_path = std::path::Path::new(binary);
    if !binary_path.is_file() {
        return Err(format!("{} is not an executable file", binary));
    }
    let project_dir = project_dir.ok_or_else(|| "Project folder is required".to_string())?;
    let shell_cmd = format!(
        "cd {} && exec {}",
        shell_quote(project_dir),
        shell_quote(binary)
    );
    let command = if ghostty_exists {
        vec![
            "open".to_string(),
            "-na".to_string(),
            "Ghostty.app".to_string(),
            "--args".to_string(),
            "-e".to_string(),
            "/bin/zsh".to_string(),
            "-lc".to_string(),
            shell_cmd,
        ]
    } else {
        vec![
            "osascript".to_string(),
            "-e".to_string(),
            format!(
                "tell application \"Terminal\" to do script {}",
                applescript_string(&shell_cmd)
            ),
        ]
    };

    Ok(ResolvedLaunch {
        tool: tool.clone(),
        command,
    })
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn applescript_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(path: &str) -> ToolInfo {
        ToolInfo {
            id: "codex_cli".to_string(),
            name: "Codex CLI".to_string(),
            tool_type: ToolType::CLI,
            installed: true,
            install_path: Some(path.to_string()),
            launch_as: None,
        }
    }

    #[test]
    fn ghostty_command_uses_open_with_zsh_login_command() {
        let path = std::env::current_exe().unwrap();
        let resolved = resolve_launch(
            &cli(&path.to_string_lossy()),
            Some("/Users/test/Project A"),
            true,
        )
        .unwrap();

        assert_eq!(
            &resolved.command[0..6],
            ["open", "-na", "Ghostty.app", "--args", "-e", "/bin/zsh"]
        );
        assert!(resolved
            .command
            .last()
            .unwrap()
            .contains("cd '/Users/test/Project A'"));
    }

    #[test]
    fn cli_rejects_config_directory_as_executable() {
        let error =
            resolve_launch(&cli("/Users/test/.codex"), Some("/Users/test"), true).unwrap_err();

        assert!(error.contains("not an executable file"));
    }

    #[test]
    fn ide_uses_open_application_name() {
        let tool = ToolInfo {
            id: "vscode".to_string(),
            name: "VS Code".to_string(),
            tool_type: ToolType::IDE,
            installed: true,
            install_path: Some("/Applications/Visual Studio Code.app".to_string()),
            launch_as: Some("Visual Studio Code".to_string()),
        };

        let resolved = resolve_launch(&tool, None, false).unwrap();

        assert_eq!(resolved.command, ["open", "-a", "Visual Studio Code"]);
    }
}
