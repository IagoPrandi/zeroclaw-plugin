---
name: idealer-ui-coherence
description: Preserve current app visual identity when creating, editing, extending, or reviewing UI screens, components, flows, copy, styling, animations, responsive behavior, or frontend PRs. Use for any app/interface work that could affect screen limits, typography, font usage, colors, button shape, chat bubbles, component layout, copy tone, visual effects, mobile ergonomics, or consistency between new and existing Idealer surfaces.
---

# Idealer UI Coherence

Use this skill as a guardrail before changing any Idealer UI. The goal is not to make screens "look better" in isolation; it is to extend the product without breaking the existing identity.

## Required Reading

Before editing UI, open:

- `PRD.md`
- `AGENTS.md`
- `.claude/skills`
- `.claude/skills/idealer-design-system/SKILL.md`
- Relevant `idealer-design-system/references/*` files:
  - `01-tokens.md` for colors, fonts, spacing, radius, shadows, effects
  - `02-components.md` for buttons, cards, pills, nav, number grid, chat bubbles
  - `03-screens-and-state.md` for screen structure, navigation, gates, overlays
  - `04-new-screen-recipe.md` for new screens or new flows

Also open the real implementation files for the surface being edited. For the current Next.js app, inspect `apps/web/src/features/idealer/idealer.css`, `shared.tsx`, `IdealerApp.tsx`, and neighboring screens in `apps/web/src/features/idealer/screens/`. For legacy prototype work, inspect the corresponding `Idealer.html`, `idealer.css`, `shared.jsx`, `app*.jsx` files if present.

Confirm in the work log or PR description which required documents were opened.

## Workflow

1. Identify the product surface.
   - Determine whether the change affects the current Next.js app, the legacy no-build prototype, admin UI, public round page, or documentation.
   - Do not apply prototype-only assumptions blindly to the Next.js app. Translate the identity rules into the actual component/style system being edited.

2. Build a local identity inventory before designing.
   - List the nearest existing screens/components that already solve a similar problem.
   - Identify the exact tokens, classes, typography roles, radius, button variants, card patterns, icon style, overlays, and animation patterns already in use.
   - Prefer reuse or small extension over new visual primitives.

3. Define the coherence contract for the change.
   - State which existing component pattern the new UI follows.
   - State which tokens/classes are used for color, type, spacing, radius, border, shadow, and motion.
   - State the intended mobile behavior first, then desktop behavior.
   - State copy tone and language. In this repo, follow the current project instruction that UI/UX copy must be English unless the user explicitly changes that rule.

4. Implement with the smallest visual vocabulary.
   - Use design tokens and existing classes/components.
   - Add a new component only when reuse would make the code harder to understand or when the pattern will be repeated.
   - Add a new token/class only when an existing token cannot express the needed state. Document why.
   - Never hard-code colors, random font sizes, arbitrary radii, one-off gradients, or effects that bypass the existing theme.

5. Verify visually and mechanically.
   - Run typecheck/lint/build or the closest available frontend checks.
   - Inspect in browser at mobile-first sizes: 320-360px, 390-430px, 768px, and a desktop width when relevant.
   - Check console errors, horizontal overflow, text clipping, focus states, disabled/loading/error/success states, and wallet/browser-safe viewport behavior.
   - If the UI changed, include screenshot paths or browser verification notes in the work log/PR description.

6. Record the work.
   - Update the work log after the task.
   - Update the Progress Tracker only when a milestone is finished.
   - Tick milestone checkboxes only after explicit user confirmation.

## Non-Negotiable UI Rules

- Preserve the mobile-first live game identity: dense, premium, dark, social, fast to scan, and built for wallet-browser use.
- Use the established font system. Numbers, monetary values, turn ids, percentages, counts, countdowns, and wallet fragments must use the numeric/display font treatment already used by the app.
- Use existing tokens for color and surfaces. New hard-coded hex/rgb/hsl values in UI code are a defect unless they are already part of the canonical token definition.
- Use established button, card, pill, sheet, nav, number-cell, and chat-bubble shapes. Do not introduce unrelated border radii, shadows, strokes, or filled icon styles.
- Use flex/grid with `gap` for layout rhythm. Avoid margin piles, absolute positioning for normal layout, fixed heights that can clip content, and `overflow: hidden` as a bug mask.
- Keep tap targets at least 44px. Keep body text readable on small screens. Never allow labels, buttons, pills, cards, or sheets to clip English copy.
- Respect existing animation modes and reduced/calm/glow toggles. New motion must be transform-based where possible and must not hide content during initial render.
- Do not use fallbacks that hide data, API, wallet, or layout errors. Show a real state and keep the issue observable.
- Do not add visible instructional text that explains the existence of UI features unless product copy explicitly requires it. Prefer controls and states that are self-evident.

## Divergence Gates

Stop and fix before delivery if any of these are true:

- A new screen does not look like it belongs next to `Home`, `Play`, `Create`, `Vote`, `Result`, `Rewards`, `Share`, or `RoundPage`.
- The change introduces colors, font families, radii, button styles, chat bubble styles, nav treatment, card treatment, or visual effects not tied to existing tokens/components.
- Desktop works but mobile clips, overflows, hides primary actions, or requires awkward scrolling for core actions.
- Copy tone differs from the app's concise game/action tone, contains typos, or mixes languages without a product reason.
- A component handles only the happy path and lacks loading, empty, error, disabled, pending wallet, or success states where those states can occur.
- The implementation duplicates an existing component pattern instead of extending or composing it.

## Optional Reference

For PR review or final self-check, load `references/ui-coherence-checklist.md`.
