import { useState } from "react";
import type { Task, TaskPriority, TaskSchedule, TaskView } from "./types";

export interface TaskRowProps {
  view: TaskView;
  busy: boolean;
  onComplete: (view: TaskView) => Promise<unknown>;
  onUndo: (view: TaskView) => Promise<unknown>;
  onRestore: (view: TaskView) => Promise<unknown>;
  onDelete: (view: TaskView) => Promise<unknown>;
  onReschedule: (view: TaskView, date: string, time?: string) => Promise<Task>;
  onUpdate: (view: TaskView, title: string, priority: string) => Promise<Task>;
}

function formatSchedule(schedule: TaskSchedule): string {
  const parts = [schedule.date];
  if (schedule.time) {
    parts.push(schedule.time.slice(0, 5));
  }
  if (schedule.all_day) {
    parts.push("全天");
  }
  return parts.join(" ");
}

function formatLocalTime(utcIso: string): string {
  const d = new Date(utcIso);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

const PRIORITY_LABEL: Record<string, string> = {
  LOW: "低",
  NORMAL: "普通",
  HIGH: "高",
};

export function TaskRow({
  view,
  busy,
  onComplete,
  onUndo,
  onRestore,
  onDelete,
  onReschedule,
  onUpdate,
}: TaskRowProps) {
  const { task, is_overdue } = view;
  const [editOpen, setEditOpen] = useState(false);
  const [editTitle, setEditTitle] = useState(task.title);
  const [editPriority, setEditPriority] = useState(task.priority);
  const [editDate, setEditDate] = useState(task.current_schedule.date);
  const [editTime, setEditTime] = useState(task.current_schedule.time?.slice(0, 5) ?? "");

  const isTodo = task.status === "TODO";
  const isCompleted = task.status === "COMPLETED";
  const isCanceled = task.status === "CANCELED";

  const safe = (promise: Promise<unknown>) => {
    void promise.catch(() => undefined);
  };

  const submitEdit = async () => {
    if (!editTitle.trim() || !editDate) {
      return;
    }
    const titleChanged = editTitle !== task.title;
    const priorityChanged = editPriority !== task.priority;
    const dateChanged = editDate !== task.current_schedule.date;
    const timeChanged = editTime !== (task.current_schedule.time?.slice(0, 5) ?? "");

    let currentView = view;
    try {
      if (titleChanged || priorityChanged) {
        const updated = await onUpdate(currentView, editTitle, editPriority);
        currentView = { ...currentView, task: updated };
      }
      if (dateChanged || timeChanged) {
        await onReschedule(currentView, editDate, editTime);
      }
    } catch {
      return;
    }
    setEditOpen(false);
  };

  const statusIcon = isCompleted ? "✓" : isCanceled ? "⊘" : "";
  const statusClass = isCompleted
    ? "task-status--completed"
    : isCanceled
      ? "task-status--canceled"
      : "task-status--todo";

  return (
    <li
      className={`task-row task-row--${task.status.toLowerCase()}${is_overdue ? " task-row--overdue" : ""}`}
      data-testid={`row-${task.id}`}
    >
      <div className="task-row-main">
        <button
          type="button"
          className={`task-toggle ${statusClass}`}
          data-testid={`toggle-${task.id}`}
          disabled={busy}
          aria-pressed={isCompleted}
          aria-label={isCompleted ? `撤销完成：${task.title}` : `完成：${task.title}`}
          onClick={() => safe(isCompleted ? onUndo(view) : onComplete(view))}
        >
          {statusIcon}
        </button>

        <div className="task-body">
          <p className={`task-title ${isCompleted || isCanceled ? "task-title--done" : ""}`}>
            {task.title}
          </p>
          <p className="task-meta">
            <span className="task-schedule">{formatSchedule(task.current_schedule)}</span>
            <span className={`task-priority task-priority--${task.priority.toLowerCase()}`}>
              {PRIORITY_LABEL[task.priority] ?? task.priority}
            </span>
            {is_overdue && (
              <span className="task-overdue" data-testid={`overdue-${task.id}`}>
                已逾期
              </span>
            )}
            {isCompleted && task.completed_at && (
              <span className="task-completed-at">完成于 {formatLocalTime(task.completed_at)}</span>
            )}
          </p>
        </div>
      </div>

      <div className="task-actions">
        {isTodo && (
          <button type="button" disabled={busy} onClick={() => setEditOpen((v) => !v)}>
            编辑
          </button>
        )}
        {isCompleted && (
          <button type="button" disabled={busy} onClick={() => safe(onUndo(view))}>
            撤销
          </button>
        )}
        {isCanceled && (
          <button type="button" disabled={busy} onClick={() => safe(onRestore(view))}>
            恢复
          </button>
        )}
        <button
          type="button"
          className="task-delete"
          disabled={busy}
          onClick={() => safe(onDelete(view))}
        >
          删除
        </button>
      </div>

      {editOpen && isTodo && (
        <form
          className="task-edit"
          onSubmit={(event) => {
            event.preventDefault();
            void submitEdit();
          }}
        >
          <textarea
            className="draft-input"
            value={editTitle}
            onChange={(event) => setEditTitle(event.target.value)}
            placeholder="任务标题"
            rows={2}
            maxLength={200}
          />
          <div className="quick-add-fields">
            <label className="quick-add-field">
              <span>优先级</span>
              <select
                value={editPriority}
                onChange={(event) => setEditPriority(event.target.value as TaskPriority)}
              >
                <option value="LOW">低</option>
                <option value="NORMAL">普通</option>
                <option value="HIGH">高</option>
              </select>
            </label>
            <label className="quick-add-field">
              <span>日期</span>
              <input
                type="date"
                value={editDate}
                onChange={(event) => setEditDate(event.target.value)}
              />
            </label>
            <label className="quick-add-field">
              <span>时间</span>
              <input
                type="time"
                value={editTime}
                onChange={(event) => setEditTime(event.target.value)}
              />
            </label>
            <button type="submit" className="quick-add-submit" disabled={busy}>
              保存
            </button>
          </div>
        </form>
      )}
    </li>
  );
}
