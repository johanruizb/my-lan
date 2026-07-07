import * as React from "react";
import * as ToastPrimitives from "@radix-ui/react-toast";
import { cva, type VariantProps } from "class-variance-authority";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

// Radix Toast: ARIA live region por defecto (role="status" / aria-live),
// swipe-to-dismiss, keyboard escape, focus management (AC-4, AC-15).

const ToastProvider = ToastPrimitives.Provider;

const toastViewportVariants = cva(
    "fixed z-[100] flex max-h-screen w-full flex-col-reverse gap-2 p-4 outline-none sm:flex-col sm:bottom-0 sm:right-0 sm:top-auto sm:max-w-[420px]",
);

export const ToastViewport = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Viewport>,
    React.ComponentPropsWithoutRef<typeof ToastPrimitives.Viewport>
>(({ className, ...props }, ref) => (
    <ToastPrimitives.Viewport
        ref={ref}
        className={cn(toastViewportVariants(), className)}
        {...props}
    />
));
ToastViewport.displayName = "ToastViewport";

type ToastVariant = "default" | "success" | "warning" | "error";

// #39: variantes migran de bg-green-50/bg-red-50 hardcoded a tokens
// semánticos --success/--warning/--destructive con sus foregrounds.
const toastVariants = cva(
    "group pointer-events-auto relative flex w-full items-start justify-between gap-3 overflow-hidden rounded-md border p-4 pr-6 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--radix-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-right-full",
    {
        variants: {
            variant: {
                default: "border-border bg-card text-card-foreground",
                success:
                    "border-transparent bg-success text-success-foreground",
                warning:
                    "border-transparent bg-warning text-warning-foreground",
                error: "border-transparent bg-destructive text-destructive-foreground",
            },
        },
        defaultVariants: { variant: "default" },
    },
);

// #42: duración por variante (ms). success breve, error persistente.
const toastDurations: Record<ToastVariant, number> = {
    default: 4000,
    success: 3000,
    warning: 4000,
    error: 6000,
};

export interface ToastProps
    extends
        React.ComponentPropsWithoutRef<typeof ToastPrimitives.Root>,
        VariantProps<typeof toastVariants> {}

export const Toast = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Root>,
    ToastProps
>(({ className, variant = "default", children, ...props }, ref) => (
    <ToastPrimitives.Root
        ref={ref}
        className={cn(toastVariants({ variant }), className)}
        {...props}
    >
        <div className="grid gap-0.5 text-sm">{children}</div>
        <ToastPrimitives.Close className="absolute right-1 top-1 rounded-md p-1 text-muted-foreground opacity-70 transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
            <X className="h-4 w-4" aria-hidden />
            <span className="sr-only">Cerrar</span>
        </ToastPrimitives.Close>
    </ToastPrimitives.Root>
));
Toast.displayName = "Toast";

// --- API basada en contexto (compatible con el hook useToast existente) -------

type ToastVariantName = ToastVariant;

interface ToastItem {
    id: string;
    message: string;
    variant: ToastVariantName;
}

interface ToastContextValue {
    toast: (message: string, variant?: ToastVariantName) => void;
}

const ToastContext = React.createContext<ToastContextValue | null>(null);

export function ToastProviderApp({ children }: { children: React.ReactNode }) {
    const [items, setItems] = React.useState<ToastItem[]>([]);

    const toast = React.useCallback(
        (message: string, variant: ToastVariantName = "default") => {
            const id = `t-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
            setItems((prev) => [...prev, { id, message, variant }]);
        },
        [],
    );

    return (
        <ToastContext.Provider value={{ toast }}>
            <ToastProvider swipeDirection="right" duration={4000}>
                {children}
                {items.map((t) => (
                    <Toast
                        key={t.id}
                        variant={t.variant}
                        duration={toastDurations[t.variant]}
                        onOpenChange={(open) => {
                            if (!open)
                                setItems((prev) =>
                                    prev.filter((x) => x.id !== t.id),
                                );
                        }}
                    >
                        {t.message}
                    </Toast>
                ))}
                <ToastViewport />
            </ToastProvider>
        </ToastContext.Provider>
    );
}

export function useToast(): ToastContextValue {
    const ctx = React.useContext(ToastContext);
    if (!ctx) {
        throw new Error("useToast debe usarse dentro de ToastProviderApp");
    }
    return ctx;
}
