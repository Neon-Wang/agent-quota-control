import { Check, ChevronDown, Plus, X } from "lucide-react";
import { useState } from "react";
import type { ToolInfo } from "../types";

interface ToolListProps {
  selectedTools: ToolInfo[];
  availableTools: ToolInfo[];
  selectedToolIds: string[];
  onChange: (ids: string[]) => void | Promise<void>;
  onLaunch: (tool: ToolInfo) => void | Promise<void>;
}

export function ToolList({
  selectedTools,
  availableTools,
  selectedToolIds,
  onChange,
}: ToolListProps) {
  const [expanded, setExpanded] = useState(false);

  function addTool(tool: ToolInfo) {
    void onChange([...selectedToolIds, tool.id].sort());
  }

  function removeTool(tool: ToolInfo) {
    void onChange(selectedToolIds.filter((id) => id !== tool.id));
  }

  return (
    <section className="panel">
      <button
        className="panel-title collapsible-title"
        type="button"
        onClick={() => setExpanded((value) => !value)}
        aria-expanded={expanded}
      >
        <span>
          <Check size={16} aria-hidden />
          工具选择
        </span>
        <span className="collapse-summary">
          已选 {selectedTools.length} · 可添加 {availableTools.length}
          <ChevronDown className={expanded ? "chevron open" : "chevron"} size={15} aria-hidden />
        </span>
      </button>
      {expanded && <div className="dual-list">
        <div>
          <h3 className="subhead">已选择</h3>
          <div className="row-list">
            {selectedTools.map((tool) => (
              <ToolSelectionRow
                key={tool.id}
                tool={tool}
                actionLabel="移除"
                onClick={() => removeTool(tool)}
                icon="remove"
              />
            ))}
            {selectedTools.length === 0 && (
              <p className="muted small">没有已选择工具。</p>
            )}
          </div>
        </div>
        <div>
          <h3 className="subhead">可添加</h3>
          <div className="row-list">
            {availableTools.map((tool) => (
              <ToolSelectionRow
                key={tool.id}
                tool={tool}
                actionLabel="添加"
                onClick={() => addTool(tool)}
                icon="add"
              />
            ))}
            {availableTools.length === 0 && (
              <p className="muted small">检测到的工具都已选择。</p>
            )}
          </div>
        </div>
      </div>}
    </section>
  );
}

function ToolSelectionRow({
  tool,
  actionLabel,
  onClick,
  icon,
}: {
  tool: ToolInfo;
  actionLabel: string;
  onClick: () => void;
  icon: "add" | "remove";
}) {
  const Icon = icon === "add" ? Plus : X;
  return (
    <div className="tool-row">
      <div>
        <strong>{tool.name}</strong>
        <p>
          {tool.toolType.toUpperCase()} · {tool.installPath ?? tool.launchAs}
        </p>
      </div>
      <button className="secondary compact" type="button" onClick={onClick}>
        <Icon size={13} aria-hidden />
        {actionLabel}
      </button>
    </div>
  );
}
