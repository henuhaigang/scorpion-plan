import { invoke } from "@tauri-apps/api/core";
import type { Task, TaskBoard } from "./types";

// Type aliases (not interfaces) so they carry an implicit index signature and
// satisfy Tauri's `InvokeArgs = Record<string, unknown>`.
export type CreateTaskArgs = {
  id: string;
  title: string;
  date?: string;
  time?: string;
  allDay?: boolean;
  priority?: string;
  categoryId?: string;
};

export type UpdateTaskArgs = {
  id: string;
  expectedVersion: number;
  title?: string;
  priority?: string;
  categoryId?: string;
};

export type RescheduleArgs = {
  id: string;
  expectedVersion: number;
  date: string;
  /** `undefined` keeps the current time; `""` clears it; `"HH:MM"` sets it. */
  time?: string;
};

export function listBoard(): Promise<TaskBoard> {
  return invoke<TaskBoard>("list_board");
}

export function createTask(args: CreateTaskArgs): Promise<Task> {
  return invoke<Task>("create_task", args);
}

export function updateTask(args: UpdateTaskArgs): Promise<Task> {
  return invoke<Task>("update_task", args);
}

export function rescheduleTask(args: RescheduleArgs): Promise<Task> {
  return invoke<Task>("reschedule_task", args);
}

export function completeTask(id: string, expectedVersion: number): Promise<Task> {
  return invoke<Task>("complete_task", { id, expectedVersion });
}

export function undoCompletion(id: string, expectedVersion: number): Promise<Task> {
  return invoke<Task>("undo_completion", { id, expectedVersion });
}

export function cancelTask(id: string, expectedVersion: number): Promise<Task> {
  return invoke<Task>("cancel_task", { id, expectedVersion });
}

export function restoreCanceledTask(id: string, expectedVersion: number): Promise<Task> {
  return invoke<Task>("restore_canceled_task", { id, expectedVersion });
}

export function softDeleteTask(id: string, expectedVersion: number): Promise<Task> {
  return invoke<Task>("soft_delete_task", { id, expectedVersion });
}

/**
 * Generate a client-side UUID so a double submission maps to the same id and
 * the backend can deduplicate. Falls back to a hand-rolled v4 when
 * `crypto.randomUUID` is unavailable.
 */
export function newId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i += 1) {
    bytes[i] = Math.floor(Math.random() * 256);
  }
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(
    16,
    20
  )}-${hex.slice(20)}`;
}
