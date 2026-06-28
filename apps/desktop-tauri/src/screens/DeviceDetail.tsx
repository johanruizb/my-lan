import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { EmptyState } from "@/components/empty-state";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import { deviceIcon, deviceLabel } from "@/components/device-icons";
import {
    ArrowLeft,
    Loader2,
    Play,
    Square,
    Download,
    ChevronDown,
    Info,
    Server,
    Radar,
    CircleCheck,
    CircleX,
    CircleSlash,
    ShieldAlert,
    Cpu,
} from "lucide-react";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import {
    cancelScan,
    exportServices,
    getDevice,
    onScanCancelled,
    onScanFinished,
    onScanHeartbeat,
    onScanProgress,
    scanPorts,
    type DeviceDetailDto,
    type ScanProgress,
    type UnlistenFn,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";

// Iconos de estado de servicio (open/closed/filtered — AC-5).
function serviceStateIcon(state: string) {
    const s = state.toLowerCase();
    if (s === "open") return CircleCheck;
    if (s === "closed") return CircleX;
    return CircleSlash;
}

function serviceStateLabel(state: string) {
    const s = state.toLowerCase();
    if (s === "open") return "Abierto";
    if (s === "closed") return "Cerrado";
    return "Filtrado";
}

export function DeviceDetail() {
    const { ip = "" } = useParams();
    const navigate = useNavigate();
    const { toast } = useToast();
    const [detail, setDetail] = useState<DeviceDetailDto | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [profile, setProfile] = useState("normal");

    const [scanning, setScanning] = useState(false);
    const [progress, setProgress] = useState<ScanProgress | null>(null);
    const [elapsed, setElapsed] = useState(0);
    const [scanTimeout, setScanTimeout] = useState(0);
    const [scanId, setScanId] = useState<string | null>(null);
    const [openInfo, setOpenInfo] = useState(true);
    const [openScan, setOpenScan] = useState(true);
    const [openServices, setOpenServices] = useState(true);

    useEffect(() => {
        let alive = true;
        getDevice(decodeURIComponent(ip))
            .then((d) => {
                if (alive) {
                    setDetail(d);
                    setError(null);
                }
            })
            .catch((e) => {
                if (alive) setError(String(e));
            });
        return () => {
            alive = false;
        };
    }, [ip]);

    // Listeners de progreso/heartbeat/cancel/finish sólo mientras escanea.
    useEffect(() => {
        if (!scanning || !scanId) return;
        const unlisteners: UnlistenFn[] = [];
        let cancelled = false;

        onScanProgress((p) => {
            setProgress(p);
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
            }
        }).then((u) => unlisteners.push(u));
        onScanFinished((f) => {
            if (f.scan_id === scanId && !cancelled) {
                setScanning(false);
                setScanId(null);
                toast("Escaneo de puertos completado.", "success");
                getDevice(decodeURIComponent(ip))
                    .then(setDetail)
                    .catch(() => {});
            }
        }).then((u) => unlisteners.push(u));

        return () => {
            unlisteners.forEach((u) => u());
        };
    }, [scanning, scanId]);

    async function handleScanPorts() {
        const id = newScanId();
        setScanId(id);
        setScanning(true);
        setProgress(null);
        setElapsed(0);
        try {
            await scanPorts(decodeURIComponent(ip), profile, id);
        } catch (e) {
            setScanning(false);
            setScanId(null);
            toast(`Error: ${e}`, "error");
        }
    }

    async function handleCancel() {
        if (scanId) {
            try {
                await cancelScan(scanId);
            } catch (e) {
                toast(`Error cancelando: ${e}`, "error");
            }
        }
    }

    async function handleExport(format: string) {
        try {
            const path = await exportServices(format);
            toast(`Servicios exportados a: ${path}`, "success");
        } catch (e) {
            toast(`Error exportando: ${e}`, "error");
        }
    }

    const pct = progress?.percent_done ?? 0;
    const remainMs = scanTimeout > elapsed ? scanTimeout - elapsed : 0;

    if (error)
        return (
            <EmptyState
                icon={ShieldAlert}
                title="No se pudo cargar el dispositivo"
                description={error}
                action={
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => navigate("/devices")}
                        className="gap-1.5"
                    >
                        <ArrowLeft className="h-4 w-4" aria-hidden />
                        Volver a dispositivos
                    </Button>
                }
            />
        );
    if (!detail)
        return (
            <div className="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                Cargando dispositivo…
            </div>
        );

    const d = detail.device;
    const Icon = deviceIcon(d.device_type);

    return (
        <div className="flex flex-col gap-4" aria-busy={scanning}>
            <Button
                variant="ghost"
                size="sm"
                onClick={() => navigate("/devices")}
                className="w-fit gap-1.5"
            >
                <ArrowLeft className="h-4 w-4" aria-hidden />
                Volver a dispositivos
            </Button>

            {/* Sección colapsable: info del dispositivo (AC-10). */}
            <Collapsible open={openInfo} onOpenChange={setOpenInfo}>
                <Card>
                    <CollapsibleTrigger asChild>
                        <CardHeader>
                            <CardTitle className="flex w-full items-center gap-2">
                                <Info
                                    className="h-5 w-5 text-primary"
                                    aria-hidden
                                />
                                <span className="flex items-center gap-2">
                                    <Icon className="h-4 w-4" aria-hidden />
                                    {d.primary_ip ?? d.id}
                                </span>
                                <Badge variant="secondary" className="ml-1">
                                    {deviceLabel(d.device_type)}
                                </Badge>
                                <ChevronDown
                                    className={cn(
                                        "ml-auto h-4 w-4 transition-transform",
                                        openInfo ? "" : "-rotate-90",
                                    )}
                                    aria-hidden
                                />
                            </CardTitle>
                        </CardHeader>
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                        <CardContent className="grid gap-3 sm:grid-cols-2">
                            <Field
                                label="MAC"
                                value={d.primary_mac ?? "—"}
                                mono
                            />
                            <Field
                                label="Hostname"
                                value={d.hostname ?? d.display_name ?? "—"}
                            />
                            <Field label="Vendor" value={d.vendor ?? "—"} />
                            <Field
                                label="Tipo"
                                value={deviceLabel(d.device_type)}
                            />
                            <Field label="Confianza" value={d.confidence} />
                            <Field
                                label="Último visto"
                                value={d.last_seen_at}
                            />
                        </CardContent>
                    </CollapsibleContent>
                </Card>
            </Collapsible>

            {/* Sección colapsable: escaneo de puertos (AC-10). */}
            <Collapsible open={openScan} onOpenChange={setOpenScan}>
                <Card>
                    <CollapsibleTrigger asChild>
                        <CardHeader>
                            <CardTitle className="flex w-full items-center gap-2">
                                <Radar
                                    className="h-5 w-5 text-primary"
                                    aria-hidden
                                />
                                Escaneo de puertos
                                <ChevronDown
                                    className={cn(
                                        "ml-auto h-4 w-4 transition-transform",
                                        openScan ? "" : "-rotate-90",
                                    )}
                                    aria-hidden
                                />
                            </CardTitle>
                        </CardHeader>
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                        <CardContent className="flex flex-col gap-4">
                            <div className="flex flex-wrap items-center gap-3">
                                <div className="flex flex-col gap-1.5">
                                    <label
                                        htmlFor="detail-profile"
                                        className="text-xs text-muted-foreground"
                                    >
                                        Perfil
                                    </label>
                                    <ProfileSelect
                                        value={profile}
                                        onChange={setProfile}
                                        className="w-40"
                                        id="detail-profile"
                                        disabled={scanning}
                                    />
                                </div>
                                <Button
                                    onClick={handleScanPorts}
                                    disabled={scanning}
                                    className="mt-5 gap-1.5"
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
                                            <Play
                                                className="h-4 w-4"
                                                aria-hidden
                                            />
                                            Escanear puertos
                                        </>
                                    )}
                                </Button>
                                {scanning && (
                                    <Button
                                        variant="destructive"
                                        onClick={handleCancel}
                                        className="mt-5 gap-1.5"
                                    >
                                        <Square
                                            className="h-4 w-4"
                                            aria-hidden
                                        />
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
                                                ? `${progress.ports_tested}/${progress.ports_total} puertos · ${pct}%`
                                                : "en progreso…"}
                                            {progress?.latest_open_port
                                                ? ` · último abierto: ${progress.latest_open_port}`
                                                : ""}
                                        </span>
                                        <span>
                                            {Math.round(elapsed / 100) / 10}s /{" "}
                                            {Math.round(remainMs / 100) / 10}s
                                        </span>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </CollapsibleContent>
                </Card>
            </Collapsible>

            {/* Sección colapsable: servicios (AC-10). */}
            <Collapsible open={openServices} onOpenChange={setOpenServices}>
                <Card>
                    <CollapsibleTrigger asChild>
                        <CardHeader>
                            <CardTitle className="flex w-full items-center gap-2">
                                <Server
                                    className="h-5 w-5 text-primary"
                                    aria-hidden
                                />
                                Servicios ({detail.services.length})
                                <ChevronDown
                                    className={cn(
                                        "ml-auto h-4 w-4 transition-transform",
                                        openServices ? "" : "-rotate-90",
                                    )}
                                    aria-hidden
                                />
                            </CardTitle>
                        </CardHeader>
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                        <CardContent>
                            <div className="mb-4 flex gap-2">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => handleExport("csv")}
                                    className="gap-1.5"
                                >
                                    <Download
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    CSV
                                </Button>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => handleExport("json")}
                                    className="gap-1.5"
                                >
                                    <Download
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    JSON
                                </Button>
                            </div>
                            {detail.services.length === 0 ? (
                                <EmptyState
                                    icon={Cpu}
                                    title="Sin servicios"
                                    description="Ejecuta un escaneo de puertos para descubrir los servicios de este dispositivo."
                                />
                            ) : (
                                <div className="overflow-x-auto rounded-md border border-border">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>Protocolo</TableHead>
                                                <TableHead>Puerto</TableHead>
                                                <TableHead>Servicio</TableHead>
                                                <TableHead>Producto</TableHead>
                                                <TableHead>Versión</TableHead>
                                                <TableHead>Estado</TableHead>
                                                <TableHead>Banner</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {detail.services.map((s) => {
                                                const StateIcon =
                                                    serviceStateIcon(s.state);
                                                return (
                                                    <TableRow key={s.id}>
                                                        <TableCell className="uppercase">
                                                            {s.protocol}
                                                        </TableCell>
                                                        <TableCell className="font-mono">
                                                            {s.port}
                                                        </TableCell>
                                                        <TableCell>
                                                            {s.service_name ??
                                                                "—"}
                                                        </TableCell>
                                                        <TableCell>
                                                            {s.product ?? "—"}
                                                        </TableCell>
                                                        <TableCell>
                                                            {s.version ?? "—"}
                                                        </TableCell>
                                                        <TableCell>
                                                            <Badge
                                                                variant={
                                                                    s.state.toLowerCase() ===
                                                                    "open"
                                                                        ? "success"
                                                                        : "outline"
                                                                }
                                                                className="gap-1"
                                                            >
                                                                <StateIcon
                                                                    className="h-3 w-3"
                                                                    aria-hidden
                                                                />
                                                                {serviceStateLabel(
                                                                    s.state,
                                                                )}
                                                            </Badge>
                                                        </TableCell>
                                                        <TableCell className="font-mono text-xs">
                                                            {s.banner ?? "—"}
                                                        </TableCell>
                                                    </TableRow>
                                                );
                                            })}
                                        </TableBody>
                                    </Table>
                                </div>
                            )}
                        </CardContent>
                    </CollapsibleContent>
                </Card>
            </Collapsible>
        </div>
    );
}

function Field({
    label,
    value,
    mono,
}: {
    label: string;
    value: string;
    mono?: boolean;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="text-xs text-muted-foreground">{label}</span>
            <span className={cn("font-medium", mono && "font-mono text-xs")}>
                {value}
            </span>
        </div>
    );
}
