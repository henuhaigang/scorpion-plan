import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("renders the DayFlow title", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "DayFlow" })).toBeInTheDocument();
  });

  it("shows the product promise", () => {
    render(<App />);
    expect(screen.getByText("让重要的事情，主动出现在你面前。")).toBeInTheDocument();
  });
});
