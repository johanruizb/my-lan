// AC-13: Badge de confianza derivado (Confiable/Reconocido/Desconocido).
// Interpretación A (decidida, evita ambigüedad y saturación): `TrustBadge` es
// el **único** badge visible en la tarjeta/fila de lista de `Devices.tsx`; el
// score numérico de `ConfidenceBadge` (Alta/Media/Baja) va accesible en el
// `title`/`aria-label` del `TrustBadge` (no se renderiza como segundo badge
// separado en la lista). En `DeviceDetail.tsx` (sección info, no lista)
// `ConfidenceBadge` se mantiene como ahora (`:303`).
//
// Patrón visual: `confidence-badge.tsx` (tabular-nums, gap-1, h-3 w-3 icons).
// Fuente única del estado: `deriveTrustState` (`lib/trust-state.ts`).

import { ShieldCheck, CheckCircle2, HelpCircle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { deriveTrustState, type TrustState } from "@/lib/trust-state";
import type { Device } from "@/lib/tauri";

type TrustVariant = "success" | "warning" | "outline";

interface TrustDisplay {
    variant: TrustVariant;
    label: string;
    Icon: typeof ShieldCheck;
}

function trustDisplay(state: TrustState): TrustDisplay {
    switch (state) {
        case "trusted":
            return {
                variant: "success",
                label: "Confiable",
                Icon: ShieldCheck,
            };
        case "recognized":
            return {
                variant: "warning",
                label: "Reconocido",
                Icon: CheckCircle2,
            };
        case "unknown":
            return {
                variant: "outline",
                label: "Desconocido",
                Icon: HelpCircle,
            };
    }
}

export interface TrustBadgeProps {
    device: Pick<Device, "is_trusted" | "confidence">;
    className?: string;
}

export function TrustBadge({ device, className }: TrustBadgeProps) {
    const state = deriveTrustState(device);
    const { variant, label, Icon } = trustDisplay(state);

    // Score numérico reutilizado del patrón `ConfidenceBadge`
    // (`confidence-badge.tsx:25-27`): `Number(confidence)` → `Number.isFinite`
    // → clamp 0-100. NaN → 0 (consistente con `ConfidenceBadge`).
    const n = Number(device.confidence);
    const score = Number.isFinite(n) ? Math.max(0, Math.min(100, n)) : 0;
    const confLabel = score >= 80 ? "Alta" : score >= 50 ? "Media" : "Baja";

    const title = `Confianza ${score}/100 — ${confLabel} · ${label}`;
    const ariaLabel = `Confianza ${score} de 100, ${confLabel}. Estado: ${label}`;

    return (
        <Badge
            variant={variant}
            className={cn("gap-1", className)}
            title={title}
            aria-label={ariaLabel}
        >
            <Icon className="h-3 w-3" aria-hidden />
            {label}
        </Badge>
    );
}
