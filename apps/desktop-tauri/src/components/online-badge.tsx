import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// Indicador online/offline para tarjetas y filas de tabla (AC-12, AC-17).
// Punto verde "En línea" / gris "Fuera de línea". Color sutil + aria-label/title.

export interface OnlineBadgeProps {
    isOnline: boolean;
    className?: string;
}

export function OnlineBadge({ isOnline, className }: OnlineBadgeProps) {
    const label = isOnline ? "En línea" : "Fuera de línea";
    const variant = isOnline ? "success" : "outline";
    const dotClass = isOnline ? "bg-green-500" : "bg-muted-foreground/50";
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
