import { Check, KeyRound, Pencil, Plus, Terminal, Trash2, X } from "lucide-react";
import { useState } from "react";
import { api } from "../api";
import codexIcon from "../assets/codex.png";
import kimiIcon from "../assets/kimi.png";
import { useTranslations } from "../i18n";
import type {
  DashboardState,
  KimiCredentialBackend,
  MonitorAccount,
} from "../types";

interface AccountSettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

type AddMode = "kimi" | "codex" | null;

export function AccountSettings({ state, onChange }: AccountSettingsProps) {
  const t = useTranslations("monitoring");
  const common = useTranslations("common");
  const [addMode, setAddMode] = useState<AddMode>(null);
  const [displayName, setDisplayName] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [backend, setBackend] = useState<KimiCredentialBackend>("keychain");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editedName, setEditedName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function beginAdd(mode: Exclude<AddMode, null>) {
    setAddMode(mode);
    setDisplayName("");
    setApiKey("");
    setError(null);
  }

  function cancelAdd() {
    setAddMode(null);
    setDisplayName("");
    setApiKey("");
    setError(null);
  }

  async function saveNewAccount() {
    const name = displayName.trim();
    if (!name || (addMode === "kimi" && !apiKey.trim())) return;
    setBusy(true);
    setError(null);
    try {
      const next =
        addMode === "kimi"
          ? await api.addKimiAccount(name, apiKey.trim(), backend)
          : await api.importCodexAccount(name);
      onChange(next);
      cancelAdd();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  function beginRename(account: MonitorAccount) {
    setEditingId(account.id);
    setEditedName(account.displayName);
    setError(null);
  }

  async function saveRename(accountId: string) {
    if (!editedName.trim()) return;
    setBusy(true);
    setError(null);
    try {
      onChange(await api.renameAccount(accountId, editedName.trim()));
      setEditingId(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function remove(account: MonitorAccount) {
    if (!window.confirm(t("confirm_delete", { name: account.displayName }))) return;
    setBusy(true);
    setError(null);
    try {
      onChange(await api.removeAccount(account.id));
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel wide account-settings">
      <div className="account-settings-header">
        <div>
          <div className="panel-title">
            <KeyRound size={15} strokeWidth={1.75} aria-hidden />
            {t("accounts_title")}
          </div>
          <p className="muted">{t("accounts_hint")}</p>
        </div>
        <div className="button-row account-add-actions">
          <button className="secondary compact" type="button" onClick={() => beginAdd("kimi")}>
            <Plus size={14} strokeWidth={1.75} aria-hidden />
            {t("add_kimi")}
          </button>
          <button className="secondary compact" type="button" onClick={() => beginAdd("codex")}>
            <Terminal size={14} strokeWidth={1.75} aria-hidden />
            {t("import_codex")}
          </button>
        </div>
      </div>

      {addMode && (
        <div className="account-form" aria-label={addMode === "kimi" ? t("add_kimi") : t("import_codex")}>
          <label className="field">
            {t("account_name")}
            <input
              aria-label={t("account_name")}
              autoFocus
              placeholder={
                addMode === "kimi"
                  ? t("account_name_placeholder_kimi")
                  : t("account_name_placeholder_codex")
              }
              value={displayName}
              onChange={(event) => setDisplayName(event.currentTarget.value)}
            />
          </label>
          {addMode === "kimi" && (
            <>
              <label className="field">
                {t("kimi_api_key")}
                <input
                  type="password"
                  value={apiKey}
                  onChange={(event) => setApiKey(event.currentTarget.value)}
                  autoComplete="off"
                />
              </label>
              <label className="field">
                {t("storage_backend")}
                <select
                  value={backend}
                  onChange={(event) => setBackend(event.currentTarget.value as KimiCredentialBackend)}
                >
                  <option value="keychain">{t("backend_keychain")}</option>
                  <option value="encrypted_vault">{t("backend_vault")}</option>
                </select>
              </label>
            </>
          )}
          {addMode === "codex" && (
            <p className="muted account-form-note">
              {t("codex_import_note")}
            </p>
          )}
          <div className="button-row account-form-actions">
            <button
              className="primary compact"
              type="button"
              disabled={busy || !displayName.trim() || (addMode === "kimi" && !apiKey.trim())}
              onClick={() => void saveNewAccount()}
            >
              <Check size={14} strokeWidth={1.75} aria-hidden />
              {addMode === "kimi" ? t("save_kimi") : t("import_codex_login")}
            </button>
            <button className="secondary compact" type="button" disabled={busy} onClick={cancelAdd}>
              {common("cancel")}
            </button>
          </div>
        </div>
      )}

      {error && <p className="error-copy account-error">{error}</p>}

      <div className="account-list">
        {state.config.accounts.map((account) => {
          const serviceLabel = account.service === "kimi" ? common("kimi_code") : common("codex");
          const nameMatchesService =
            account.displayName.trim() === serviceLabel;
          const subtitle = nameMatchesService
            ? account.providerIdentityHint ?? null
            : [serviceLabel, account.providerIdentityHint]
                .filter(Boolean)
                .join(" · ");

          return (
          <div className="account-row" key={account.id}>
            <div className="account-identity">
              <img
                className="service-mark"
                src={account.service === "kimi" ? kimiIcon : codexIcon}
                alt=""
                aria-hidden
              />
              <div
                className={
                  subtitle
                    ? "account-name-stack"
                    : "account-name-stack account-name-stack-single"
                }
              >
                <div className="account-name-slot">
                  {editingId === account.id ? (
                    <input
                      className="account-name-editor"
                      aria-label={t("new_account_name")}
                      autoFocus
                      value={editedName}
                      onFocus={(event) => event.currentTarget.select()}
                      onChange={(event) => setEditedName(event.currentTarget.value)}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" && editedName.trim()) {
                          event.preventDefault();
                          void saveRename(account.id);
                        } else if (event.key === "Escape") {
                          event.preventDefault();
                          setEditingId(null);
                        }
                      }}
                    />
                  ) : (
                    <strong>{account.displayName}</strong>
                  )}
                </div>
                {subtitle ? <small>{subtitle}</small> : null}
              </div>
            </div>
            <div className="account-row-actions">
              {editingId === account.id ? (
                <>
                  <button className="icon-button" type="button" title={t("save_name")} aria-label={t("save_name_for", { name: account.displayName })} disabled={busy || !editedName.trim()} onClick={() => void saveRename(account.id)}>
                    <Check size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                  <button className="icon-button" type="button" title={t("cancel_rename")} aria-label={t("cancel_rename")} disabled={busy} onClick={() => setEditingId(null)}>
                    <X size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                </>
              ) : (
                <>
                  <button className="icon-button" type="button" title={t("rename")} aria-label={t("rename_account", { name: account.displayName })} disabled={busy} onClick={() => beginRename(account)}>
                    <Pencil size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                  <button className="icon-button danger-button" type="button" title={t("delete_account")} aria-label={t("delete_account_for", { name: account.displayName })} disabled={busy} onClick={() => void remove(account)}>
                    <Trash2 size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                </>
              )}
            </div>
          </div>
          );
        })}
      </div>
    </section>
  );
}
