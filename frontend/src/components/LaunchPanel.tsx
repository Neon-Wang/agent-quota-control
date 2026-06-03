import { Play, Terminal } from "lucide-react";
import type { ToolInfo } from "../types";

interface LaunchPanelProps {
  tools: ToolInfo[];
  onLaunch: (tool: ToolInfo) => void | Promise<void>;
}

export function LaunchPanel({ tools, onLaunch }: LaunchPanelProps) {
  const ideTools = tools.filter((tool) => tool.toolType === "ide");
  const cliTools = tools.filter((tool) => tool.toolType === "cli");

  return (
    <section className="panel">
      <div className="panel-title">
        <Terminal size={16} aria-hidden />
        已选择启动器
      </div>
      {tools.length === 0 ? (
        <p className="muted">还没有选择启动器。可以在下方添加工具。</p>
      ) : (
        <div className="launcher-grid">
          <ToolGroup title="IDE 与应用" tools={ideTools} onLaunch={onLaunch} />
          <ToolGroup title="CLI 工具" tools={cliTools} onLaunch={onLaunch} />
        </div>
      )}
    </section>
  );
}

function ToolGroup({
  title,
  tools,
  onLaunch,
}: {
  title: string;
  tools: ToolInfo[];
  onLaunch: (tool: ToolInfo) => void | Promise<void>;
}) {
  return (
    <div>
      <h3 className="subhead">{title}</h3>
      {tools.length === 0 ? (
        <p className="muted small">未选择。</p>
      ) : (
        <div className="row-list">
          {tools.map((tool) => (
            <div className="tool-row" key={tool.id}>
              <div>
                <strong>{tool.name}</strong>
                <p>{tool.installPath ?? tool.launchAs ?? "可启动"}</p>
              </div>
              <button
                className="icon-button"
                type="button"
                aria-label={`启动 ${tool.name}`}
                onClick={() => void onLaunch(tool)}
              >
                <Play size={14} aria-hidden />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
