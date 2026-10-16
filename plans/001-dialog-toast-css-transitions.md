# 001 — Restore Native CSS Transitions for Dialog and Toast Overlays

- **Status**: DONE (implemented in feature/anim-001-dialog-toast @ 85f739a)
- **Commit**: 04a6868
- **Severity**: HIGH
- **Category**: Missed opportunities & Easing / duration
- **Estimated scope**: 3 files (`apps/desktop-tauri/src/index.css`, `apps/desktop-tauri/src/components/ui/dialog.tsx`, `apps/desktop-tauri/src/components/ui/toast.tsx`)

## Problem

In `apps/desktop-tauri/src/components/ui/dialog.tsx` and `apps/desktop-tauri/src/components/ui/toast.tsx`, overlay and container components reference Tailwind classes such as `data-[state=open]:animate-in`, `data-[state=open]:fade-in-0`, `data-[state=open]:zoom-in-95`, and `data-[state=closed]:slide-out-to-right-full`.

However, `tailwindcss-animate` is neither installed in `package.json` nor registered in `tailwind.config.js`. These utility classes produce zero compiled CSS rules.

As a consequence:
1. Every modal dialog in the application (About modal, Onboarding tour, Censorship upgrade prompt, and unsaved changes confirmation dialogs in `DeviceDetail` and `Settings`) teleports into the viewport instantaneously at 0ms. On dismiss, it disappears abruptly with zero exit transition.
2. Every toast notification (copied IP/MAC feedback, discovery scan completion, export results) snaps into the bottom-right corner without an entrance motion and vanishes with an abrupt layout cut upon timeout.

### Current Code Excerpts

```tsx
/* apps/desktop-tauri/src/components/ui/dialog.tsx:18-25 */
    <DialogPrimitive.Overlay
        ref={ref}
        className={cn(
            "fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=open]:fade-in-0",
            className,
        )}
        {...props}
    />
```

```tsx
/* apps/desktop-tauri/src/components/ui/dialog.tsx:35-42 */
        <DialogPrimitive.Content
            ref={ref}
            className={cn(
                "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border border-border bg-background p-6 shadow-lg data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95 sm:rounded-lg overscroll-contain",
                className,
            )}
            {...props}
        >
```

```tsx
/* apps/desktop-tauri/src/components/ui/toast.tsx:41-43 */
const toastVariants = cva(
    "group pointer-events-auto relative flex w-full items-start justify-between gap-3 overflow-hidden rounded-lg border p-4 pr-8 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--radix-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full",
    {
```

## Target

Implement lightweight, native CSS `@keyframes` and state-binding classes in `index.css` that hook into Radix UI's `data-[state=open]`, `data-[state=closed]`, and `data-[swipe=*]` attributes.

- **Properties animated**: Strictly GPU-accelerated `transform` and `opacity`. No layout property animations (`width`, `height`, `top`, `left`).
- **Modal Content Entrance**: `scale(0.96) translate(-50%, -48%)` and `opacity: 0` → settled `scale(1) translate(-50%, -50%)` and `opacity: 1` over **200ms** with curve `cubic-bezier(0.16, 1, 0.3, 1)`. Modals stay centered (`transform-origin: center`).
- **Modal Content Exit**: `scale(1) translate(-50%, -50%)` → `scale(0.97) translate(-50%, -48%)` and `opacity: 0` over **150ms** with curve `cubic-bezier(0.4, 0, 1, 1)`.
- **Modal Overlay**: Fade in from `opacity: 0` to `opacity: 1` over **200ms** `cubic-bezier(0.16, 1, 0.3, 1)`; fade out over **150ms** `cubic-bezier(0.4, 0, 1, 1)`.
- **Toast Entrance**: Enter from bottom edge `translateY(12px) scale(0.96)` and `opacity: 0` → `translateY(0) scale(1)` and `opacity: 1` over **220ms** with curve `cubic-bezier(0.16, 1, 0.3, 1)`.
- **Toast Exit / Swipe End**: Slide off to the right edge `translateX(0) → translateX(100%)` and `opacity: 1 → 0` over **180ms** with curve `cubic-bezier(0.4, 0, 1, 1)`.
- **Toast Swipe Cancel**: Spring back to `translateX(0)` over **200ms** with `cubic-bezier(0.16, 1, 0.3, 1)`.
- **Reduced Motion**: Gracefully preserve opacity fading for orientation while disabling all `translate` and `scale` transforms.

