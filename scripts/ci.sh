#!/usr/bin/env bash
# DayFlow CI pipeline: format -> lint -> test -> build.
# Runs the same checks as .github/workflows/ci.yml.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

step() {
  echo ""
  echo "==> $1"
}

step "0. Toolchain versions"
node --version
pnpm --version
rustc --version
cargo --version

step "1. Install dependencies"
pnpm install --frozen-lockfile

step "2. Format check (prettier + rustfmt)"
pnpm format:check
pnpm --filter dayflow-desktop format:rust:check

step "3. Lint (eslint + clippy)"
pnpm lint

step "4. Typecheck + unit tests (vitest + cargo test)"
pnpm --filter dayflow-desktop exec tsc --noEmit
pnpm test

step "5. Build (vite + tauri packaged app)"
pnpm build

echo ""
echo "CI: all checks passed"
