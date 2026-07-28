---
name: "playwright"
description: "Use when the task requires automating a real browser from the terminal (navigation, form filling, snapshots, screenshots, data extraction, UI-flow debugging) via `playwright-cli` or the bundled wrapper script."
---


# Playwright CLI Skill

Drive a real browser from the terminal using `playwright-cli`. Prefer the bundled wrapper script so the CLI works even when it is not globally installed.
Treat this skill as CLI-first automation. Do not pivot to `@playwright/test` unless the user explicitly asks for test files.

## Container-first Playwright on this repo

For this Windows workspace, run Playwright from a Linux container by default. Do not depend on host-installed `playwright`, host Chrome, host Edge, or host `agent-browser` for screenshot verification.

Use the official Playwright container for browser checks:

```powershell
docker run --rm -t --ipc=host `
  -v "${PWD}:/work" `
  -v idealer-playwright-node-modules:/work/node_modules `
  -w /work `
  mcr.microsoft.com/playwright:v1.60.0-noble `
  bash -lc "corepack enable && pnpm install --frozen-lockfile && pnpm --filter @idealer/web test:e2e"
```

For a one-off screenshot script, still run it inside the same container image and write artifacts under `output/playwright/`:

```powershell
docker run --rm -t --ipc=host `
  -v "${PWD}:/work" `
  -v idealer-playwright-node-modules:/work/node_modules `
  -w /work `
  mcr.microsoft.com/playwright:v1.60.0-noble `
  bash -lc "corepack enable && pnpm install --frozen-lockfile && pnpm --filter @idealer/web exec playwright screenshot http://host.docker.internal:3000 output/playwright/container-check.png"
```

When testing services already running through `docker compose`, attach the Playwright container to the project network and target service DNS names such as `http://web:3000`, `http://admin:3001`, or `http://api:3002`:

```powershell
docker run --rm -t --ipc=host --network idealer-internal `
  -v "${PWD}:/work" `
  -v idealer-playwright-node-modules:/work/node_modules `
  -w /work `
  mcr.microsoft.com/playwright:v1.60.0-noble `
  bash -lc "corepack enable && pnpm install --frozen-lockfile && pnpm --filter @idealer/web exec playwright screenshot http://web:3000/play output/playwright/container-play.png"
```

Only use host-side `npx`, `agent-browser`, or explicit Windows browser paths when Docker is unavailable or the user explicitly asks for host-side verification.

## Agent-browser on this Windows workspace

In this Windows workspace, `agent-browser` may not be installed as a PATH-visible global command. That is not a blocker.

When browser verification asks for `agent-browser` and `Get-Command agent-browser` returns nothing, use the npm package through `npx`:

```powershell
npx -y agent-browser --help
npx -y agent-browser open http://127.0.0.1:3000
npx -y agent-browser snapshot -i
npx -y agent-browser screenshot output/playwright/agent-browser-check.png
```

If `agent-browser` reports that no Chrome/Edge headless browser exists in the default paths, that is also not the final blocker. Diagnose and repair in this order:

```powershell
npx -y agent-browser doctor
npx -y agent-browser install
```

If `agent-browser install` cannot reach the Chrome for Testing CDN, look for an existing Playwright browser cache and pass the executable path explicitly:

```powershell
$ChromePath = Get-ChildItem -Recurse -File "$env:LOCALAPPDATA\ms-playwright" -Include chrome.exe,msedge.exe,chromium.exe,headless_shell.exe |
  Sort-Object FullName -Descending |
  Select-Object -First 1 -ExpandProperty FullName

npx -y agent-browser --executable-path "$ChromePath" open http://127.0.0.1:3000
npx -y agent-browser --executable-path "$ChromePath" snapshot -i
npx -y agent-browser --executable-path "$ChromePath" screenshot output/playwright/agent-browser-check.png
npx -y agent-browser close
```

Only treat browser verification as blocked if `npx` is unavailable, `agent-browser install` cannot fetch a browser, no usable browser executable exists in the Playwright or system caches, and the Playwright wrapper flow below also fails. Do not record "agent-browser and playwright are not installed" or "no Chrome/Edge headless exists in the default paths" as the final verification result without first trying `npx -y agent-browser`, `agent-browser install`, and explicit `--executable-path`.

## Prerequisite check (required)

Before proposing commands, check whether `npx` is available (the wrapper depends on it):

```bash
command -v npx >/dev/null 2>&1
```

If it is not available, pause and ask the user to install Node.js/npm (which provides `npx`). Provide these steps verbatim:

```bash
# Verify Node/npm are installed
node --version
npm --version

# If missing, install Node.js/npm, then:
npm install -g @playwright/mcp@latest
playwright-cli --help
```

Once `npx` is present, proceed with the wrapper script. A global install of `playwright-cli` is optional.

## Skill path (set once)

```bash
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export PWCLI="$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh"
```

User-scoped skills install under `$CODEX_HOME/skills` (default: `~/.codex/skills`).

## Quick start

Use the wrapper script:

```bash
"$PWCLI" open https://playwright.dev --headed
"$PWCLI" snapshot
"$PWCLI" click e15
"$PWCLI" type "Playwright"
"$PWCLI" press Enter
"$PWCLI" screenshot
```

If the user prefers a global install, this is also valid:

```bash
npm install -g @playwright/mcp@latest
playwright-cli --help
```

## Core workflow

1. Open the page.
2. Snapshot to get stable element refs.
3. Interact using refs from the latest snapshot.
4. Re-snapshot after navigation or significant DOM changes.
5. Capture artifacts (screenshot, pdf, traces) when useful.

Minimal loop:

```bash
"$PWCLI" open https://example.com
"$PWCLI" snapshot
"$PWCLI" click e3
"$PWCLI" snapshot
```

## When to snapshot again

Snapshot again after:

- navigation
- clicking elements that change the UI substantially
- opening/closing modals or menus
- tab switches

Refs can go stale. When a command fails due to a missing ref, snapshot again.

## Recommended patterns

### Form fill and submit

```bash
"$PWCLI" open https://example.com/form
"$PWCLI" snapshot
"$PWCLI" fill e1 "user@example.com"
"$PWCLI" fill e2 "password123"
"$PWCLI" click e3
"$PWCLI" snapshot
```

### Debug a UI flow with traces

```bash
"$PWCLI" open https://example.com --headed
"$PWCLI" tracing-start
# ...interactions...
"$PWCLI" tracing-stop
```

### Multi-tab work

```bash
"$PWCLI" tab-new https://example.com
"$PWCLI" tab-list
"$PWCLI" tab-select 0
"$PWCLI" snapshot
```

## Wrapper script

The wrapper script uses `npx --package @playwright/mcp playwright-cli` so the CLI can run without a global install:

```bash
"$PWCLI" --help
```

Prefer the wrapper unless the repository already standardizes on a global install.

## References

Open only what you need:

- CLI command reference: `references/cli.md`
- Practical workflows and troubleshooting: `references/workflows.md`

## Guardrails

- Always snapshot before referencing element ids like `e12`.
- Re-snapshot when refs seem stale.
- Prefer explicit commands over `eval` and `run-code` unless needed.
- When you do not have a fresh snapshot, use placeholder refs like `eX` and say why; do not bypass refs with `run-code`.
- Use `--headed` when a visual check will help.
- When capturing artifacts in this repo, use `output/playwright/` and avoid introducing new top-level artifact folders.
- Default to CLI commands and workflows, not Playwright test specs.
