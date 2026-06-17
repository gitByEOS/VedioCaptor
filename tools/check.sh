#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

WORKFLOW_FILE=".github/workflows/release.yml"

reject_workflow_pattern() {
  local pattern="$1"
  local message="$2"

  if grep -Eq "$pattern" "$WORKFLOW_FILE"; then
    echo "$message" >&2
    exit 1
  fi
}

require_workflow_pattern() {
  local pattern="$1"
  local message="$2"

  if ! grep -Eq "$pattern" "$WORKFLOW_FILE"; then
    echo "$message" >&2
    exit 1
  fi
}

reject_workflow_pattern 'uses: actions/checkout@v[0-5]($|[^0-9])' "checkout action must support Node.js 24"
reject_workflow_pattern 'uses: actions/setup-node@v[0-5]($|[^0-9])' "setup-node action must support Node.js 24"
reject_workflow_pattern 'uses: softprops/action-gh-release@v[0-2]($|[^0-9])' "release action must support Node.js 24"
reject_workflow_pattern 'uses: tauri-apps/tauri-action@v1($|[^0-9])' "tauri-action v1 does not exist"
require_workflow_pattern 'uses: tauri-apps/tauri-action@v0\.6\.2' "workflow must pin tauri-action to v0.6.2"
reject_workflow_pattern 'node-version: 20($|[^0-9])' "workflow must build with Node.js 24"
reject_workflow_pattern 'universal-apple-darwin' "workflow must not build universal macOS artifacts"
reject_workflow_pattern 'macos-latest|windows-latest' "workflow must pin release runner versions"
reject_workflow_pattern 'libwebkit2gtk-4\.0-dev' "Tauri v2 on Ubuntu needs libwebkit2gtk-4.1-dev"
reject_workflow_pattern 'run: npm install($|[[:space:]])' "workflow must use npm ci"

npm ci
npm run build
GIT_CONFIG_GLOBAL=/dev/null cargo check --manifest-path src-tauri/Cargo.toml --locked
