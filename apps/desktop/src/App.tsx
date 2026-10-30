import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as api from "./api";
import { TaskRow } from "./TaskRow";
import type { HistoryView, Task, TaskBoard, TaskView } from "./types";

type MainView = "board" | "history";

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
    onComplete: (view: TaskView) => Promise<unknown>;
    onUndo: (view: TaskView) => Promise<unknown>;
    onRestore: (view: TaskView) => Promise<unknown>;
    onDelete: (view: TaskView) => Promise<unknown>;
    onReschedule: (view: TaskView, date: string, time?: string) => Promise<Task>;
    onUpdate: (view: TaskView, title: string, priority: string) => Promise<Task>;
  };
}

function Section({ id, title, views, empty, busy, handlers }: SectionProps) {
  return (
    <section className="board-section" aria-labelledby={`${id}-title`}>
      <div className="board-section-header">
        <h2 id={`${id}-title`} className="board-section-title">
          {title}
        </h2>
        <span className="board-section-count" aria-label={`${views.length} 项任务`}>
          {views.length}
        </span>
      </div>
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
  const [history, setHistory] = useState<HistoryView | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [mainView, setMainView] = useState<MainView>("board");
  const [showQuickAdd, setShowQuickAdd] = useState(false);

  const [draft, setDraft] = useState("");
  const [draftDate, setDraftDate] = useState(todayISO());
  const [draftTime, setDraftTime] = useState("");
  const [draftPriority, setDraftPriority] = useState("NORMAL");
  const saveTimer = useRef<number | null>(null);
  const draftRef = useRef(draft);
  draftRef.current = draft;
  const quickAddRef = useRef<HTMLTextAreaElement>(null);

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

  const refreshHistory = useCallback(async () => {
    try {
      const next = await api.listHistory();
      setHistory(next);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;

    invoke<string>("load_draft")
      .then((savedDraft) => {
        if (!active) {
          return;
        }
        setDraft(savedDraft);
      })
      .catch(() => undefined);

    refresh();

    return () => {
      active = false;
    };
  }, [refresh]);

  useEffect(() => {
    if (mainView === "history" && !history) {
      refreshHistory();
    }
  }, [mainView, history, refreshHistory]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (showQuickAdd) {
          setShowQuickAdd(false);
        } else {
          event.preventDefault();
          invoke("panel_hide").catch(() => undefined);
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [showQuickAdd]);

  useEffect(() => {
    if (showQuickAdd) {
      quickAddRef.current?.focus();
    }
  }, [showQuickAdd]);

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
    async function run<T>(fn: () => Promise<T>): Promise<T> {
      setBusy(true);
      try {
        const result = await fn();
        await Promise.all([refresh(), refreshHistory()]);
        return result;
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        throw err;
      } finally {
        setBusy(false);
      }
    },
    [refresh, refreshHistory]
  );

  const addTask = async () => {
    const title = draft.trim();
    if (!title) {
      setError("请输入任务内容");
      return;
    }
    const id = api.newId();
    try {
      await run(() =>
        api.createTask({
          id,
          title,
          date: draftDate || undefined,
          time: draftTime || undefined,
          allDay: false,
          priority: draftPriority,
        })
      );
      setDraft("");
      saveDraft("");
      setShowQuickAdd(false);
    } catch {
      return;
    }
  };

  const handlers = {
    onComplete: (view: TaskView) => run(() => api.completeTask(view.task.id, view.task.version)),
    onUndo: (view: TaskView) =>
      run(() => api.undoCompletion(view.task.id, view.task.version)).then(() => {
        setMainView("board");
      }),
    onRestore: (view: TaskView) =>
      run(() => api.restoreCanceledTask(view.task.id, view.task.version)).then(() => {
        setMainView("board");
      }),
    onDelete: (view: TaskView) => run(() => api.softDeleteTask(view.task.id, view.task.version)),
    onReschedule: (view: TaskView, date: string, time?: string) =>
      run(() =>
        api.rescheduleTask({ id: view.task.id, expectedVersion: view.task.version, date, time })
      ),
    onUpdate: (view: TaskView, title: string, priority: string) =>
      run(() =>
        api.updateTask({
          id: view.task.id,
          expectedVersion: view.task.version,
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
          <p className="panel-subtitle">今天，轻松安排</p>
        </div>
        <button
          type="button"
          className="add-btn"
          data-testid="add-btn"
          onClick={() => setShowQuickAdd((v) => !v)}
        >
          {showQuickAdd ? "收起" : "+ 添加"}
        </button>
      </header>

      {showQuickAdd && (
        <form
          className="quick-add"
          onSubmit={(event) => {
            event.preventDefault();
            void addTask();
          }}
        >
          <textarea
            ref={quickAddRef}
            className="draft-input"
            value={draft}
            onChange={(event) => updateDraft(event.target.value)}
            placeholder="输入任务内容"
            data-testid="draft-input"
            rows={2}
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
                step={60}
                value={draftTime}
                onChange={(event) => setDraftTime(event.target.value)}
              />
            </label>
            <label className="quick-add-field">
              <span>优先级</span>
              <select
                value={draftPriority}
                onChange={(event) => setDraftPriority(event.target.value)}
              >
                <option value="LOW">低</option>
                <option value="NORMAL">普通</option>
                <option value="HIGH">高</option>
              </select>
            </label>
            <button type="submit" className="quick-add-submit" disabled={busy}>
              添加
            </button>
          </div>
        </form>
      )}

      {loading ? (
        <p className="board-loading" data-testid="board-loading">
          正在加载任务…
        </p>
      ) : error && !board && !history ? (
        <div className="board-error" role="alert" data-testid="board-error">
          <p>无法加载任务列表。</p>
          <p className="board-error-detail">{error}</p>
          <button
            type="button"
            onClick={() => (mainView === "board" ? refresh() : refreshHistory())}
          >
            重试
          </button>
        </div>
      ) : mainView === "board" ? (
        board && (
          <div className="board">
            <Section
              id="today-timed"
              title="今日"
              views={board.today.timed}
              empty="今天还没有任务。"
              busy={busy}
              handlers={handlers}
            />
            <Section
              id="today-untimed"
              title="待安排"
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
        )
      ) : (
        <div className="history-view">
          <div className="history-header">
            <button type="button" onClick={() => setMainView("board")}>
              ← 返回
            </button>
            <h2>历史</h2>
          </div>
          {history && history.items.length === 0 ? (
            <p className="board-empty">暂无历史记录。</p>
          ) : (
            history && (
              <ul className="task-list">
                {history.items.map((view) => (
                  <TaskRow key={view.task.id} view={view} busy={busy} {...handlers} />
                ))}
              </ul>
            )
          )}
        </div>
      )}

      <div className="panel-actions">
        <button
          type="button"
          onClick={() => setMainView(mainView === "board" ? "history" : "board")}
        >
          {mainView === "board" ? "历史" : "任务"}
        </button>
        <button type="button" onClick={() => invoke("panel_hide").catch(() => undefined)}>
          隐藏面板
        </button>
        <button type="button" onClick={() => invoke("panel_quit").catch(() => undefined)}>
          退出
        </button>
      </div>

      {(error || loading) && (
        <p className="panel-status" role="status" aria-live="polite" data-testid="app-status">
          {statusText}
        </p>
      )}
    </main>
  );
}
