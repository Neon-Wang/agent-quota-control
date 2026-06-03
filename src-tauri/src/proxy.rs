use crate::types::{ProxyMode, ProxyTestResult, ServiceProxyConfig};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxySelection {
    Direct,
    Proxy(String),
    Unavailable(String),
}

pub fn select_proxy(
    config: &ServiceProxyConfig,
    can_connect: impl Fn(&str, u64) -> bool,
) -> ProxySelection {
    match config.mode {
        ProxyMode::Off => ProxySelection::Direct,
        ProxyMode::On => match config.proxy_url.as_deref() {
            Some(url) if is_valid_proxy_url(url) => ProxySelection::Proxy(url.to_string()),
            Some(_) => ProxySelection::Unavailable("Invalid proxy URL".to_string()),
            None => ProxySelection::Unavailable("Proxy URL is required".to_string()),
        },
        ProxyMode::Auto => {
            let mut candidates = Vec::new();
            if let Some(url) = config.proxy_url.as_deref().filter(|url| !url.is_empty()) {
                candidates.push(url.to_string());
            }
            candidates.extend(
                config
                    .auto_ports
                    .iter()
                    .map(|port| format!("http://127.0.0.1:{port}")),
            );

            for candidate in candidates {
                if is_valid_proxy_url(&candidate) && can_connect(&candidate, config.timeout_ms) {
                    return ProxySelection::Proxy(candidate);
                }
            }
            ProxySelection::Direct
        }
    }
}

pub fn apply_proxy(
    builder: reqwest::ClientBuilder,
    service: &str,
    config: &ServiceProxyConfig,
) -> reqwest::ClientBuilder {
    match select_proxy(config, proxy_accepts_tcp) {
        ProxySelection::Proxy(url) => match reqwest::Proxy::all(&url) {
            Ok(proxy) => {
                log::info!("{service} request using proxy {url}");
                builder.proxy(proxy)
            }
            Err(error) => {
                log::warn!("{service} proxy configuration failed: {error}");
                builder
            }
        },
        ProxySelection::Direct => {
            log::debug!("{service} request using direct connection");
            builder
        }
        ProxySelection::Unavailable(message) => {
            log::warn!("{service} proxy unavailable: {message}");
            builder
        }
    }
}

pub fn test_proxy_config(config: &ServiceProxyConfig) -> ProxyTestResult {
    match select_proxy(config, proxy_accepts_tcp) {
        ProxySelection::Proxy(url) => ProxyTestResult {
            status: "proxy".to_string(),
            proxy_url: Some(url.clone()),
            message: format!("代理已连接：{url}"),
        },
        ProxySelection::Direct => ProxyTestResult {
            status: "direct".to_string(),
            proxy_url: None,
            message: "未检测到可用代理，当前走直连".to_string(),
        },
        ProxySelection::Unavailable(message) => ProxyTestResult {
            status: "unavailable".to_string(),
            proxy_url: None,
            message: localized_unavailable_message(&message),
        },
    }
}

fn localized_unavailable_message(message: &str) -> String {
    match message {
        "Invalid proxy URL" => "代理地址无效，请使用 http、https 或 socks5 地址".to_string(),
        "Proxy URL is required" => "已开启强制代理，但尚未填写代理地址".to_string(),
        other => format!("代理未连通：{other}"),
    }
}

fn proxy_accepts_tcp(url: &str, timeout_ms: u64) -> bool {
    let Ok(url) = url::Url::parse(url) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    let Ok(addr) = format!("{host}:{port}").parse::<SocketAddr>() else {
        return false;
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok()
}

fn is_valid_proxy_url(value: &str) -> bool {
    match url::Url::parse(value) {
        Ok(url) => matches!(url.scheme(), "http" | "https" | "socks5") && url.host_str().is_some(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(mode: ProxyMode) -> ServiceProxyConfig {
        ServiceProxyConfig {
            mode,
            proxy_url: None,
            auto_ports: vec![7897, 7890],
            timeout_ms: 250,
        }
    }

    #[test]
    fn auto_chooses_first_available_port() {
        let selected = select_proxy(&config(ProxyMode::Auto), |url, _| url.ends_with(":7897"));

        assert_eq!(
            selected,
            ProxySelection::Proxy("http://127.0.0.1:7897".to_string())
        );
    }

    #[test]
    fn auto_falls_back_to_second_port() {
        let selected = select_proxy(&config(ProxyMode::Auto), |url, _| url.ends_with(":7890"));

        assert_eq!(
            selected,
            ProxySelection::Proxy("http://127.0.0.1:7890".to_string())
        );
    }

    #[test]
    fn auto_returns_direct_when_no_proxy_available() {
        assert_eq!(
            select_proxy(&config(ProxyMode::Auto), |_, _| false),
            ProxySelection::Direct
        );
    }

    #[test]
    fn on_requires_valid_proxy_url() {
        let mut config = config(ProxyMode::On);
        config.proxy_url = Some("not a url".to_string());

        assert!(matches!(
            select_proxy(&config, |_, _| true),
            ProxySelection::Unavailable(_)
        ));
    }

    #[test]
    fn off_never_probes_ports() {
        let selected = select_proxy(&config(ProxyMode::Off), |_, _| panic!("should not probe"));

        assert_eq!(selected, ProxySelection::Direct);
    }
}
