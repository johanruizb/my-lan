import { formatRelative, formatTimestamp } from "@/lib/format";
import { cn } from "@/lib/utils";

// Timestamp unificado (AC-12): relativo visible ("hace X") + absoluto en title
// (hover/focus). Estrategia única para las 4 locations: Devices card+table,
// Scans historial (Iniciado/Finalizado) y DeviceDetail "Último visto".
// Reusa lib/format.ts (mismo helper compartido, una sola estrategia de formato).

export interface RelativeTimeProps {
    value: string | number | Date | null | undefined;
    className?: string;
}

export function RelativeTime({ value, className }: RelativeTimeProps) {
    return (
        <span
            title={formatTimestamp(value)}
            className={cn("tabular-nums", className)}
        >
            {formatRelative(value)}
        </span>
    );
}
