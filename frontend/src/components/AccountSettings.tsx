import { Check, KeyRound, Pencil, Plus, Terminal, Trash2, X } from "lucide-react";
import { useState } from "react";
import { api } from "../api";
import codexIcon from "../assets/codex.png";
import kimiIcon from "../assets/kimi.png";
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
    if (!window.confirm(`删除账号“${account.displayName}”？`)) return;
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
            上游监控账号
          </div>
          <p className="muted">每个账号对应一张概览卡片，也可用于桌面 Widget。</p>
        </div>
        <div className="button-row account-add-actions">
          <button className="secondary compact" type="button" onClick={() => beginAdd("kimi")}>
            <Plus size={14} strokeWidth={1.75} aria-hidden />
            添加 Kimi 账号
          </button>
          <button className="secondary compact" type="button" onClick={() => beginAdd("codex")}>
            <Terminal size={14} strokeWidth={1.75} aria-hidden />
            导入 Codex 账号
          </button>
        </div>
      </div>

      {addMode && (
        <div className="account-form" aria-label={addMode === "kimi" ? "添加 Kimi 账号" : "导入 Codex 账号"}>
          <label className="field">
            账号名称
            <input
              aria-label="账号名称"
              autoFocus
              placeholder={addMode === "kimi" ? "例如：工作 Kimi" : "例如：个人 Codex"}
              value={displayName}
              onChange={(event) => setDisplayName(event.currentTarget.value)}
            />
          </label>
          {addMode === "kimi" && (
            <>
              <label className="field">
                Kimi API Key
                <input
                  type="password"
                  value={apiKey}
                  onChange={(event) => setApiKey(event.currentTarget.value)}
                  autoComplete="off"
                />
              </label>
              <label className="field">
                存储方式
                <select
                  value={backend}
                  onChange={(event) => setBackend(event.currentTarget.value as KimiCredentialBackend)}
                >
                  <option value="keychain">macOS 钥匙串</option>
                  <option value="encrypted_vault">本地加密存储</option>
                </select>
              </label>
            </>
          )}
          {addMode === "codex" && (
            <p className="muted account-form-note">
              将当前 Codex CLI 登录导入应用专属钥匙串，后续切换 CLI 登录不会改变此账号。
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
              {addMode === "kimi" ? "保存 Kimi 账号" : "导入当前 Codex 登录"}
            </button>
            <button className="secondary compact" type="button" disabled={busy} onClick={cancelAdd}>
              取消
            </button>
          </div>
        </div>
      )}

      {error && <p className="error-copy account-error">{error}</p>}

      <div className="account-list">
        {state.config.accounts.map((account) => {
          const serviceLabel = account.service === "kimi" ? "Kimi Code" : "Codex";
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
                      aria-label="新账号名称"
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
                  <button className="icon-button" type="button" title="保存名称" aria-label={`保存 ${account.displayName} 的名称`} disabled={busy || !editedName.trim()} onClick={() => void saveRename(account.id)}>
                    <Check size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                  <button className="icon-button" type="button" title="取消重命名" aria-label="取消重命名" disabled={busy} onClick={() => setEditingId(null)}>
                    <X size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                </>
              ) : (
                <>
                  <button className="icon-button" type="button" title="重命名" aria-label={`重命名 ${account.displayName}`} disabled={busy} onClick={() => beginRename(account)}>
                    <Pencil size={14} strokeWidth={1.75} aria-hidden />
                  </button>
                  <button className="icon-button danger-button" type="button" title="删除账号" aria-label={`删除 ${account.displayName}`} disabled={busy} onClick={() => void remove(account)}>
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
