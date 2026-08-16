import { cn } from "@/lib/utils";

// Indicador online/offline para tarjetas, filas de tabla y headers (AC-12,
// AC-17). Texto plano con punto de color, sin pill: reduce el ruido de chips
// en la UI. Color sutil + aria-label/title.

export interface OnlineBadgeProps {
    isOnline: boolean;
    className?: string;
}

export function OnlineBadge({ isOnline, className }: OnlineBadgeProps) {
    // Tolerate a missing value at runtime (defensive: el backend serializa
    // is_online vía models.rs #[serde(default)], pero si llegara undefined,
    // default a offline). fix review #2.
    const online = isOnline ?? false;
    const label = online ? "En línea" : "Fuera de línea";
    // #35: dot verde usa el token semántico `success` (no green-500 hardcodeado).
    const dotClass = online ? "bg-success" : "bg-muted-foreground/50";
    return (
        <span
            className={cn(
                "inline-flex items-center gap-1.5 text-xs",
                className,
            )}
            title={label}
            aria-label={label}
        >
            <span
                className={cn("h-1.5 w-1.5 rounded-full", dotClass)}
                aria-hidden
            />
            {label}
        </span>
    );
}
