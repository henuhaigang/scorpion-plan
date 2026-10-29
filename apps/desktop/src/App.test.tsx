import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { Task, TaskBoard, TaskBucket, TaskView } from "./types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

let board: TaskBoard = {
  today: { timed: [], untimed: [] },
  legacy: [],
  future: [],
};

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: "11111111-1111-4111-8111-111111111111",
    title: "写周报",
    status: "TODO",
    original_schedule: { date: "2026-10-26", time: null, all_day: false },
    current_schedule: { date: "2026-10-26", time: null, all_day: false },
    priority: "NORMAL",
    category_id: null,
    reminder_preference: { INHERIT_DEFAULT: null },
    created_at: "2026-10-26T04:00:00Z",
    updated_at: "2026-10-26T04:00:00Z",
    completed_at: null,
    deleted_at: null,
    version: 1,
    ...overrides,
  };
}

function makeView(task: Task, bucket: TaskBucket = "TODAY", is_overdue = false): TaskView {
  return { task, bucket, is_overdue };
}

function boardWith(untimed: TaskView[]): TaskBoard {
  return { today: { timed: [], untimed }, legacy: [], future: [] };
}

beforeEach(() => {
  vi.clearAllMocks();
  board = { today: { timed: [], untimed: [] }, legacy: [], future: [] };
  mockedInvoke.mockImplementation((command: string, args?: unknown) => {
    const a = args as Record<string, unknown> | undefined;
    if (command === "load_draft") {
      return Promise.resolve("保存的草稿");
    }
    if (command === "panel_state") {
      return Promise.resolve("visible");
    }
    if (command === "list_board") {
      return Promise.resolve(board);
    }
    if (command === "update_task") {
      return Promise.resolve(
        makeTask({
          id: a?.id as string,
          title: a?.title as string,
          priority: a?.priority as Task["priority"],
          version: (a?.expectedVersion as number) + 1,
        })
      );
    }
    if (command === "reschedule_task") {
      const time = a?.time;
      return Promise.resolve(
        makeTask({
          id: a?.id as string,
          current_schedule: {
            date: a?.date as string,
            time: time === "" ? null : (time as string | null),
            all_day: false,
          },
          version: (a?.expectedVersion as number) + 1,
        })
      );
    }
    return Promise.resolve();
  });
});

afterEach(() => {
  cleanup();
});

describe("App shell", () => {
  it("renders the DayFlow title", async () => {
    render(<App />);
    expect(await screen.findByRole("heading", { name: "DayFlow" })).toBeInTheDocument();
  });

  it("shows the product promise", async () => {
    render(<App />);
    expect(await screen.findByText("让重要的事情，主动出现在你面前。")).toBeInTheDocument();
  });

  it("loads the persisted draft", async () => {
    render(<App />);
    fireEvent.click(await screen.findByTestId("add-btn"));
    expect(await screen.findByDisplayValue("保存的草稿")).toBeInTheDocument();
  });

  it("saves draft changes through the local service", async () => {
    render(<App />);
    fireEvent.click(await screen.findByTestId("add-btn"));
    const input = await screen.findByTestId("draft-input");

    fireEvent.change(input, { target: { value: "新草稿" } });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("save_draft", { draft: "新草稿" });
    });
  });

  it("hides the panel when Escape is pressed", async () => {
    render(<App />);
    await screen.findByTestId("add-btn");

    fireEvent.keyDown(window, { key: "Escape" });

    expect(mockedInvoke).toHaveBeenCalledWith("panel_hide");
  });
});

describe("Board rendering", () => {
  it("renders all four sections", async () => {
    render(<App />);
    expect(await screen.findByRole("heading", { name: "今日 · 定时" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "今日 · 待安排" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "遗留" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "未来" })).toBeInTheDocument();
  });

  it("renders tasks in the untimed section", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1", title: "待办一" }))]);
    render(<App />);
    expect(await screen.findByText("待办一")).toBeInTheDocument();
  });

  it("shows an empty state for an empty section", async () => {
    render(<App />);
    expect(await screen.findByText("今天还没有定时任务。")).toBeInTheDocument();
  });

  it("shows an overdue badge for overdue tasks", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1" }), "TODAY", true)]);
    render(<App />);
    expect(await screen.findByTestId("overdue-t-1")).toHaveTextContent("已逾期");
  });

  it("shows a loading state before the board resolves", () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_board") {
        return new Promise(() => undefined);
      }
      return Promise.resolve();
    });
    render(<App />);
    expect(screen.getByTestId("board-loading")).toBeInTheDocument();
  });

  it("shows an error state with retry when the board fails", async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_board") {
        return Promise.reject(new Error("数据库读取失败"));
      }
      return Promise.resolve();
    });
    render(<App />);
    const errorBox = await screen.findByTestId("board-error");
    expect(within(errorBox).getByText("数据库读取失败")).toBeInTheDocument();

    const listBoardCalls = () =>
      mockedInvoke.mock.calls.filter(([cmd]) => cmd === "list_board").length;
    const before = listBoardCalls();

    fireEvent.click(within(errorBox).getByRole("button", { name: "重试" }));
    await waitFor(() => {
      expect(listBoardCalls()).toBeGreaterThan(before);
    });
  });
});

