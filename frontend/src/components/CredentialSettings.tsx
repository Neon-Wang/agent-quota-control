import { KeyRound, Save, Trash2 } from "lucide-react";
import { useState } from "react";
import { api } from "../api";
import type { DashboardState, KimiCredentialBackend } from "../types";

interface CredentialSettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

export function CredentialSettings({ state, onChange }: CredentialSettingsProps) {
  const [apiKey, setApiKey] = useState("");
  const [backend, setBackend] = useState<KimiCredentialBackend>(
    state.config.credentials.kimiBackend,
  );
  const [saving, setSaving] = useState(false);

  async function save() {
    if (!apiKey.trim()) return;
    setSaving(true);
    try {
      onChange(await api.saveKimiApiKey(apiKey, backend));
      setApiKey("");
    } finally {
      setSaving(false);
    }
  }

  async function clear() {
    onChange(await api.clearKimiApiKey(backend));
  }

  return (
    <section className="panel">
      <div className="panel-title">
        <KeyRound size={16} aria-hidden />
        Kimi API Key
      </div>
      <label className="field">
        存储方式
        <select
          value={backend}
          onChange={(event) =>
            setBackend(event.currentTarget.value as KimiCredentialBackend)
          }
        >
          <option value="keychain">Keychain</option>
          <option value="encrypted_vault">加密文件库</option>
        </select>
      </label>
      <label className="field">
        API Key
        <input
          type="password"
          value={apiKey}
          onChange={(event) => setApiKey(event.currentTarget.value)}
          placeholder="sk-..."
        />
      </label>
      <div className="button-row">
        <button className="primary" type="button" onClick={save} disabled={saving}>
          <Save size={14} aria-hidden />
          保存 Key
        </button>
        <button className="secondary" type="button" onClick={clear}>
          <Trash2 size={14} aria-hidden />
          清除
        </button>
      </div>
      <p className="muted">
        Keychain 会直接保存密钥。加密文件库会把密文写入配置目录，并把主密钥保存在 Keychain。
      </p>
    </section>
  );
}
