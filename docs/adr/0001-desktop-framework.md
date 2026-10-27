# ADR 0001: Desktop Framework

- Status: Proposed
- Decision candidate: Tauri 2 + React + TypeScript
- Alternative: Electron + React + TypeScript

## Context
DayFlow's defining desktop feature is a compact tray/menu-bar quick panel with reliable toggle, blur-hide, draft preservation, multi-monitor positioning, and no accidental app exit.

## Decision
Start with Tauri 2 because the product values a lightweight desktop footprint and the team can use React/TypeScript for UI while isolating native behavior in Rust. This is a candidate decision, not a claim of proven platform parity.

## Milestone evidence
- 2026-10-26 (M1): macOS aarch64 packaged app builds via `pnpm run ci`; Rust domain, SQLite persistence, and application use cases are implemented. Tray/menu-bar runtime behavior remains unverified and is still required before release.
- 2026-10-27 (M2): macOS aarch64 packaged app builds via `pnpm run ci`; tray icon, panel show/hide/toggle, close-to-hide, blur-hide state coordination, draft persistence commands, and the React quick panel are implemented. The packaged app was launched locally as `DayFlow.app`; visual multi-display and Windows tray behavior still require platform-specific verification before release.

## Required evidence before release
- macOS packaged app: menu-bar icon, toggle, blur-hide, focus, screen edge positioning, multi-display.
- Windows packaged app: tray icon, toggle, blur-hide, taskbar/display scaling, multi-display.
- Repeated clicks and blur/tray race tests.
- Startup, explicit quit and draft preservation.
- Compare actual defects and implementation cost against Electron if a required behavior is unreliable.

## Reversal criteria
If a core requirement cannot be implemented reliably with supported Tauri APIs without brittle platform-specific workarounds, create an ADR comparing Electron before proceeding further.
