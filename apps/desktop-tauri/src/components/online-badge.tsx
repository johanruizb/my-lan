import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// Indicador online/offline para tarjetas y filas de tabla (AC-12, AC-17).
// Punto verde "En línea" / gris "Fuera de línea". Color sutil + aria-label/title.

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
    const variant = online ? "success" : "outline";
    // #35: dot verde usa el token semántico `success` (no green-500 hardcodeado).
    const dotClass = online ? "bg-success" : "bg-muted-foreground/50";
    return (
        <Badge
            variant={variant}
            className={cn(className)}
            title={label}
            aria-label={label}
        >
            <span
                className={cn("h-1.5 w-1.5 rounded-full", dotClass)}
                aria-hidden
            />
            {label}
        </Badge>
    );
}
