use std::path::{Path, PathBuf};

const HELPER_NAME: &str = "agent-quota-widget-reload";

fn helper_path_for(executable: &Path) -> Result<PathBuf, String> {
    executable
        .parent()
        .map(|directory| directory.join(HELPER_NAME))
        .ok_or_else(|| "Unable to resolve the Widget reload helper path".to_string())
}

pub(crate) fn reload_timelines() -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Unable to resolve the app executable: {error}"))?;
    let helper = helper_path_for(&executable)?;
    if !helper.is_file() {
        return Err(format!(
            "Widget reload helper is missing at {}",
            helper.display()
        ));
    }
    let status = std::process::Command::new(&helper)
        .status()
        .map_err(|error| format!("Unable to run Widget reload helper: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Widget reload helper exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_lives_next_to_the_host_executable() {
        let executable =
            Path::new("/Applications/Agent Quota Control.app/Contents/MacOS/agent-quota-control");
        assert_eq!(
            helper_path_for(executable).unwrap(),
            Path::new(
                "/Applications/Agent Quota Control.app/Contents/MacOS/agent-quota-widget-reload"
            )
        );
    }
}
