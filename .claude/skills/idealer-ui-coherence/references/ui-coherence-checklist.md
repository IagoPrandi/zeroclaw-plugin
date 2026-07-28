# UI Coherence Checklist

Use this checklist before delivering any Idealer UI change.

## Context

- [ ] Opened `PRD.md`.
- [ ] Opened `AGENTS.md`.
- [ ] Opened `.claude/skills`.
- [ ] Opened `.claude/skills/idealer-design-system/SKILL.md`.
- [ ] Opened the relevant design-system reference files.
- [ ] Opened neighboring implementation files for the edited UI surface.
- [ ] Work log or PR description confirms the required markdown documents were opened.

## Identity

- [ ] New UI follows an existing Idealer screen/component pattern.
- [ ] Colors use existing tokens/classes, not one-off literals.
- [ ] Font family, font size, weight, line height, and numeric font treatment match nearby UI.
- [ ] Button shape, height, padding, icon style, disabled state, and active/tap feedback match existing buttons.
- [ ] Cards, pills, sheets, chat bubbles, dividers, and nav treatment match existing shapes and borders.
- [ ] Effects use existing glow, shadow, gradient, and animation patterns.
- [ ] Copy is concise, correctly spelled, and follows the current project UI language rule.

## Layout

- [ ] Mobile layout is designed first and works at 320-360px and 390-430px widths.
- [ ] Desktop/tablet behavior is intentional and does not stretch mobile components awkwardly.
- [ ] Layout uses flex/grid with `gap` instead of margin piles.
- [ ] No unintended horizontal overflow.
- [ ] Text wraps or truncates intentionally and never clips inside buttons, pills, cards, nav, or sheets.
- [ ] Tap targets are at least 44px.
- [ ] Fixed/sticky actions do not cover content.

## States

- [ ] Loading state exists where data or wallet state is pending.
- [ ] Empty state is explicit and product-appropriate.
- [ ] Error state exposes the issue instead of masking it with a fake fallback.
- [ ] Disabled state explains unavailable actions when needed.
- [ ] Success/pending wallet/failed wallet states are visible for transaction flows.
- [ ] Focus and keyboard behavior are usable for buttons, links, inputs, modals, and sheets.

## Verification

- [ ] Typecheck/lint/build or closest available checks passed.
- [ ] Browser inspected with no console errors.
- [ ] Screenshots or verification notes exist for changed mobile states.
- [ ] No unrelated visual refactor was included.
- [ ] Work log updated.
- [ ] Milestone checkbox was not ticked without explicit user confirmation.
