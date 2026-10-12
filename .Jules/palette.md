## 2025-02-18 - Filtered Empty State vs. Initial Empty State
**Learning:** When filtering lists in desktop interfaces, showing an initial empty state ("No items in network") when items exist but don't match active filters confuses users. Differentiating a "No results found" state with a direct "Clear filters" action improves filter visibility and recoverability.
**Action:** Always check if total items > 0 when filtered items == 0, and present a distinct empty state with a reset filter CTA.
