# UI State

Design source: `DESIGN.md` (tokens + named rules), `apps/desktop-tauri/tailwind.config.js`, `apps/desktop-tauri/src/index.css`, `apps/desktop-tauri/src/components/ui/*`
Design source covers: colors, typography roles, spacing, radii, elevation, component rules, named rules
Inventory built: `2ff8c4a` · rebuilt in cycle 1 · `2026-10-29` (after `C-03` merged `card-header` into `card`)
Inventory stale if: `apps/desktop-tauri/src/App.tsx`, `apps/desktop-tauri/src/screens/**`, `apps/desktop-tauri/src/components/**`, `apps/desktop-tauri/src/index.css`, `apps/desktop-tauri/tailwind.config.js`

Mode: `cycle` (closed)
Cycle: `1`
Scope: all routed screens + AboutDialog + shared visual components
Status: `closed` — backlog has no open high/medium findings
Next action: On the next `audit`, rebuild the inventory only if the paths above changed since cycle 1, then audit new or modified screens against `DESIGN.md`. No pending fixes.

## Screens

| Screen | Route / entry | Job | Layout family | Shared components |
|---|---|---|---|---|
| Red | `/` | summary | header + KPI cards | network-header, security-score-card |
| Dispositivos | `/devices` | inventory | table / card grid | alert, button, card, card-header (toolbar), dropdown-menu, empty-state, export-report-modal, info-tooltip, input, progress, select, table, toast, confidence-badge, copy-button, device-icons, masked-value, online-badge, relative-time, trust-badge |
| Detalle de host | `/devices/:ip` | diagnostic | stacked panels | alert, badge, button, card, collapsible, dialog, dropdown-menu, empty-state, form-field, info-tooltip, input, progress, switch, table, toast, confidence-badge, copy-button, device-icons, masked-value, online-badge, profile-select, trust-badge |
| Herramientas | `/tools` | operations | tabs + dense panels | alert, badge, button, card, input, select, table, toast, speed-test-panel |
| Historial | `/scans` | history | table | alert, badge, button, card, card-header (toolbar), empty-state, relative-time, table |
| Ajustes | `/settings` | configuration | section stack | alert, badge, button, card, dialog, info-tooltip, input, switch, toast, gdrive-sync-dialog, onboarding-dialog, updater-provider, workspace-provider |
| No encontrado | `*` | error | centered empty state | button, empty-state |
| Acerca de | sidebar modal (`AboutDialog`) | product info | dialog | alert, badge, button, dialog, progress |

## Shared components

