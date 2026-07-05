import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// Badge escala 0-100 (rango confirmado crates/mylan-core/src/confidence.rs: Confidence(u8) MAX=100).
// Number.isFinite(n) ? n : 0 maneja null/undefined/NaN/empty string (frontend recibe string en tauri.ts).
// Reusa por F4.5 (reemplaza número raw en Devices/DeviceDetail).

type ConfidenceVariant = "success" | "warning" | "outline";

export interface ConfidenceBadgeProps {
    value: string | number | null | undefined;
    className?: string;
}

function classify(score: number): {
    variant: ConfidenceVariant;
    label: string;
} {
    if (score >= 80) return { variant: "success", label: "Alta" };
    if (score >= 50) return { variant: "warning", label: "Media" };
    return { variant: "outline", label: "Baja" };
}

export function ConfidenceBadge({ value, className }: ConfidenceBadgeProps) {
    const n = Number(value);
    const score = Number.isFinite(n) ? n : 0;
    const clamped = Math.max(0, Math.min(100, score));
    const { variant, label } = classify(clamped);
    return (
        <Badge
            variant={variant}
            className={cn("tabular-nums", className)}
            title={`Confianza ${clamped}/100 — ${label}`}
            aria-label={`Confianza ${clamped} de 100, ${label}`}
        >
            {clamped}
        </Badge>
    );
}
