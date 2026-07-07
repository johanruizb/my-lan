import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";

// Certeza 0-100 (rango confirmado crates/mylan-core/src/confidence.rs:
// Confidence(u8) MAX=100). Number.isFinite(n) ? n : 0 maneja
// null/undefined/NaN/empty string (frontend recibe string en tauri.ts).
//
// #31: render integrado barra+número en una fila — label "Certeza" + Progress
// flex-1 (h-1.5) + número % al final. Un solo componente, no Badge + Progress
// separados. Reusa por F4.5 (reemplaza número raw en Devices/DeviceDetail).
// #25: label y title/aria "Certeza" (era "Confianza"). La barra se coloriza por
// umbral (success/warning/outline = Alta/Media/Baja) vía arbitrary variant sobre
// el indicador Radix. El componente Progress es compartido (Dashboard, Devices,
// DeviceDetail) y su indicador es hardcoded `bg-primary`; no lo modificamos —
// usamos `[&>div]:bg-*` que targetea el div indicador hijo del Root con
// especificidad (0,1,1) > `bg-primary` (0,1,0) del indicador.

type ConfidenceVariant = "success" | "warning" | "outline";

export interface ConfidenceBadgeProps {
    value: string | number | null | undefined;
    className?: string;
    /** Mostrar label "Certeza" inline (default true). false cuando un header
     * externo ya provee el label + tooltip (p. ej. DeviceDetail). */
    showLabel?: boolean;
}

function classify(score: number): {
    variant: ConfidenceVariant;
    label: string;
} {
    if (score >= 80) return { variant: "success", label: "Alta" };
    if (score >= 50) return { variant: "warning", label: "Media" };
    return { variant: "outline", label: "Baja" };
}

// Color del indicador por variante. Strings estáticos (Tailwind JIT necesita
// ver el literal completo para generar la regla).
const barIndicatorClass: Record<ConfidenceVariant, string> = {
    success: "[&>div]:bg-success",
    warning: "[&>div]:bg-warning",
    outline: "[&>div]:bg-muted-foreground",
};

export function ConfidenceBadge({
    value,
    className,
    showLabel = true,
}: ConfidenceBadgeProps) {
    const n = Number(value);
    const score = Number.isFinite(n) ? n : 0;
    const clamped = Math.max(0, Math.min(100, score));
    const { variant, label } = classify(clamped);
    const titleText = `Certeza ${clamped}/100 — ${label}`;
    return (
        <div
            className={cn("flex items-center gap-2", className)}
            title={titleText}
        >
            {showLabel && (
                <span className="shrink-0 text-xs text-muted-foreground">
                    Certeza
                </span>
            )}
            <Progress
                value={clamped}
                aria-label={titleText}
                className={cn("h-1.5 flex-1", barIndicatorClass[variant])}
            />
            <span className="shrink-0 text-xs tabular-nums">{clamped}%</span>
        </div>
    );
}
