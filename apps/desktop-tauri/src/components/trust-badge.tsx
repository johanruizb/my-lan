// Badge de confianza **manual** y binario (Confiable/No confiable) basado
// únicamente en `is_trusted` (ADR-0006). La medición automática 0-100 vive
// aparte en `ConfidenceBadge` como "Certeza"; `TrustBadge` ya no deriva
// estado ni muestra score, y el estado intermedio "Reconocido" se elimina.
//
// Patrón visual: `confidence-badge.tsx` (tabular-nums, gap-1, h-3 w-3 icons).
// Variantes: success (ShieldCheck) para Confiable, outline para No confiable.

import { ShieldCheck } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { Device } from "@/lib/tauri";

interface TrustDisplay {
    variant: "success" | "outline";
    label: string;
}

function trustDisplay(isTrusted: boolean): TrustDisplay {
    return isTrusted
        ? { variant: "success", label: "Confiable" }
        : { variant: "outline", label: "No confiable" };
}

export interface TrustBadgeProps {
    device: Pick<Device, "is_trusted">;
    className?: string;
}

export function TrustBadge({ device, className }: TrustBadgeProps) {
    const { variant, label } = trustDisplay(device.is_trusted ?? false);

    return (
        <Badge
            variant={variant}
            className={cn("gap-1", className)}
            title={label}
            aria-label={label}
        >
            <ShieldCheck className="h-3 w-3" aria-hidden />
            {label}
        </Badge>
    );
}
