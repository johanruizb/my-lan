// AC-13: helper compartido `deriveTrustState` — fuente única del estado de
// confianza derivado (Confiable/Reconocido/Desconocido) consumido por
// `TrustBadge`, `Devices.tsx`, `DeviceDetail.tsx` y filtros (AC-14).
// Sin impacto backend, sin migración: es vista derivada de `is_trusted` +
// `confidence` (la fuente sigue siendo el struct `Device`).
//
// `Number(confidence)` maneja string ("70") o number (70); `NaN >= 50` es
// `false` → "unknown" (caso `confidence: "high"` no numérico del backend
// legacy). Umbral 50 alinea con `ConfidenceBadge.classify`
// (`confidence-badge.tsx:19`).

import type { Device } from "@/lib/tauri";

export type TrustState = "trusted" | "recognized" | "unknown";

export function deriveTrustState(
    device: Pick<Device, "is_trusted" | "confidence">,
): TrustState {
    if (device.is_trusted) return "trusted";
    return Number(device.confidence) >= 50 ? "recognized" : "unknown";
}
