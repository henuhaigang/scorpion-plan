// Types mirror the Rust serialization in `src-tauri/src/domain` and `app`.
// The UI only renders these; it never recomputes bucketing, sorting, or overdue.

export type TaskStatus = "TODO" | "COMPLETED" | "CANCELED";

export type TaskPriority = "LOW" | "NORMAL" | "HIGH";

export type TaskBucket = "TODAY" | "FUTURE" | "LEGACY" | "COMPLETED" | "CANCELED" | "DELETED";

export interface TaskSchedule {
  /** Calendar date, `YYYY-MM-DD`. */
  date: string;
  /** Wall-clock time `HH:MM:SS`, or `null` when the task has no specific time. */
  time: string | null;
  all_day: boolean;
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  original_schedule: TaskSchedule;
  current_schedule: TaskSchedule;
  priority: TaskPriority;
  category_id: string | null;
  reminder_preference: unknown;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  deleted_at: string | null;
  version: number;
}

export interface TaskView {
  task: Task;
  bucket: TaskBucket;
  is_overdue: boolean;
}

export interface TodayView {
  timed: TaskView[];
  untimed: TaskView[];
}

export interface TaskBoard {
  today: TodayView;
  legacy: TaskView[];
  future: TaskView[];
}
