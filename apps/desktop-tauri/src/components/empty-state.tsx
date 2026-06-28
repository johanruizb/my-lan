import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

// Empty states ilustrados/amigables (AC-6): icono + copy + CTA opcional.
export function EmptyState({
    icon: Icon,
    title,
    description,
    action,
    className,
}: {
    icon: LucideIcon;
    title: string;
    description?: string;
    action?: ReactNode;
    className?: string;
}) {
    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-border bg-muted/30 px-6 py-12 text-center",
                className,
            )}
        >
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <Icon className="h-6 w-6" aria-hidden />
            </div>
            <div className="flex flex-col gap-1">
                <p className="text-sm font-medium text-foreground">{title}</p>
                {description && (
                    <p className="max-w-sm text-sm text-muted-foreground">
                        {description}
                    </p>
                )}
            </div>
            {action}
        </div>
    );
}