## Repo Conventions to Follow

- Keyframes and motion classes live in `apps/desktop-tauri/src/index.css`. Look at lines 128-180 of `index.css` as exemplar:
  - `@keyframes radar-pulse` and `@keyframes progress-indeterminate` are defined in plain CSS.
  - `@media (prefers-reduced-motion: reduce)` is consolidated at lines 170-180.
- Radix UI primitives (`@radix-ui/react-dialog` and `@radix-ui/react-toast`) natively inspect attached CSS animations on unmount: when `data-[state=closed]` has an active animation, Radix automatically defers DOM unmounting until the animation finishes.
- Do NOT add new npm dependencies (`tailwindcss-animate` is not needed when native CSS keyframes provide direct, zero-overhead control).

## Steps

### Step 1: Add keyframes and utility classes in `apps/desktop-tauri/src/index.css`

Append the following CSS rules to `apps/desktop-tauri/src/index.css` right before the `@media (prefers-reduced-motion: reduce)` block (around line 168):

```css
/* Dialog and Overlay transitions */
@keyframes dialog-overlay-show {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

@keyframes dialog-overlay-hide {
    from {
        opacity: 1;
    }
    to {
        opacity: 0;
    }
}

@keyframes dialog-content-show {
    from {
        opacity: 0;
        transform: translate(-50%, -48%) scale(0.96);
    }
    to {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1);
    }
}

@keyframes dialog-content-hide {
    from {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1);
    }
    to {
        opacity: 0;
        transform: translate(-50%, -48%) scale(0.97);
    }
}

.dialog-overlay[data-state="open"] {
    animation: dialog-overlay-show 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.dialog-overlay[data-state="closed"] {
    animation: dialog-overlay-hide 150ms cubic-bezier(0.4, 0, 1, 1) forwards;
}

.dialog-content[data-state="open"] {
    animation: dialog-content-show 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.dialog-content[data-state="closed"] {
    animation: dialog-content-hide 150ms cubic-bezier(0.4, 0, 1, 1) forwards;
}

/* Toast notifications */
@keyframes toast-slide-in {
    from {
        opacity: 0;
        transform: translateY(12px) scale(0.96);
    }
    to {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}

@keyframes toast-slide-out {
    from {
        opacity: 1;
        transform: translateX(0);
    }
    to {
        opacity: 0;
        transform: translateX(100%);
    }
}

.toast-root[data-state="open"] {
    animation: toast-slide-in 220ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.toast-root[data-state="closed"] {
    animation: toast-slide-out 180ms cubic-bezier(0.4, 0, 1, 1) forwards;
}

.toast-root[data-swipe="move"] {
    transform: translateX(var(--radix-toast-swipe-move-x));
    transition: none;
}

.toast-root[data-swipe="cancel"] {
    transform: translateX(0);
    transition: transform 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-root[data-swipe="end"] {
    animation: toast-slide-out 180ms cubic-bezier(0.4, 0, 1, 1) forwards;
}
```

And update the existing `@media (prefers-reduced-motion: reduce)` block in `index.css`:

```css
@media (prefers-reduced-motion: reduce) {
    .animate-radar-ring-1,
    .animate-radar-ring-2,
    .animate-radar-ring-3,
    .animate-ping,
    .animate-pulse,
    .animate-spin,
    .progress-indeterminate {
        animation: none;
    }

    @keyframes dialog-content-show {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    @keyframes dialog-content-hide {
        from { opacity: 1; }
        to { opacity: 0; }
    }
    @keyframes toast-slide-in {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    @keyframes toast-slide-out {
        from { opacity: 1; }
        to { opacity: 0; }
    }
    .toast-root[data-swipe="cancel"] {
        transition: none;
    }
}
```

