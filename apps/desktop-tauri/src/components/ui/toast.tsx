import * as React from "react";
import * as ToastPrimitives from "@radix-ui/react-toast";
import { cva, type VariantProps } from "class-variance-authority";
import {
    AlertCircle,
    AlertTriangle,
    CheckCircle2,
    Info,
    X,
} from "lucide-react";
import { cn } from "@/lib/utils";

// Componente Toast canónico de shadcn/ui con Radix Toast y Lucide Icons
// https://ui.shadcn.com/docs/components/base/toast

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
ToastViewport.displayName = ToastPrimitives.Viewport.displayName;

export type ToastVariant =
    | "default"
    | "success"
    | "warning"
    | "error"
    | "destructive";

const toastVariants = cva(
    "toast-root group pointer-events-auto relative flex w-full items-start justify-between gap-3 overflow-hidden rounded-lg border p-4 pr-8 shadow-lg",
    {
        variants: {
            variant: {
                default:
                    "border-border bg-card/95 text-card-foreground backdrop-blur supports-[backdrop-filter]:bg-card/85",
                success:
                    "border-emerald-500/30 bg-card/95 text-card-foreground backdrop-blur supports-[backdrop-filter]:bg-card/85 dark:border-emerald-500/40",
                warning:
                    "border-amber-500/30 bg-card/95 text-card-foreground backdrop-blur supports-[backdrop-filter]:bg-card/85 dark:border-amber-500/40",
                error:
                    "border-destructive/40 bg-card/95 text-card-foreground backdrop-blur supports-[backdrop-filter]:bg-card/85 dark:border-destructive/60",
                destructive:
                    "destructive group border-destructive/40 bg-destructive/10 text-destructive dark:border-destructive/60",
            },
        },
        defaultVariants: { variant: "default" },
    },
);

const toastDurations: Record<ToastVariant, number> = {
    default: 4000,
    success: 3500,
    warning: 4500,
    error: 6000,
    destructive: 6000,
};

export interface ToastProps
    extends
        React.ComponentPropsWithoutRef<typeof ToastPrimitives.Root>,
        VariantProps<typeof toastVariants> {}

export const Toast = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Root>,
    ToastProps
>(({ className, variant = "default", ...props }, ref) => (
    <ToastPrimitives.Root
        ref={ref}
        className={cn(toastVariants({ variant }), className)}
        {...props}
    />
));
Toast.displayName = ToastPrimitives.Root.displayName;

export const ToastAction = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Action>,
    React.ComponentPropsWithoutRef<typeof ToastPrimitives.Action>
>(({ className, ...props }, ref) => (
    <ToastPrimitives.Action
        ref={ref}
        className={cn(
            "inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium ring-offset-background transition-colors hover:bg-secondary focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
            className,
        )}
        {...props}
    />
));
ToastAction.displayName = ToastPrimitives.Action.displayName;

export const ToastClose = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Close>,
    React.ComponentPropsWithoutRef<typeof ToastPrimitives.Close>
>(({ className, ...props }, ref) => (
    <ToastPrimitives.Close
        ref={ref}
        className={cn(
            "absolute right-2 top-2 rounded-md p-1 text-muted-foreground opacity-70 transition-opacity hover:opacity-100 hover:text-foreground focus:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring",
            className,
        )}
        toast-close=""
        {...props}
    >
        <X className="h-4 w-4" aria-hidden />
        <span className="sr-only">Cerrar</span>
    </ToastPrimitives.Close>
));
ToastClose.displayName = ToastPrimitives.Close.displayName;

export const ToastTitle = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Title>,
    React.ComponentPropsWithoutRef<typeof ToastPrimitives.Title>
>(({ className, ...props }, ref) => (
    <ToastPrimitives.Title
        ref={ref}
        className={cn("text-xs font-semibold tracking-tight text-foreground", className)}
        {...props}
    />
));
ToastTitle.displayName = ToastPrimitives.Title.displayName;

export const ToastDescription = React.forwardRef<
    React.ElementRef<typeof ToastPrimitives.Description>,
    React.ComponentPropsWithoutRef<typeof ToastPrimitives.Description>
>(({ className, ...props }, ref) => (
    <ToastPrimitives.Description
        ref={ref}
        className={cn("text-xs text-muted-foreground leading-normal", className)}
        {...props}
    />
));
ToastDescription.displayName = ToastPrimitives.Description.displayName;

function ToastIcon({ variant }: { variant: ToastVariant }) {
    switch (variant) {
        case "success":
            return (
                <CheckCircle2
                    className="h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400 mt-0.5"
                    aria-hidden
                />
            );
        case "warning":
            return (
                <AlertTriangle
                    className="h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400 mt-0.5"
                    aria-hidden
                />
            );
        case "error":
        case "destructive":
            return (
                <AlertCircle
                    className="h-4 w-4 shrink-0 text-destructive mt-0.5"
                    aria-hidden
                />
            );
        default:
            return (
                <Info
                    className="h-4 w-4 shrink-0 text-primary mt-0.5"
                    aria-hidden
                />
            );
    }
}

interface ToastItem {
    id: string;
    title?: string;
    message: string;
    variant: ToastVariant;
    duration: number;
}

export interface ToastOptions {
    title?: string;
    description?: string;
    variant?: ToastVariant;
    duration?: number;
}

export interface ToastContextValue {
    toast: (
        messageOrOptions: string | ToastOptions,
        variant?: ToastVariant,
    ) => void;
}

const ToastContext = React.createContext<ToastContextValue | null>(null);

export function ToastProviderApp({ children }: { children: React.ReactNode }) {
    const [items, setItems] = React.useState<ToastItem[]>([]);

    const toast = React.useCallback(
        (
            messageOrOptions: string | ToastOptions,
            variant: ToastVariant = "default",
        ) => {
            const id = `t-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
            if (typeof messageOrOptions === "string") {
                setItems((prev) => [
                    ...prev,
                    {
                        id,
                        message: messageOrOptions,
                        variant,
                        duration: toastDurations[variant],
                    },
                ]);
            } else {
                const itemVariant = messageOrOptions.variant ?? variant;
                setItems((prev) => [
                    ...prev,
                    {
                        id,
                        title: messageOrOptions.title,
                        message: messageOrOptions.description ?? "",
                        variant: itemVariant,
                        duration:
                            messageOrOptions.duration ??
                            toastDurations[itemVariant],
                    },
                ]);
            }
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
                        duration={t.duration}
                        onOpenChange={(open) => {
                            if (!open)
                                setItems((prev) =>
                                    prev.filter((x) => x.id !== t.id),
                                );
                        }}
                    >
                        <div className="flex items-start gap-2.5 w-full">
                            <ToastIcon variant={t.variant} />
                            <div className="grid gap-0.5 text-xs flex-1">
                                {t.title && <ToastTitle>{t.title}</ToastTitle>}
                                <ToastDescription
                                    className={cn(
                                        !t.title &&
                                            "font-medium text-foreground text-xs",
                                    )}
                                >
                                    {t.message}
                                </ToastDescription>
                            </div>
                        </div>
                        <ToastClose />
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
