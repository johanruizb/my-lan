import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { CardHeader } from "@/components/ui/card-header";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { EmptyState } from "@/components/empty-state";
import { useToast } from "@/components/ui/toast";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { FormField } from "@/components/ui/form-field";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";
import { formatTimestamp } from "@/lib/format";
import {
    Radar,
    Loader2,
    Play,
    Square,
    History,
    ShieldAlert,
    CircleCheck,
    CircleX,
    RefreshCw,
} from "lucide-react";
import {
    cancelScan,
    listDevices,
    listScans,
    onScanCancelled,
    onScanFinished,
    onScanHeartbeat,
    onScanProgress,
    scanPorts,
    type Device,
    type ScanProgress,
    type ScanSummaryDto,
    type Service,
    type UnlistenFn,
} from "@/lib/tauri";

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

export function Scans() {
    const { toast } = useToast();
    const [ip, setIp] = useState("");
    const [profile, setProfile] = useState("normal");

    const [scanning, setScanning] = useState(false);
    const [progress, setProgress] = useState<ScanProgress | null>(null);
    const [elapsed, setElapsed] = useState(0);
    const [scanTimeout, setScanTimeout] = useState(0);
    const [openPorts, setOpenPorts] = useState<Service[]>([]);
    const [scanId, setScanId] = useState<string | null>(null);

    const [history, setHistory] = useState<ScanSummaryDto[]>([]);
    const [historyLoading, setHistoryLoading] = useState(true);
    const [historyError, setHistoryError] = useState<string | null>(null);

    // Selector de dispositivos (AC-5): lista de dispositivos descubiertos para
    // elegir como objetivo del escaneo de puertos. IP manual preservado en
    // Collapsible "avanzado" para IPs no descubiertas.
    const [devices, setDevices] = useState<Device[]>([]);
    const [selectedDeviceId, setSelectedDeviceId] = useState("");
    const [openManualIp, setOpenManualIp] = useState(false);

    const formRef = useRef<HTMLDivElement>(null);

    function scrollToForm() {
        formRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
    }

    async function refreshDevices() {
        try {
            const rows = await listDevices();
            setDevices(rows);
        } catch {
            setDevices([]);
        }
    }

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
        refreshDevices();
    }, []);

    useEffect(() => {
        if (!scanning || !scanId) return;
        const unlisteners: UnlistenFn[] = [];
        let cancelled = false;

        onScanProgress((p) => {
            setProgress(p);
            if (p.latest_open_port) {
                setOpenPorts((prev) =>
                    prev.some((s) => s.port === p.latest_open_port)
                        ? prev
                        : [...prev, { port: p.latest_open_port } as Service],
                );
            }
        }).then((u) => unlisteners.push(u));
        onScanHeartbeat((h) => {
            if (h.scan_id === scanId) {
                setElapsed(h.elapsed_ms);
                setScanTimeout(h.scan_timeout_ms);
            }
        }).then((u) => unlisteners.push(u));
        onScanCancelled((c) => {
            if (c.scan_id === scanId) {
                cancelled = true;
                setScanning(false);
                setScanId(null);
                toast("Escaneo cancelado.", "default");
                refreshHistory();
            }
        }).then((u) => unlisteners.push(u));
        onScanFinished((f) => {
            if (f.scan_id === scanId && !cancelled) {
                setScanning(false);
                setScanId(null);
                toast("Escaneo completado.", "success");
                refreshHistory();
            }
        }).then((u) => unlisteners.push(u));

        return () => {
            unlisteners.forEach((u) => u());
        };
    }, [scanning, scanId]);

    async function handleStart() {
        if (!ip.trim()) {
            toast("Introduce una IP.", "error");
            return;
        }
        const id = newScanId();
        setScanId(id);
        setScanning(true);
        setProgress(null);
        setElapsed(0);
        setOpenPorts([]);
        try {
            const services = await scanPorts(ip.trim(), profile, id);
            setOpenPorts(services);
        } catch (e) {
            setScanning(false);
            setScanId(null);
            toast(`Error: ${e}`, "error");
        }
    }

    async function handleCancel() {
        if (scanId)
            await cancelScan(scanId).catch((e) =>
                toast(`Error: ${e}`, "error"),
            );
    }

    const pct = progress?.percent_done ?? 0;
    const remainMs = scanTimeout > elapsed ? scanTimeout - elapsed : 0;

    return (
        <div
            className={cn("flex flex-col", SECTION_GAP)}
            aria-busy={scanning || historyLoading}
        >
            <div ref={formRef}>
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Radar
                                className="h-5 w-5 text-primary"
                                aria-hidden
                            />
                            Escaneo de puertos
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-4">
                        <div className="flex flex-wrap items-end gap-3">
                            {/* Selector de dispositivos (primario, AC-5): lista los
                            dispositivos descubiertos como objetivo principal. */}
                            <FormField
                                label="Dispositivo a escanear"
                                htmlFor="scan-device"
                                helper="Elige un dispositivo de tu red"
                            >
                                <Select
                                    value={selectedDeviceId}
                                    onValueChange={(v) => {
                                        setSelectedDeviceId(v);
                                        const d = devices.find(
                                            (d) => d.id === v,
                                        );
                                        if (d) setIp(d.primary_ip ?? "");
                                    }}
                                    disabled={scanning}
                                >
                                    <SelectTrigger
                                        id="scan-device"
                                        className="w-64"
                                    >
                                        <SelectValue placeholder="Selecciona un dispositivo" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {devices.length === 0 && (
                                            <div className="p-2 text-xs text-muted-foreground">
                                                No hay dispositivos
                                                descubiertos. Usa IP manual.
                                            </div>
                                        )}
                                        {devices.map((d) => (
                                            <SelectItem
                                                key={d.id}
                                                value={d.id}
                                                disabled={!d.primary_ip}
                                            >
                                                {d.primary_ip ?? d.id} —{" "}
                                                {d.hostname ??
                                                    d.display_name ??
                                                    "Sin nombre"}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </FormField>

                            {/* IP manual (avanzado): preserva IPs arbitrarias no
                            descubiertas (AC-5). Colapsado por defecto. */}
                            <Collapsible
                                open={openManualIp}
                                onOpenChange={setOpenManualIp}
                            >
                                <CollapsibleTrigger className="flex w-fit items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
                                    <ChevronDown
                                        className="h-3.5 w-3.5 transition-transform data-[state=closed]:-rotate-90"
                                        aria-hidden
                                    />
                                    IP manual (avanzado)
                                </CollapsibleTrigger>
                                <CollapsibleContent>
                                    <div className="mt-2">
                                        <FormField
                                            label="IP del host"
                                            htmlFor="scan-ip"
                                            helper="Solo si no aparece en la lista de arriba"
                                        >
                                            <Input
                                                id="scan-ip"
                                                placeholder="192.168.1.10"
                                                value={ip}
                                                onChange={(e) => {
                                                    setIp(e.target.value);
                                                    setSelectedDeviceId("");
                                                }}
                                                className="w-48"
                                                disabled={scanning}
                                            />
                                        </FormField>
                                    </div>
                                </CollapsibleContent>
                            </Collapsible>

                            <FormField label="Perfil" htmlFor="scan-profile">
                                <ProfileSelect
                                    value={profile}
                                    onChange={setProfile}
                                    className="w-40"
                                    id="scan-profile"
                                    disabled={scanning}
                                />
                            </FormField>
                            <Button
                                onClick={handleStart}
                                disabled={scanning}
                                className="gap-1.5"
                            >
                                {scanning ? (
                                    <>
                                        <Loader2
                                            className="h-4 w-4 animate-spin"
                                            aria-hidden
                                        />
                                        Escaneando…
                                    </>
                                ) : (
                                    <>
                                        <Play className="h-4 w-4" aria-hidden />
                                        Iniciar
                                    </>
                                )}
                            </Button>
                            {scanning && (
                                <Button
                                    variant="destructive"
                                    onClick={handleCancel}
                                    className="gap-1.5"
                                >
                                    <Square className="h-4 w-4" aria-hidden />
                                    Cancelar
                                </Button>
                            )}
                        </div>

                        {scanning && (
                            // Live region para progreso/heartbeat/cancel (AC-15).
                            <div
                                className="flex flex-col gap-2"
                                aria-live="polite"
                                aria-atomic="true"
                            >
                                <Progress
                                    value={pct}
                                    indeterminate={pct === 0}
                                />
                                <div className="flex justify-between text-xs text-muted-foreground">
                                    <span>
                                        {progress
                                            ? `${progress.ports_tested}/${progress.ports_total} · ${pct}%`
                                            : "en progreso…"}
                                    </span>
                                    <span>
                                        {Math.round(elapsed / 100) / 10}s /{" "}
                                        {Math.round(remainMs / 100) / 10}s
                                    </span>
                                </div>
                            </div>
                        )}

                        <div>
                            <div className="flex items-center gap-1 text-sm font-medium">
                                <span>
                                    Puertos abiertos ({openPorts.length})
                                </span>
                                <InfoTooltip
                                    term="Puertos abiertos"
                                    glossaryKey="puerto"
                                />
                            </div>
                            <ul
                                className="mt-2 flex flex-wrap gap-2"
                                aria-label="Puertos abiertos"
                            >
                                {openPorts.length === 0 && (
                                    <li className="text-xs text-muted-foreground">
                                        Sin puertos abiertos detectados aún.
                                    </li>
                                )}
                                {openPorts.map((s) => (
                                    <li
                                        key={s.port}
                                        className="rounded-md border border-border bg-muted px-2 py-1 text-xs font-mono"
                                    >
                                        {s.port}
                                    </li>
                                ))}
                            </ul>
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* Historial de scans (AC-11, vía list_scans IPC). */}
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
                                    description="Aún no hay escaneos. Inicia uno para ver puertos abiertos."
                                    action={
                                        <Button
                                            size="sm"
                                            onClick={scrollToForm}
                                            className="gap-1.5"
                                        >
                                            <Play
                                                className="h-3.5 w-3.5"
                                                aria-hidden
                                            />
                                            Iniciar escaneo
                                        </Button>
                                    }
                                />
                            )}

                        {!historyError &&
                            !historyLoading &&
                            history.length > 0 && (
                                <div className="overflow-x-auto rounded-md border border-border">
                                    <table className="w-full caption-bottom text-sm">
                                        <thead className="[&_tr]:border-b">
                                            <tr className="border-b">
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Tipo
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Perfil
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Estado
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Iniciado
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Finalizado
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Vivos
                                                </th>
                                                <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                                                    Nuevos
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody className="[&_tr:last-child]:border-0">
                                            {history.map((s) => (
                                                <tr
                                                    key={s.id}
                                                    className="border-b transition-colors hover:bg-muted/50"
                                                >
                                                    <td className="p-3 align-middle">
                                                        <Badge
                                                            variant={
                                                                s.hosts_alive >
                                                                    0 ||
                                                                s.hosts_new > 0
                                                                    ? "secondary"
                                                                    : "outline"
                                                            }
                                                        >
                                                            {s.hosts_alive >
                                                                0 ||
                                                            s.hosts_new > 0
                                                                ? "Descubrimiento"
                                                                : "Puertos"}
                                                        </Badge>
                                                    </td>
                                                    <td className="p-3 align-middle">
                                                        <Badge
                                                            variant="outline"
                                                            className="capitalize"
                                                        >
                                                            {s.profile}
                                                        </Badge>
                                                    </td>
                                                    <td className="p-3 align-middle">
                                                        {statusBadge(s.status)}
                                                    </td>
                                                    <td className="p-3 align-middle text-xs text-muted-foreground">
                                                        {formatTimestamp(
                                                            s.started_at,
                                                        )}
                                                    </td>
                                                    <td className="p-3 align-middle text-xs text-muted-foreground">
                                                        {formatTimestamp(
                                                            s.finished_at,
                                                        )}
                                                    </td>
                                                    <td className="p-3 align-middle font-medium">
                                                        {s.hosts_alive}
                                                    </td>
                                                    <td className="p-3 align-middle font-medium">
                                                        {s.hosts_new}
                                                    </td>
                                                </tr>
                                            ))}
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
