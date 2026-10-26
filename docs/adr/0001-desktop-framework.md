# ADR 0001: Desktop Framework

- Status: Proposed
- Decision candidate: Tauri 2 + React + TypeScript
- Alternative: Electron + React + TypeScript

## Context
DayFlow's defining desktop feature is a compact tray/menu-bar quick panel with reliable toggle, blur-hide, draft preservation, multi-monitor positioning, and no accidental app exit.

## Decision
Start with Tauri 2 because the product values a lightweight desktop footprint and the team can use React/TypeScript for UI while isolating native behavior in Rust. This is a candidate decision, not a claim of proven platform parity.

## Required evidence before release
- macOS packaged app: menu-bar icon, toggle, blur-hide, focus, screen edge positioning, multi-display.
- Windows packaged app: tray icon, toggle, blur-hide, taskbar/display scaling, multi-display.
- Repeated clicks and blur/tray race tests.
- Startup, explicit quit and draft preservation.
- Compare actual defects and implementation cost against Electron if a required behavior is unreliable.

## Reversal criteria
If a core requirement cannot be implemented reliably with supported Tauri APIs without brittle platform-specific workarounds, create an ADR comparing Electron before proceeding further.
