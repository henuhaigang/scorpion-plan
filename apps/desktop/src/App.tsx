import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as api from "./api";
import { TaskRow } from "./TaskRow";
import type { TaskBoard, TaskView } from "./types";

type PanelState = "visible" | "hidden";

function todayISO(): string {
  const d = new Date();
  const local = new Date(d.getTime() - d.getTimezoneOffset() * 60000);
  return local.toISOString().slice(0, 10);
}

interface SectionProps {
  id: string;
  title: string;
  views: TaskView[];
  empty: string;
  busy: boolean;
  handlers: {
    onComplete: (view: TaskView) => void;
    onUndo: (view: TaskView) => void;
    onCancel: (view: TaskView) => void;
    onRestore: (view: TaskView) => void;
    onDelete: (view: TaskView) => void;
    onReschedule: (view: TaskView, date: string, time?: string) => void;
    onUpdate: (view: TaskView, title: string, priority: string) => void;
  };
}

function Section({ id, title, views, empty, busy, handlers }: SectionProps) {
  return (
    <section className="board-section" aria-labelledby={`${id}-title`}>
      <h2 id={`${id}-title`} className="board-section-title">
        {title}
      </h2>
      {views.length === 0 ? (
        <p className="board-empty">{empty}</p>
      ) : (
        <ul className="task-list">
          {views.map((view) => (
            <TaskRow key={view.task.id} view={view} busy={busy} {...handlers} />
          ))}
        </ul>
      )}
    </section>
  );
}

export default function App() {
  const [board, setBoard] = useState<TaskBoard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [panelState, setPanelState] = useState<PanelState>("hidden");

  const [draft, setDraft] = useState("");
  const [draftDate, setDraftDate] = useState(todayISO());
  const [draftTime, setDraftTime] = useState("");
  const saveTimer = useRef<number | null>(null);
  const draftRef = useRef(draft);
  draftRef.current = draft;

  const saveDraft = useCallback((value: string) => {
    if (saveTimer.current !== null) {
      window.clearTimeout(saveTimer.current);
    }
    saveTimer.current = window.setTimeout(() => {
      invoke("save_draft", { draft: value }).catch(() => undefined);
    }, 150);
  }, []);

  const refresh = useCallback(async () => {
    try {
      const next = await api.listBoard();
      setBoard(next);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
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
      })
      .catch(() => undefined);

    refresh();

    return () => {
      active = false;
    };
  }, [refresh]);

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

  const run = useCallback(
    async (fn: () => Promise<unknown>) => {
      setBusy(true);
      try {
        await fn();
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [refresh]
  );

  const addTask = () => {
    const title = draft.trim();
    if (!title) {
      setError("请输入任务内容");
      return;
    }
    const id = api.newId();
    run(() =>
      api.createTask({
        id,
        title,
        date: draftDate || undefined,
        time: draftTime || undefined,
        all_day: false,
        priority: "NORMAL",
      })
    ).then(() => {
      setDraft("");
      saveDraft("");
    });
  };

  const handlers = {
    onComplete: (view: TaskView) => run(() => api.completeTask(view.task.id, view.task.version)),
    onUndo: (view: TaskView) => run(() => api.undoCompletion(view.task.id, view.task.version)),
    onCancel: (view: TaskView) => run(() => api.cancelTask(view.task.id, view.task.version)),
    onRestore: (view: TaskView) =>
      run(() => api.restoreCanceledTask(view.task.id, view.task.version)),
    onDelete: (view: TaskView) => run(() => api.softDeleteTask(view.task.id, view.task.version)),
    onReschedule: (view: TaskView, date: string, time?: string) =>
      run(() =>
        api.rescheduleTask({ id: view.task.id, expected_version: view.task.version, date, time })
      ),
    onUpdate: (view: TaskView, title: string, priority: string) =>
      run(() =>
        api.updateTask({
          id: view.task.id,
          expected_version: view.task.version,
          title,
          priority,
        })
      ),
  };

  const statusText = error ? `出错了：${error}` : loading ? "正在加载任务…" : "任务列表已就绪";

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

      <form
        className="quick-add"
        onSubmit={(event) => {
          event.preventDefault();
          addTask();
        }}
      >
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
        <div className="quick-add-fields">
          <label className="quick-add-field">
            <span>日期</span>
            <input
              type="date"
              value={draftDate}
              onChange={(event) => setDraftDate(event.target.value)}
            />
          </label>
          <label className="quick-add-field">
            <span>时间</span>
            <input
              type="time"
              value={draftTime}
              onChange={(event) => setDraftTime(event.target.value)}
            />
          </label>
          <button type="submit" className="quick-add-submit" disabled={busy}>
            添加
          </button>
        </div>
      </form>

      {loading ? (
        <p className="board-loading" data-testid="board-loading">
          正在加载任务…
        </p>
      ) : error && !board ? (
        <div className="board-error" role="alert" data-testid="board-error">
          <p>无法加载任务列表。</p>
          <p className="board-error-detail">{error}</p>
          <button type="button" onClick={() => refresh()}>
            重试
          </button>
        </div>
      ) : board ? (
        <div className="board">
          <Section
            id="today-timed"
            title="今日 · 定时"
            views={board.today.timed}
            empty="今天还没有定时任务。"
            busy={busy}
            handlers={handlers}
          />
          <Section
            id="today-untimed"
            title="今日 · 待安排"
            views={board.today.untimed}
            empty="今天还没有待安排的任务。"
            busy={busy}
            handlers={handlers}
          />
          <Section
            id="legacy"
            title="遗留"
            views={board.legacy}
            empty="没有遗留任务。"
            busy={busy}
            handlers={handlers}
          />
          <Section
            id="future"
            title="未来"
            views={board.future}
            empty="还没有安排到未来的任务。"
            busy={busy}
            handlers={handlers}
          />
        </div>
      ) : null}

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

      <p className="panel-status" role="status" aria-live="polite" data-testid="app-status">
        {statusText}
      </p>
    </main>
  );
}
