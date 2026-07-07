import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { CardHeader } from "@/components/ui/card-header";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/empty-state";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";
import { RelativeTime } from "@/components/relative-time";
import {
    History,
    Loader2,
    ShieldAlert,
    CircleCheck,
    CircleX,
    RefreshCw,
    Network,
} from "lucide-react";
import { listScans, type ScanSummaryDto } from "@/lib/tauri";

// Scans (ADR-0001 #23, #12): vista de solo-lectura — tabla cronológica de
// escaneos pasados (descubrimiento de red y de puertos por IP). El launch de
// escaneo de puertos vive en DeviceDetail (T15). Click en fila:
// - scan de puertos con target_ip → /devices/:ip (detalle del dispositivo).
// - scan de descubrimiento → /devices (inventario).

function statusBadge(status: string) {
    const s = status.toLowerCase();
    if (s === "completed")
        return (
            <Badge variant="success" className="gap-1">
                <CircleCheck className="h-3 w-3" aria-hidden />
                Completado
            </Badge>
        );
    if (s === "failed")
        return (
            <Badge variant="destructive" className="gap-1">
                <CircleX className="h-3 w-3" aria-hidden />
                Fallido
            </Badge>
        );
    return (
        <Badge variant="secondary" className="gap-1">
            <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
            En curso
        </Badge>
    );
}

// Ruta de detalle asociada a un escaneo: puertos con target → /devices/:ip;
// descubrimiento (o puertos sin target) → /devices.
function scanTargetPath(s: ScanSummaryDto): string {
    if (s.scan_type === "ports" && s.target_ip) {
        return `/devices/${encodeURIComponent(s.target_ip)}`;
    }
    return "/devices";
}

function scanTypeLabel(scanType: string): string {
    return scanType.toLowerCase() === "ports" ? "Puertos" : "Descubrimiento";
}

