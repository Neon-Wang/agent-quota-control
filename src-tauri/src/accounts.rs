use crate::types::{CredentialRef, MonitorAccount, ServiceKind};

pub(crate) fn new_account_id() -> String {
    format!("account-{:032x}", rand::random::<u128>())
}

pub(crate) fn add_account_with_id(
    accounts: &mut Vec<MonitorAccount>,
    id: &str,
    service: ServiceKind,
    display_name: &str,
    credential_ref: CredentialRef,
    created_at: i64,
) -> Result<MonitorAccount, String> {
    let id = id.trim();
    if id.is_empty() || accounts.iter().any(|account| account.id == id) {
        return Err("账号标识无效或已存在".to_string());
    }

    let display_name = normalized_name(display_name)?;
    ensure_unique_name(accounts, service, &display_name, None)?;
    let account = MonitorAccount {
        id: id.to_string(),
        service,
        display_name,
        provider_identity_hint: None,
        credential_ref,
        enabled: true,
        created_at,
    };
    accounts.push(account.clone());
    Ok(account)
}

pub(crate) fn rename_account(
    accounts: &mut [MonitorAccount],
    account_id: &str,
    display_name: &str,
) -> Result<(), String> {
    let index = accounts
        .iter()
        .position(|account| account.id == account_id)
        .ok_or_else(|| "账号不存在".to_string())?;
    let display_name = normalized_name(display_name)?;
    ensure_unique_name(
        accounts,
        accounts[index].service,
        &display_name,
        Some(account_id),
    )?;
    accounts[index].display_name = display_name;
    Ok(())
}

pub(crate) fn remove_account(
    accounts: &mut Vec<MonitorAccount>,
    account_id: &str,
) -> Result<MonitorAccount, String> {
    let index = accounts
        .iter()
        .position(|account| account.id == account_id)
        .ok_or_else(|| "账号不存在".to_string())?;
    Ok(accounts.remove(index))
}

fn normalized_name(display_name: &str) -> Result<String, String> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err("账号名称不能为空".to_string());
    }
    Ok(display_name.to_string())
}

fn ensure_unique_name(
    accounts: &[MonitorAccount],
    service: ServiceKind,
    display_name: &str,
    excluding_id: Option<&str>,
) -> Result<(), String> {
    let is_duplicate = accounts.iter().any(|account| {
        account.service == service
            && Some(account.id.as_str()) != excluding_id
            && account.display_name.eq_ignore_ascii_case(display_name)
    });
    if is_duplicate {
        return Err(format!("{} 已存在同名账号", service_label(service)));
    }
    Ok(())
}

fn service_label(service: ServiceKind) -> &'static str {
    match service {
        ServiceKind::Kimi => "Kimi",
        ServiceKind::Codex => "Codex",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kimi_account(id: &str, name: &str) -> MonitorAccount {
        MonitorAccount {
            id: id.to_string(),
            service: ServiceKind::Kimi,
            display_name: name.to_string(),
            provider_identity_hint: None,
            credential_ref: CredentialRef::KimiKeychain {
                account: format!("kimi-{id}"),
            },
            enabled: true,
            created_at: 100,
        }
    }

    #[test]
    fn add_account_trims_the_name_and_preserves_the_requested_id() {
        let mut accounts = Vec::new();

        let account = add_account_with_id(
            &mut accounts,
            "account-1",
            ServiceKind::Kimi,
            "  工作账号  ",
            CredentialRef::KimiKeychain {
                account: "kimi-account-1".to_string(),
            },
            100,
        )
        .expect("account should be accepted");

        assert_eq!(account.id, "account-1");
        assert_eq!(account.display_name, "工作账号");
        assert_eq!(accounts, vec![account]);
    }

    #[test]
    fn add_account_rejects_duplicate_names_for_the_same_service() {
        let mut accounts = vec![kimi_account("existing", "Work")];

        let error = add_account_with_id(
            &mut accounts,
            "new",
            ServiceKind::Kimi,
            " work ",
            CredentialRef::KimiKeychain {
                account: "kimi-new".to_string(),
            },
            101,
        )
        .expect_err("duplicate name should be rejected");

        assert_eq!(error, "Kimi 已存在同名账号");
        assert_eq!(accounts.len(), 1);
    }

    #[test]
    fn rename_account_preserves_the_stable_id() {
        let mut accounts = vec![kimi_account("stable", "Old")];

        rename_account(&mut accounts, "stable", " New ").expect("rename should work");

        assert_eq!(accounts[0].id, "stable");
        assert_eq!(accounts[0].display_name, "New");
    }

    #[test]
    fn remove_account_returns_the_removed_record() {
        let mut accounts = vec![kimi_account("remove-me", "Personal")];

        let removed = remove_account(&mut accounts, "remove-me").expect("account should exist");

        assert_eq!(removed.id, "remove-me");
        assert!(accounts.is_empty());
    }

    #[test]
    fn generated_account_ids_are_unique_and_storage_safe() {
        let first = new_account_id();
        let second = new_account_id();

        assert_ne!(first, second);
        assert!(first.starts_with("account-"));
        assert!(first
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-'));
    }
}
