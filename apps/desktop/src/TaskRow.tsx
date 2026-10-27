import { useState } from "react";
import type { TaskPriority, TaskSchedule, TaskView } from "./types";

export interface TaskRowProps {
  view: TaskView;
  busy: boolean;
  onComplete: (view: TaskView) => void;
  onUndo: (view: TaskView) => void;
  onCancel: (view: TaskView) => void;
  onRestore: (view: TaskView) => void;
  onDelete: (view: TaskView) => void;
  onReschedule: (view: TaskView, date: string, time?: string) => void;
  onUpdate: (view: TaskView, title: string, priority: string) => void;
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
  onCancel,
  onRestore,
  onDelete,
  onReschedule,
  onUpdate,
}: TaskRowProps) {
  const { task, is_overdue } = view;
  const [editing, setEditing] = useState(false);
  const [date, setDate] = useState(task.current_schedule.date);
  const [time, setTime] = useState(task.current_schedule.time?.slice(0, 5) ?? "");

  const [editOpen, setEditOpen] = useState(false);
  const [editTitle, setEditTitle] = useState(task.title);
  const [editPriority, setEditPriority] = useState(task.priority);

  const isTodo = task.status === "TODO";
  const isCompleted = task.status === "COMPLETED";
  const isCanceled = task.status === "CANCELED";

  const submitReschedule = (clearTime: boolean) => {
    onReschedule(view, date, clearTime ? "" : time || undefined);
    setEditing(false);
  };

  const submitEdit = () => {
    onUpdate(view, editTitle, editPriority);
    setEditOpen(false);
  };

  return (
    <li
      className={`task-row task-row--${task.status.toLowerCase()}${is_overdue ? " task-row--overdue" : ""}`}
      data-testid={`row-${task.id}`}
    >
      <div className="task-row-main">
        <button
          type="button"
          className="task-toggle"
          data-testid={`toggle-${task.id}`}
          disabled={busy}
          aria-pressed={isCompleted}
          aria-label={isCompleted ? `撤销完成：${task.title}` : `完成：${task.title}`}
          onClick={() => (isCompleted ? onUndo(view) : onComplete(view))}
        >
          {isCompleted ? "✓" : ""}
        </button>

        <div className="task-body">
          <p className="task-title">{task.title}</p>
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
          </p>
        </div>
      </div>

      <div className="task-actions">
        {isTodo && (
          <>
            <button type="button" disabled={busy} onClick={() => onComplete(view)}>
              完成
            </button>
            <button type="button" disabled={busy} onClick={() => onCancel(view)}>
              取消
            </button>
            <button type="button" disabled={busy} onClick={() => setEditing((v) => !v)}>
              改期
            </button>
            <button type="button" disabled={busy} onClick={() => setEditOpen((v) => !v)}>
              编辑
            </button>
          </>
        )}
        {isCompleted && (
          <button type="button" disabled={busy} onClick={() => onUndo(view)}>
            撤销
          </button>
        )}
        {isCanceled && (
          <button type="button" disabled={busy} onClick={() => onRestore(view)}>
            恢复
          </button>
        )}
        <button
          type="button"
          className="task-delete"
          disabled={busy}
          onClick={() => onDelete(view)}
        >
          删除
        </button>
      </div>

      {editing && isTodo && (
        <form
          className="task-reschedule"
          onSubmit={(event) => {
            event.preventDefault();
            submitReschedule(false);
          }}
        >
          <label className="reschedule-field">
            <span>日期</span>
            <input
              type="date"
              value={date}
              onChange={(event) => setDate(event.target.value)}
              required
            />
          </label>
          <label className="reschedule-field">
            <span>时间</span>
            <input type="time" value={time} onChange={(event) => setTime(event.target.value)} />
          </label>
          <div className="reschedule-actions">
            <button type="submit" disabled={busy}>
              确定
            </button>
            <button type="button" disabled={busy} onClick={() => submitReschedule(true)}>
              清除时间
            </button>
            <button type="button" onClick={() => setEditing(false)}>
              收起
            </button>
          </div>
        </form>
      )}

      {editOpen && isTodo && (
        <form
          className="task-edit"
          onSubmit={(event) => {
            event.preventDefault();
            submitEdit();
          }}
        >
          <label className="reschedule-field">
            <span>标题</span>
            <input
              type="text"
              value={editTitle}
              onChange={(event) => setEditTitle(event.target.value)}
              required
              maxLength={200}
            />
          </label>
          <label className="reschedule-field">
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
          <div className="reschedule-actions">
            <button type="submit" disabled={busy}>
              保存
            </button>
            <button type="button" onClick={() => setEditOpen(false)}>
              收起
            </button>
          </div>
        </form>
      )}
    </li>
  );
}