export function Scans() {
    const navigate = useNavigate();
    const [history, setHistory] = useState<ScanSummaryDto[]>([]);
    const [historyLoading, setHistoryLoading] = useState(true);
    const [historyError, setHistoryError] = useState<string | null>(null);

    async function refreshHistory() {
        setHistoryLoading(true);
        try {
            const rows = await listScans();
            setHistory(rows);
            setHistoryError(null);
        } catch (e) {
            setHistoryError(String(e));
            setHistory([]);
        } finally {
            setHistoryLoading(false);
        }
    }

    useEffect(() => {
        refreshHistory();
    }, []);

    function openDevices() {
        navigate("/devices");
    }

    function openScan(s: ScanSummaryDto) {
        navigate(scanTargetPath(s));
    }

    function handleRowKeyDown(e: React.KeyboardEvent, s: ScanSummaryDto) {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            openScan(s);
        }
    }

    return (
        <div
            className={cn("flex flex-col", SECTION_GAP)}
            aria-busy={historyLoading}
        >
            <section aria-label="Historial de escaneos">
                <Card>
                    <CardHeader variant="toolbar">
                        <CardTitle className="flex items-center gap-2">
                            <History
                                className="h-5 w-5 text-primary"
                                aria-hidden
                            />
                            Historial ({history.length})
                        </CardTitle>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={refreshHistory}
                            disabled={historyLoading}
                        >
                            {historyLoading ? "Cargando…" : "Actualizar"}
                        </Button>
                    </CardHeader>
                    <CardContent>
                        {historyError && (
                            <div role="alert" className="mb-4">
                                <EmptyState
                                    icon={ShieldAlert}
                                    title="No se pudo cargar el historial"
                                    description={historyError}
                                    action={
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={refreshHistory}
                                            className="gap-1.5"
                                        >
                                            <RefreshCw
                                                className="h-4 w-4"
                                                aria-hidden
                                            />
                                            Reintentar
                                        </Button>
                                    }
                                />
                            </div>
                        )}

                        {!historyError && historyLoading && (
                            <div className="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                                <Loader2
                                    className="h-4 w-4 animate-spin"
                                    aria-hidden
                                />
                                Cargando historial…
                            </div>
                        )}

                        {!historyError &&
                            !historyLoading &&
                            history.length === 0 && (
                                <EmptyState
                                    icon={History}
                                    title="Sin escaneos previos"
                                    description="Aún no hay escaneos registrados. Explora los dispositivos de tu red."
                                    action={
                                        <Button
                                            size="sm"
                                            onClick={openDevices}
                                            className="gap-1.5"
                                        >
                                            <Network
                                                className="h-3.5 w-3.5"
                                                aria-hidden
                                            />
                                            Ver dispositivos
                                        </Button>
                                    }
                                />
                            )}

                        {!historyError &&
                            !historyLoading &&
                            history.length > 0 && (
                                <div className="overflow-x-auto rounded-md border border-border">
                                    <table className="w-full caption-bottom text-sm">
                                        <caption className="sr-only">
                                            Historial cronológico de escaneos
                                        </caption>
                                        <thead className="[&_tr]:border-b">
                                            <tr className="border-b">
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Tipo
                                                </th>
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Destino
                                                </th>
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Fecha
                                                </th>
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Perfil
                                                </th>
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Puertos abiertos
                                                </th>
                                                <th
                                                    scope="col"
                                                    className="h-10 px-3 text-left align-middle font-medium text-muted-foreground"
                                                >
                                                    Estado
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody className="[&_tr:last-child]:border-0">
                                            {history.map((s) => {
                                                const isPorts =
                                                    s.scan_type.toLowerCase() ===
                                                    "ports";
                                                const hasDiscovery =
                                                    !isPorts &&
                                                    (s.hosts_alive > 0 ||
                                                        s.hosts_new > 0);
                                                const targetLabel =
                                                    isPorts && s.target_ip
                                                        ? s.target_ip
                                                        : "Red local";
                                                return (
                                                    <tr
                                                        key={s.id}
                                                        className="cursor-pointer border-b transition-colors hover:bg-muted/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                                                        onClick={() =>
                                                            openScan(s)
                                                        }
                                                        tabIndex={0}
                                                        onKeyDown={(e) =>
                                                            handleRowKeyDown(
                                                                e,
                                                                s,
                                                            )
                                                        }
                                                        role="button"
                                                        aria-label={`Abrir ${scanTypeLabel(s.scan_type)} ${s.profile} del ${s.started_at} (${targetLabel})`}
                                                    >
                                                        <td className="p-3 align-middle">
                                                            <Badge
                                                                variant="outline"
                                                                className="capitalize"
                                                            >
                                                                {scanTypeLabel(
                                                                    s.scan_type,
                                                                )}
                                                            </Badge>
                                                        </td>
                                                        <td className="p-3 align-middle">
                                                            <div className="flex flex-col gap-0.5">
                                                                <span className="font-medium text-primary">
                                                                    {
                                                                        targetLabel
                                                                    }
                                                                </span>
                                                                {hasDiscovery && (
                                                                    <span className="text-xs text-muted-foreground">
                                                                        {
                                                                            s.hosts_alive
                                                                        }{" "}
                                                                        vivos ·{" "}
                                                                        {
                                                                            s.hosts_new
                                                                        }{" "}
                                                                        nuevos
                                                                    </span>
                                                                )}
                                                            </div>
                                                        </td>
                                                        <td className="p-3 align-middle text-xs text-muted-foreground">
                                                            <RelativeTime
                                                                value={
                                                                    s.started_at
                                                                }
                                                            />
                                                        </td>
                                                        <td className="p-3 align-middle">
                                                            <Badge
                                                                variant="outline"
                                                                className="capitalize"
                                                            >
                                                                {s.profile}
                                                            </Badge>
                                                        </td>
                                                        <td className="p-3 align-middle text-muted-foreground">
                                                            {isPorts
                                                                ? s.open_ports
                                                                : "—"}
                                                        </td>
                                                        <td className="p-3 align-middle">
                                                            {statusBadge(
                                                                s.status,
                                                            )}
                                                        </td>
                                                    </tr>
                                                );
                                            })}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                    </CardContent>
                </Card>
            </section>
        </div>
    );
}
