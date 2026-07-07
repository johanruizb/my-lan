import type { HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

// CardHeader primitive unificado (reemplaza ad-hoc Devices.tsx:116/Scans.tsx:295).
// Variantes: default flex-col, toolbar flex-row items-center justify-between.
// F3.4 migra imports desde card.tsx CardHeader a este primitive.

export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
    variant?: "default" | "toolbar";
}

export function CardHeader({
    className,
    variant = "default",
    ...props
}: CardHeaderProps) {
    return (
        <div
            className={cn(
                "p-3",
                variant === "toolbar"
                    ? "flex flex-row items-center justify-between gap-4"
                    : "flex flex-col gap-1",
                className,
            )}
            {...props}
        />
    );
}
