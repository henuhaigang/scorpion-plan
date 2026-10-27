import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type PanelState = "visible" | "hidden";

export default function App() {
  const [draft, setDraft] = useState("");
  const [status, setStatus] = useState("正在加载草稿…");
  const [panelState, setPanelState] = useState<PanelState>("hidden");
  const saveTimer = useRef<number | null>(null);
  const draftRef = useRef(draft);

  draftRef.current = draft;

  const saveDraft = useCallback((value: string) => {
    if (saveTimer.current !== null) {
      window.clearTimeout(saveTimer.current);
    }

    saveTimer.current = window.setTimeout(() => {
      invoke("save_draft", { draft: value }).catch(() => {
        setStatus("草稿保存失败");
      });
    }, 150);
  }, []);

  useEffect(() => {
    let active = true;

    Promise.all([invoke<string>("load_draft"), invoke<string>("panel_state")])
      .then(([savedDraft, state]) => {
        if (!active) {
          return;
        }
        setDraft(savedDraft);
        setPanelState(state === "visible" ? "visible" : "hidden");
        setStatus("快速面板已就绪");
      })
      .catch(() => {
        if (!active) {
          return;
        }
        setStatus("本地服务未连接，草稿暂不可用");
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        invoke("panel_hide").catch(() => undefined);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  useEffect(() => {
    return () => {
      if (saveTimer.current !== null) {
        window.clearTimeout(saveTimer.current);
      }
      invoke("save_draft", { draft: draftRef.current }).catch(() => undefined);
    };
  }, []);

  const updateDraft = (value: string) => {
    setDraft(value);
    saveDraft(value);
  };

  return (
    <main className="panel">
      <header className="panel-header">
        <div>
          <h1 className="panel-title">DayFlow</h1>
          <p className="panel-subtitle">让重要的事情，主动出现在你面前。</p>
        </div>
        <span className="panel-state" data-testid="panel-state">
          {panelState === "visible" ? "面板可见" : "面板隐藏"}
        </span>
      </header>

      <label className="draft-label" htmlFor="quick-add">
        快速添加
      </label>
      <textarea
        id="quick-add"
        className="draft-input"
        value={draft}
        onChange={(event) => updateDraft(event.target.value)}
        placeholder="输入任务，按 Esc 隐藏面板"
        data-testid="draft-input"
      />

      <div className="panel-actions">
        <button type="button" onClick={() => invoke("panel_hide").catch(() => undefined)}>
          隐藏面板
        </button>
        <button type="button" onClick={() => invoke("panel_toggle").catch(() => undefined)}>
          切换面板
        </button>
        <button type="button" onClick={() => invoke("panel_quit").catch(() => undefined)}>
          退出
        </button>
      </div>

      <p className="panel-status" data-testid="app-status">
        {status}
      </p>
    </main>
  );
}