describe("Quick add", () => {
  it("creates a task with a client-generated UUID", async () => {
    render(<App />);
    fireEvent.click(await screen.findByTestId("add-btn"));
    const input = await screen.findByTestId("draft-input");
    fireEvent.change(input, { target: { value: "新任务" } });
    fireEvent.click(screen.getByRole("button", { name: "添加" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "create_task",
        expect.objectContaining({
          title: "新任务",
          id: expect.stringMatching(
            /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i
          ),
        })
      );
    });
  });

  it("rejects an empty quick add", async () => {
    render(<App />);
    fireEvent.click(await screen.findByTestId("add-btn"));
    const input = await screen.findByTestId("draft-input");
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByRole("button", { name: "添加" }));

    await waitFor(() => {
      expect(screen.getByTestId("app-status")).toHaveTextContent("请输入任务内容");
    });
    expect(mockedInvoke).not.toHaveBeenCalledWith("create_task", expect.anything());
  });
});

describe("Lifecycle actions", () => {
  it("completes a task via the toggle control", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1" }))]);
    render(<App />);
    fireEvent.click(await screen.findByTestId("toggle-t-1"));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("complete_task", {
        id: "t-1",
        expectedVersion: 1,
      });
    });
  });

  it("undoes completion for a completed task", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1", status: "COMPLETED", version: 2 }))]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "撤销" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("undo_completion", {
        id: "t-1",
        expectedVersion: 2,
      });
    });
  });

  it("restores a canceled task", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1", status: "CANCELED", version: 2 }))]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "恢复" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("restore_canceled_task", {
        id: "t-1",
        expectedVersion: 2,
      });
    });
  });

  it("soft-deletes a task", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1" }))]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "删除" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("soft_delete_task", {
        id: "t-1",
        expectedVersion: 1,
      });
    });
  });
});

describe("Edit (combined form)", () => {
  it("edits a task's title and priority", async () => {
    board = boardWith([makeView(makeTask({ id: "t-1" }))]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "编辑" }));

    const row = screen.getByTestId("row-t-1");
    fireEvent.change(within(row).getByPlaceholderText("任务标题"), { target: { value: "新标题" } });
    fireEvent.change(within(row).getByLabelText("优先级"), { target: { value: "HIGH" } });
    fireEvent.click(within(row).getByRole("button", { name: "保存" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "update_task",
        expect.objectContaining({
          id: "t-1",
          expectedVersion: 1,
          title: "新标题",
          priority: "HIGH",
        })
      );
    });
  });

  it("reschedules to a new date via the edit form", async () => {
    board = boardWith([
      makeView(
        makeTask({
          id: "t-1",
          current_schedule: { date: "2026-10-26", time: "09:00:00", all_day: false },
        })
      ),
    ]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "编辑" }));

    const row = screen.getByTestId("row-t-1");
    const dateInput = within(row).getByLabelText("日期");
    fireEvent.change(dateInput, { target: { value: "2026-11-02" } });
    fireEvent.click(within(row).getByRole("button", { name: "保存" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "reschedule_task",
        expect.objectContaining({
          id: "t-1",
          expectedVersion: 1,
          date: "2026-11-02",
          time: "09:00",
        })
      );
    });
  });

  it("clears the task time when the edit form time field is emptied", async () => {
    board = boardWith([
      makeView(
        makeTask({
          id: "t-1",
          current_schedule: { date: "2026-10-26", time: "09:00:00", all_day: false },
        })
      ),
    ]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "编辑" }));

    const row = screen.getByTestId("row-t-1");
    const timeInput = within(row).getByLabelText("时间");
    fireEvent.change(timeInput, { target: { value: "" } });
    fireEvent.click(within(row).getByRole("button", { name: "保存" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "reschedule_task",
        expect.objectContaining({
          id: "t-1",
          expectedVersion: 1,
          date: "2026-10-26",
          time: "",
        })
      );
    });
  });

  it("updates title first, then reschedules using the updated version", async () => {
    board = boardWith([
      makeView(
        makeTask({
          id: "t-1",
          current_schedule: { date: "2026-10-26", time: null, all_day: false },
        })
      ),
    ]);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "编辑" }));

    const row = screen.getByTestId("row-t-1");
    fireEvent.change(within(row).getByPlaceholderText("任务标题"), { target: { value: "新标题" } });
    fireEvent.change(within(row).getByLabelText("日期"), { target: { value: "2026-11-02" } });
    fireEvent.click(within(row).getByRole("button", { name: "保存" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "update_task",
        expect.objectContaining({
          id: "t-1",
          expectedVersion: 1,
          title: "新标题",
        })
      );
      expect(mockedInvoke).toHaveBeenCalledWith(
        "reschedule_task",
        expect.objectContaining({
          id: "t-1",
          expectedVersion: 2,
          date: "2026-11-02",
          time: "",
        })
      );
    });
  });
});
