# Live devnet verification & Docker/Windows gotchas

Read this before running any worker container, `docker compose`, or live operator
script against devnet. The gotchas section is load-bearing: several of them make a
**failed or stale run look like a passing one**.

## Contents
- [1. Dry-run first, then gated broadcast](#1-dry-run-first-then-gated-broadcast)
- [2. Reconcile on-chain + API + audit, and check idempotency](#2-reconcile-on-chain--api--audit-and-check-idempotency)
- [3. Where to run it (authority keypair lives in the worker container)](#3-where-to-run-it)
- [4. Docker/Windows gotchas that make verification lie](#4-dockerwindows-gotchas-that-make-verification-lie)
- [5. LLM preflight (rule compilation)](#5-llm-preflight-rule-compilation)
- [6. End-to-end settlement verification checklist](#6-end-to-end-settlement-verification-checklist)

## 1. Dry-run first, then gated broadcast

Every live operator script defaults to **dry-run** and only broadcasts when an env
flag is set. Always dry-run first to confirm the plan/target, then broadcast.

- Settlement runner: `apps/worker/src/settlement-run.ts` — broadcasts only with
  `IDEALER_SETTLE_BROADCAST=1`.
- Input seeder: `apps/worker/src/devnet-seed-real-inputs.ts` — `IDEALER_M5_INPUTS_BROADCAST=1`
  (+ `IDEALER_M5_INPUTS_OPEN_TURN=1` to open a fresh turn).
- Claim: `apps/worker/src/devnet-claim-prize.ts` — `IDEALER_SETTLE_BROADCAST=1`.
- Manual turn control: `POST /manual/open|close|cycle|settle` (gated by
  `IDEALER_TURN_MANUAL_CONTROL_ENABLED=true`).

The dry-run reads on-chain + API + recorded-transition state and prints the next
planned action without sending anything. If the dry-run shows the wrong target
turn or an unexpected plan, fix that before broadcasting.

## 2. Reconcile on-chain + API + audit, and check idempotency

A settlement/claim is verified only when all three stores agree:

1. **On-chain** (source of truth): re-read the `TurnState` / `Claim` PDA and confirm
   the expected fields (`activeRuleId`, `DrawStatus::Fulfilled`, draw numbers,
   `status=closed`, claim `Claimed`). Use the settlement runner's dry-run read or a
   direct RPC `getAccountInfo`.
2. **API projection**: `GET /rounds/{turnId}` (draw, election `settled` with the
   elected rule, winners, claims), `GET /verification-results?turnId=`,
   `GET /claim-records?turnId=`.
3. **Audit chain (M3.7)**: `GET /settlement-transitions?turnId=` should show the full
   §5.5 chain; `GET /audit/{logHash}` returns `integrity: true`. Each settlement
   transition is chained into the hash-linked audit log.

**Idempotency check (required):** re-run the same broadcast. It must be a no-op —
the runner returns `action=done (already_closed)` and all counts (transitions,
audit logs, results, winners, claims) stay unchanged. A second run that does more
work is a bug.

Note the API store is in-memory: restarting `idealer-api` wipes projections (on-chain
state persists). Re-derive/re-project if you restarted the API mid-verification.

## 3. Where to run it

Authority-gated instructions (open/close turn, set cycle, settle, record_claim) must
be signed by the on-chain `GlobalConfig.authority`, whose keypair lives **only** in
the `idealer-solana-home` Docker volume (mounted into the worker at
`IDEALER_TURN_AUTHORITY_KEYPAIR_PATH`). So **run authority ops inside the worker
container** via `docker exec` on the running worker, not from the host.

`claim_prize`/`claim_reward` are signed by the **claimant**, not the authority — the
claim operator reconstructs the deterministic seeded player keypair, so it needs RPC
+ API but not the authority key.

Set `IDEALER_API_BASE_URL=http://api:3002` for in-container worker→API calls (the
`.env` value `http://localhost:3002` resolves to the container itself, not the API
service). Override it with `docker exec -e`.

## 4. Docker/Windows gotchas that make verification lie

These are environmental, not code bugs, but each one can make you misread the result.

- **Git-bash mangles `docker run -v` Windows paths.** Run any `docker`/`docker run`
  with `-v <C:\path>:...` mounts from the **PowerShell** tool, not the Bash tool.
  (The SBF build/deploy already do this.)
- **Backgrounded `cmd | grep | tail` buffers until EOF.** A long-running
  `docker exec`/`docker compose run` piped through `grep`/`tail`/`Select-String` and
  backgrounded shows an **empty output file until the command finishes** — empty does
  NOT mean failure or success. To capture progress/results reliably, redirect inside
  the container to a **bind-mounted repo path** (e.g. `> /app/apps/worker/src/_out.json`,
  which the worker bind-mounts to `./apps/worker/src/`) and read it host-side, then
  delete it. Do not mount a host temp dir that Docker Desktop doesn't share — the
  mount silently writes nowhere.
- **`tsx watch` does NOT hot-reload over the Windows bind mount.** After editing
  `apps/api/src` or `apps/worker/src`, the running container keeps the OLD code. A
  new route 404s / old behavior persists until you `docker compose restart <svc>`.
  Confirm new code is live (e.g. the endpoint returns 400 not 404) before trusting a
  result. (One-off `docker compose run`/`docker exec tsx src/<file>.ts` DO read the
  current bind-mounted source, since tsx recompiles each invocation — so new operator
  scripts work without a restart, but new *package.json scripts* don't, because the
  container's package.json is baked into the image. Run `tsx src/<file>.ts` directly.)
- **Worker server crashes the loop on a transient devnet-RPC `ConnectTimeoutError`.**
  `api.devnet.solana.com:443` can time out from inside the container even when the
  host reaches it. Retry; confirm container egress with a quick
  `docker exec <worker> node -e "fetch('https://api.devnet.solana.com',...)"`.
- **Docker Desktop can wedge** (Dead/orphan worker container, name conflict, compose
  project state pinned to a deleted container). Symptoms: `docker compose up` keeps
  trying to recreate a phantom id, container stuck `Created`/`Dead`. Clear with
  `docker rm -f <id>` / `docker compose rm -sf worker`; if it won't clear, a Docker
  Desktop restart is required — that is an environment action, surface it to the user
  rather than thrashing.
- **`.env` overrides shell env in compose substitution on this setup.** To flip a
  worker flag (e.g. `IDEALER_TURN_AUTOMATION_ENABLED=false`,
  `IDEALER_TURN_MANUAL_CONTROL_ENABLED=true`), edit `.env`, recreate the worker, then
  **restore `.env`** afterward — an inline `VAR=x docker compose up` may not take.
  Verify via `GET :3003/health` (`turnAutomation`/`manualControl`).

## 5. LLM preflight (rule compilation)

Seeding a real eligible rule calls `/rule-preflight` (and `/rule-validations/run`),
which chains ~5-6 local LLM calls.

- `gemma4:latest` is **too slow** — the chain exceeds the timeout (>4 min) and hangs.
  Point the dealer model at `gemma3:4b` (`LLM_MODEL=gemma3:4b`,
  `LLM_PRIMARY_MODEL=gemma3:4b`, `LLM_TRANSLATE_TO_ENGLISH=false`) and restart the API;
  preflight then returns `decision=VALID` in ~2-3 min.
- The seed script must store the validation via `/rule-validations/run` (not only the
  preflight), and the on-chain `precompile_rule` must use **that** stored artifact hash
  — otherwise the elected rule can't be verified (`ACTIVE_RULE_ARTIFACT_NOT_FOUND`,
  hash mismatch).

## 6. End-to-end settlement verification checklist

For a full trustless-settlement run (M5.x) with real inputs:

1. Worker up, automation OFF, manual control ON, API on `gemma3:4b`, container egress OK.
2. Seed a turn (broadcast): N real `submit_guess` + ≥1 eligible LLM rule + votes; confirm
   the seeder summary has real signatures and the rule reached `Eligible`.
3. Drive `settlement-run.ts` (broadcast); it waits for the play window to end then runs
   lock → finalist VRF → elimination window → close → active VRF → freeze → draw → close
   → verify → winners → claims → CLOSED. (Devnet uses the `devnet-test`-gated short
   elimination window so this is minutes, not an hour.)
4. Reconcile per section 2 (on-chain `activeRuleId`/draw, `GET /rounds`, verification
   results with real WIN/LOSE, settlement-transitions + audit integrity).
5. Re-run the broadcast → must be `already_closed` (idempotent).
6. Claims: `record_claim` recorded the available claims; a claimant-signed `claim_prize`
   transfers real SOL (balance delta = recorded amount minus fee), the claim flips to
   `Claimed`, a re-claim is rejected on-chain (`ClaimNotAvailable`, custom 6058), and a
   below-threshold reward stays `unavailable` (never recorded).
