# DayFlow — Agent Handoff

DayFlow is a local-first personal task and schedule app for macOS, Windows, iPhone, and Android.

**This repository is a specification and agent handoff pack, not a completed runnable app.** Use it as the source of truth for your coding agent to create the actual monorepo, implement features, run tests, and report evidence.

## Product principle
“让重要的事情，主动出现在你面前。”

## Start with your coding agent
1. Extract this directory into a new, empty Git repository.
2. Open the repository root in Claude Code, Codex, or your chosen coding agent.
3. Ask the agent to read `AGENTS.md` completely and follow `docs/IMPLEMENTATION_PLAN.md`.
4. Require the agent to inspect the local environment and repository before changing files.
5. Ask it to implement one milestone at a time, run tests, and report exact commands/results. Do not accept claims of tests passing without actual command output.

## Source of truth
- Product requirements: `docs/PRD.md`
- Domain model: `docs/DOMAIN_MODEL.md`
- Data and sync: `docs/DATA_AND_SYNC.md`
- Reminders: `docs/REMINDER_RULES.md`
- Architecture: `docs/ARCHITECTURE.md`
- Acceptance tests: `docs/ACCEPTANCE_TESTS.md`
- Implementation order: `docs/IMPLEMENTATION_PLAN.md`
- Technical decisions: `docs/adr/`

## Recommended product delivery
- V1: desktop-first local core, with the domain and storage designed for sync.
- V1.1: account and multi-device synchronization.
- V1.2: advanced task workflows, recurrence and optional AI features.

Do not attempt to ship all four platforms simultaneously. Build shared domain contracts first, then desktop, then mobile, then cloud sync.
