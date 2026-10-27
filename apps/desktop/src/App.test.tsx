import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockImplementation((command: string) => {
    if (command === "load_draft") {
      return Promise.resolve("保存的草稿");
    }
    if (command === "panel_state") {
      return Promise.resolve("visible");
    }
    return Promise.resolve();
  });
});

afterEach(() => {
  cleanup();
});

describe("App", () => {
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
    expect(await screen.findByDisplayValue("保存的草稿")).toBeInTheDocument();
  });

  it("saves draft changes through the local service", async () => {
    render(<App />);
    const input = await screen.findByTestId("draft-input");

    fireEvent.change(input, { target: { value: "新草稿" } });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("save_draft", {
        draft: "新草稿",
      });
    });
  });

  it("hides the panel when Escape is pressed", async () => {
    render(<App />);
    await screen.findByTestId("draft-input");

    fireEvent.keyDown(window, { key: "Escape" });

    expect(mockedInvoke).toHaveBeenCalledWith("panel_hide");
  });
});
