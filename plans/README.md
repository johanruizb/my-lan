# Animation Improvement Plans

This directory contains prioritized, self-contained implementation plans produced by the `improve-animations` motion advisor. Each plan is designed to be executed by any agent without relying on conversation context.

## Active Plans

| # | Title | Severity | Category | Status | Dependencies |
|---|---|---|---|---|---|
| [001](001-dialog-toast-css-transitions.md) | Restore Native CSS Transitions for Dialog and Toast Overlays | HIGH | Missed opportunities & Easing | DONE | None |

## Recommended Execution Order

1. **`001-dialog-toast-css-transitions.md`** — Foundations for all overlays in MyLAN. Replaces dead `animate-in` classes in `dialog.tsx` and `toast.tsx` with hardware-accelerated, native CSS `@keyframes` in `index.css`, restoring smooth 200ms entrances and exits for all modals and notifications.

## Execution

To execute plan 001 with an agent, run:
```bash
# Hand off plan to an agent or executor
improve-animations execute plans/001-dialog-toast-css-transitions.md
```