### Step 2: Update `apps/desktop-tauri/src/components/ui/dialog.tsx`

Replace the dead `animate-in` classes on `DialogOverlay` and `DialogContent` with the new `.dialog-overlay` and `.dialog-content` utility classes.

In `DialogOverlay`:
```tsx
export const DialogOverlay = React.forwardRef<
    React.ElementRef<typeof DialogPrimitive.Overlay>,
    React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, ...props }, ref) => (
    <DialogPrimitive.Overlay
        ref={ref}
        className={cn(
            "fixed inset-0 z-50 bg-black/80 dialog-overlay",
            className,
        )}
        {...props}
    />
));
```

In `DialogContent`:
```tsx
export const DialogContent = React.forwardRef<
    React.ElementRef<typeof DialogPrimitive.Content>,
    React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>
>(({ className, children, ...props }, ref) => (
    <DialogPortal>
        <DialogOverlay />
        <DialogPrimitive.Content
            ref={ref}
            className={cn(
                "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border border-border bg-background p-6 shadow-lg sm:rounded-lg overscroll-contain dialog-content",
                className,
            )}
            {...props}
        >
```

### Step 3: Update `apps/desktop-tauri/src/components/ui/toast.tsx`

In `apps/desktop-tauri/src/components/ui/toast.tsx`, update `toastVariants`:

Replace:
```tsx
const toastVariants = cva(
    "group pointer-events-auto relative flex w-full items-start justify-between gap-3 overflow-hidden rounded-lg border p-4 pr-8 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--radix-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full",
```

With:
```tsx
const toastVariants = cva(
    "toast-root group pointer-events-auto relative flex w-full items-start justify-between gap-3 overflow-hidden rounded-lg border p-4 pr-8 shadow-lg",
```

## Boundaries

- Do NOT touch `Collapsible`, `DropdownMenu`, or `Select` in this plan (keep scope strictly focused on `Dialog` and `Toast`).
- Do NOT install `tailwindcss-animate` or modify `package.json` / `tailwind.config.js`.
- Do NOT change any dialog content markup or layout styling.
- Do NOT alter `toastDurations` or toast logic in `toast.tsx`.
- If the target lines in `dialog.tsx`, `toast.tsx`, or `index.css` differ from the excerpts above, STOP and report drift before making edits.

## Verification

### Mechanical Verification
Run the desktop test suite and production build from the repo root:
```bash
npm --prefix apps/desktop-tauri run build
npm --prefix apps/desktop-tauri test
```
- Both commands must succeed with exit code 0.
- Verify that `dist/assets/index-*.css` contains the keyframes `@keyframes dialog-content-show` and `@keyframes toast-slide-in`.

### Feel Check
1. **Dialog Feel Check**:
   - In the sidebar footer, click **Acerca de** to open the About dialog.
   - Confirm the dark overlay fades in smoothly (200ms) while the modal card emerges with a subtle scale from `0.96` to `1` without bouncing.
   - Press `Escape` or click the `X` button. Confirm the modal smoothly fades and scales to `0.97` before unmounting, rather than vanishing instantaneously.
2. **Toast Feel Check**:
   - In the device list, click the `CopyButton` on any IP or MAC address.
   - Confirm the toast slides gently upward into the bottom-right corner over ~220ms.
   - Click the toast `X` close button or wait for timeout: confirm it slides off to the right edge over ~180ms.
   - On a touch/trackpad, drag the toast to the right: confirm it follows the finger/cursor without delay (`data-swipe="move"`), snaps back smoothly on release (`data-swipe="cancel"`), and dismisses on full swipe (`data-swipe="end"`).
3. **Reduced Motion Check**:
   - In browser / webview DevTools (Rendering tab), enable `prefers-reduced-motion: reduce`.
   - Open and close the dialog; trigger a toast.
   - Confirm position and scale changes are suppressed, while gentle opacity fades remain active for state indication.

- **Done when**: All dialogs and toasts smoothly animate their entrance and exit in under 220ms on transform/opacity only, with zero layout shift and clean reduced-motion fallbacks.