| Component | Path | Consumers | Variants |
|---|---|---|---|
| alert | `components/ui/alert.tsx` | About, Devices, Scans, Settings, Tools, network-header | default, destructive |
| badge | `components/ui/badge.tsx` | DeviceDetail, Scans, Tools, security-score-card, speed-test-panel, gdrive-sync-dialog, sync-status-indicator, About, Settings, App | default, secondary, outline, info, success, warning, destructive |
| button | `components/ui/button.tsx` | 20 files (all screens + dialogs) | default, destructive, outline, secondary, ghost, link; sm, default, lg, icon |
| card | `components/ui/card.tsx` | DeviceDetail, Devices, Scans, Settings, Tools, gdrive-sync-dialog, network-header, security-score-card, speed-test-panel | single (12px, shadow-sm) |
| card-header | `components/ui/card.tsx` | Devices, Scans, DeviceDetail, Settings, Tools, speed-test-panel | default, toolbar |
| collapsible | `components/ui/collapsible.tsx` | DeviceDetail | single |
| dialog | `components/ui/dialog.tsx` | About, DeviceDetail, Settings, 4 dialogs | single |
| dropdown-menu | `components/ui/dropdown-menu.tsx` | DeviceDetail, Devices, workspace-selector | single |
| empty-state | `components/empty-state.tsx` | DeviceDetail, Devices, NotFound, Scans | single |
| form-field | `components/ui/form-field.tsx` | DeviceDetail, workspace-manage-dialog | single |
| info-tooltip | `components/ui/info-tooltip.tsx` | DeviceDetail, Devices, Settings, network-header | single |
| input | `components/ui/input.tsx` | App, DeviceDetail, Devices, Settings, Tools, 2 dialogs | single |
| progress | `components/ui/progress.tsx` | About, DeviceDetail, Devices, confidence-badge, network-header | single |
| select | `components/ui/select.tsx` | Devices, Tools, profile-select | single |
| switch | `components/ui/switch.tsx` | DeviceDetail, Settings, export-report-modal | single |
| table | `components/ui/table.tsx` | DeviceDetail, Devices, Scans, Tools, speed-test-panel | single (head 40px, `tabular-nums`) |
| toast | `components/ui/toast.tsx` | 11 files | single |
| tooltip | `components/ui/tooltip.tsx` | App, info-tooltip, sync-status-indicator | single |
| confidence-badge | `components/confidence-badge.tsx` | DeviceDetail, Devices | single |
| copy-button | `components/copy-button.tsx` | DeviceDetail, Devices | single |
| device-icons | `components/device-icons.tsx` | DeviceDetail, Devices | single |
| masked-value | `components/masked-value.tsx` | DeviceDetail, Devices, network-header | single (`tabular-nums`) |
| network-header | `components/network-header.tsx` | Dashboard | single |
| online-badge | `components/online-badge.tsx` | DeviceDetail, Devices | single (8px dot) |
| relative-time | `components/relative-time.tsx` | Devices, Scans | single |
| security-score-card | `components/security-score-card.tsx` | Dashboard | single |
| speed-test-panel | `components/speed-test-panel.tsx` | Tools | single |
| trust-badge | `components/trust-badge.tsx` | DeviceDetail, Devices | single |
| profile-select | `components/profile-select.tsx` | App, DeviceDetail, network-header | single |
| sync-status-indicator | `components/sync-status-indicator.tsx` | App | single |
| workspace-selector | `components/workspace-selector.tsx` | App | single |
| export-report-modal, gdrive-sync-dialog, onboarding-dialog, workspace-manage-dialog, censura-upgrade-dialog | `components/*.tsx` | Devices / Settings / App | single |

Non-visual providers (`theme-provider`, `workspace-provider`, `updater-provider`, `censorship-provider`) are excluded from the visual inventory.

## Tokens

| Group | Path | Notes |
|---|---|---|
| Color roles (HSL vars) | `src/index.css` | background, foreground, card, primary, secondary, muted, accent, destructive, success, warning, popover, border, input, ring |
| Tinted-state foregrounds | `src/index.css` | `--primary-strong`, `--success-strong`, `--warning-strong`, `--destructive-strong` — text on 10% semantic tints only |
| Color mapping | `tailwind.config.js` | tokens exposed as Tailwind colors, including `*.strong` |
| Radius scale | `tailwind.config.js` (`--radius: 0.5rem`) | sm 4px, md 6px, lg 8px (controls), xl 12px (containers) |
| Type scale | `DESIGN.md` | display 20, headline 18, title 16, body 14, label 12, micro 11 |
| Spacing scale | `DESIGN.md` | xs 4, sm 8, md 16, lg 24, xl 32 |
| Component rules | `DESIGN.md` | buttons, cards, inputs, tables, badges, collapsibles, named rules |

## Limitations

- No rendered capture: the Tauri desktop app was not launched. Validation is measured, not seen — TypeScript, ESLint and the production build pass, and the compiled CSS contains the new tokens, but pixel-level results are unverified.
- Interaction states were audited through the shared primitives; unreachable/edge states were not exercised.

## Cycle log

- `2026-10-29` · audit · 11 findings recorded, no code changed · `.ui/backlog.md`
- `2026-10-29` · cycle 1 · 11 findings closed, 2 accepted deviations (`C-12`, `C-13`) · ESLint clean, `vite build` OK, `tsc` only pre-existing errors in `executive-report-pdf`, `export-report-modal`, `speed-test-panel:100` · no captures
