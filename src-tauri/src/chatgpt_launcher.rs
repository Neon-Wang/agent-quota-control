use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

const CHATGPT_EXECUTABLE: &str = "/Applications/ChatGPT.app/Contents/MacOS/ChatGPT";
const PROXY_HTTP: &str = "http://127.0.0.1:7897";
const PROXY_SOCKS: &str = "socks5://127.0.0.1:7897";
const NO_PROXY: &str = "localhost,127.0.0.1,::1";

#[derive(Debug, PartialEq, Eq)]
struct ChatGptLaunchSpec {
    executable: &'static str,
    args: [&'static str; 2],
    environment: [(&'static str, &'static str); 8],
}

impl ChatGptLaunchSpec {
    fn env(&self, name: &str) -> Option<&'static str> {
        self.environment
            .iter()
            .find_map(|(key, value)| (*key == name).then_some(*value))
    }
}

fn chatgpt_launch_spec() -> ChatGptLaunchSpec {
    ChatGptLaunchSpec {
        executable: CHATGPT_EXECUTABLE,
        args: [
            "--proxy-server=http://127.0.0.1:7897",
            "--proxy-bypass-list=localhost;127.0.0.1;::1",
        ],
        environment: [
            ("HTTP_PROXY", PROXY_HTTP),
            ("HTTPS_PROXY", PROXY_HTTP),
            ("ALL_PROXY", PROXY_SOCKS),
            ("NO_PROXY", NO_PROXY),
            ("http_proxy", PROXY_HTTP),
            ("https_proxy", PROXY_HTTP),
            ("all_proxy", PROXY_SOCKS),
            ("no_proxy", NO_PROXY),
        ],
    }
}

pub fn is_chatgpt_installed() -> bool {
    Path::new(CHATGPT_EXECUTABLE)
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

pub fn is_chatgpt_running() -> bool {
    Command::new("/usr/bin/pgrep")
        .args(["-x", "ChatGPT"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn validate_launch_state(installed: bool, running: bool) -> Result<(), String> {
    if !installed {
        return Err("未找到 /Applications/ChatGPT.app".to_string());
    }
    if running {
        return Err("ChatGPT 已在运行，请先完全退出后再使用 7897 代理启动".to_string());
    }
    Ok(())
}

pub fn launch_chatgpt_with_7897() -> Result<(), String> {
    validate_launch_state(is_chatgpt_installed(), is_chatgpt_running())?;
    let spec = chatgpt_launch_spec();
    let child = Command::new(spec.executable)
        .args(spec.args)
        .envs(spec.environment)
        .spawn()
        .map_err(|error| format!("无法通过 7897 代理启动 ChatGPT: {error}"))?;
    if let Err(error) = Command::new("/usr/bin/caffeinate")
        .args(["-d", "-w", &child.id().to_string()])
        .spawn()
    {
        log::warn!("ChatGPT 已启动，但无法保持唤醒状态: {error}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_spec_matches_the_7897_shell_launcher_contract() {
        let spec = chatgpt_launch_spec();

        assert_eq!(
            spec.executable,
            "/Applications/ChatGPT.app/Contents/MacOS/ChatGPT"
        );
        assert!(spec.args.contains(&"--proxy-server=http://127.0.0.1:7897"));
        assert!(spec
            .args
            .contains(&"--proxy-bypass-list=localhost;127.0.0.1;::1"));
        assert_eq!(spec.env("HTTP_PROXY"), Some("http://127.0.0.1:7897"));
        assert_eq!(spec.env("HTTPS_PROXY"), Some("http://127.0.0.1:7897"));
        assert_eq!(spec.env("ALL_PROXY"), Some("socks5://127.0.0.1:7897"));
        assert_eq!(spec.env("NO_PROXY"), Some("localhost,127.0.0.1,::1"));
        assert_eq!(spec.env("http_proxy"), spec.env("HTTP_PROXY"));
        assert_eq!(spec.env("https_proxy"), spec.env("HTTPS_PROXY"));
        assert_eq!(spec.env("all_proxy"), spec.env("ALL_PROXY"));
        assert_eq!(spec.env("no_proxy"), spec.env("NO_PROXY"));
    }

    #[test]
    fn launch_validation_rejects_missing_or_running_chatgpt() {
        assert_eq!(
            validate_launch_state(false, false),
            Err("未找到 /Applications/ChatGPT.app".to_string())
        );
        assert_eq!(
            validate_launch_state(true, true),
            Err("ChatGPT 已在运行，请先完全退出后再使用 7897 代理启动".to_string())
        );
        assert_eq!(validate_launch_state(true, false), Ok(()));
    }
}
