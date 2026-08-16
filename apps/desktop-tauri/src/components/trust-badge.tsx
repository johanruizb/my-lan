// Indicador de confianza **manual** y binario (Confiable/No confiable) basado
// únicamente en `is_trusted` (ADR-0006). La medición automática 0-100 vive
// aparte en `ConfidenceBadge` como "Certeza"; `TrustBadge` ya no deriva
// estado ni muestra score, y el estado intermedio "Reconocido" se elimina.
//
// Texto plano con ícono, sin pill: reduce el ruido de chips en la UI.
// success (ShieldCheck verde) para Confiable, muted para No confiable.

import { ShieldCheck } from "lucide-react";
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
    const iconClass =
        variant === "success" ? "text-success" : "text-muted-foreground";

    return (
        <span
            className={cn("inline-flex items-center gap-1 text-xs", className)}
            title={label}
            aria-label={label}
        >
            <ShieldCheck className={cn("h-3.5 w-3.5", iconClass)} aria-hidden />
            {label}
        </span>
    );
}
