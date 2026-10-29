# UI Backlog

Design source: `DESIGN.md`
Inventory: `2ff8c4a` · rebuilt in cycle 1 (`2026-10-29`) after `C-03`

## Open

| ID | Scope | Dimension | Observed vs rule | Class | Severity | Status |
|---|---|---|---|---|---|---|
| C-12 | Tools (Radar chart) | color | Bar/legend colors use the palette directly (`bg-rose-500/80`, `bg-emerald-500`) instead of semantic tokens | justified | low | accepted |
| C-13 | component:Collapsible | elevation | Radix root in DeviceDetail carries the container recipe (`rounded-xl border bg-card`) without `shadow-sm` because `Card` cannot own the Radix root | justified | low | accepted |

## Closed

| ID | Scope | Result | Evidence |
|---|---|---|---|
| C-01 | system-wide | One radius scale: controls 8px (`rounded-lg`), containers 12px (`rounded-xl`). `Card` and every container primitive moved to 12px; `Button`/`Input`/`Select` to 8px. `DESIGN.md` frontmatter now matches Tailwind's real steps (sm 4 / md 6 / lg 8 / xl 12). | `card.tsx`, `button.tsx`, `input.tsx`, `select.tsx`, `dialog.tsx`, `toast.tsx`, `dropdown-menu.tsx`, `alert.tsx`; build + ESLint clean |
| C-02 | component:CardTitle | Default is now Title role 16px (`text-base`); 14 redundant per-screen overrides removed (Settings 7, Tools 5, speed-test 2, DeviceDetail 5). | `card.tsx`, 4 consumers |
| C-03 | component:CardHeader | One primitive in `ui/card.tsx` with `default`/`toolbar` variants; `ui/card-header.tsx` deleted and its 2 consumers re-pointed. | `card.tsx`, `Devices.tsx`, `Scans.tsx` |
| C-04 | screen:Tools | 18 locally styled surfaces migrated to `Card` (5 KPI tiles, 6 Tools panels, 6 speed-test tiles, security score, DeviceDetail dossier, 3 gdrive cards). `shadow-xs`/`shadow-2xs` (nonexistent in Tailwind 3.4) eliminated. | `rg "shadow-xs\|shadow-2xs" src` → 0; build clean |
| C-05 | all screens | Display role lowered to 20px in the source (all 7 screens already agreed). 24px remains available for KPI numerals, not for page titles. | `DESIGN.md` Typography |
| C-06 | component:Badge | Added `info` plus tinted `success`/`warning`/`destructive` (10% bg, 30% border) backed by new `--*-strong` foreground tokens for AA contrast. Migrated 20 hand-rolled chips and tokenized status surfaces in Scans, DeviceDetail, security-score-card, speed-test, sync indicator, About, Settings, App, gdrive dialog and Tools. | `badge.tsx`, `index.css`, `tailwind.config.js`; `rg "bg-(emerald\|amber\|red\|rose\|sky)-[0-9]+/[0-9]+" src` → 0 except chart palette |
| C-07 | component:Table, MaskedValue | `tabular-nums` now lives on the `Table` root and on every `MaskedValue` branch, so all tables and network identifiers inherit alignment. | `table.tsx`, `masked-value.tsx` |
| C-08 | system-wide | Micro role fixed at 11px; all 160 arbitrary sizes now use the single step (3×9px and 55×10px raised). Rule recorded as The Micro Floor. | `rg "text-\[(9\|10)px\]" src` → 0 |
| C-09 | component:Table | `TableHead` height 32px → 40px per Tables rule. | `table.tsx` |
| C-10 | component:OnlineBadge | Status dot 6px → 8px; 6 legend/list dots in Tools aligned at the same time. | `online-badge.tsx`, `Tools.tsx` |
| C-11 | AppShell | Mobile gutter `px-4` → `px-6`, matching `px-6 py-6 md:px-8`. | `App.tsx` |

## Decisions

- `2026-10-29` · When every screen agrees and only the design source disagrees (`C-05`), update the source in one coordinated change instead of mass-editing screens · avoids churn, keeps the source authoritative · all screens.
- `2026-10-29` · Audit scope covers the 7 routed screens plus AboutDialog; captures were not taken because the desktop app was not launched · findings are source-derived · all screens.
- `2026-10-29` · Radius scale is expressed with Tailwind's native steps only: `sm` 4px, `md` 6px, `lg` 8px, `xl` 12px. Controls use `lg`; containers use `xl`. No custom radius token is introduced · removes the 8-vs-12 ambiguity that produced `C-01` · all screens.
- `2026-10-29` · Severity scales with more than three levels collapse onto the semantic set: critical → `destructive`, high → `warning`, medium → `outline`, low → `secondary`. Off-palette hues (yellow, blue, indigo, rose) are not reintroduced · protects The Telemetry Rarity Rule · DeviceDetail, security-score-card.
- `2026-10-29` · Minimum text size is 11px (The Micro Floor). Density is bought with padding and line-height, never with smaller type · 9–10px labels were unreadable at 1× on the target 1080p · all screens.
- `2026-10-29` · Chart palettes are a justified deviation from the semantic badges: data visualization needs categorical hues and is exempt from The Semantic State Rule · Tools radar/oscilloscope.
