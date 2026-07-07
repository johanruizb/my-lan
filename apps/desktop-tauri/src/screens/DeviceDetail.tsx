import { useEffect, useMemo, useRef, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/empty-state";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
    deviceIcon,
    deviceLabel,
    isKnownDeviceType,
} from "@/components/device-icons";
import { OnlineBadge } from "@/components/online-badge";
import { TrustBadge } from "@/components/trust-badge";
import {
    ArrowLeft,
    Loader2,
    Play,
    Square,
    Download,
    Info,
    Server,
    Radar,
    CircleCheck,
    CircleX,
    CircleSlash,
    ShieldAlert,
    Cpu,
    Settings,
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
    updateDevice,
    type DeviceDetailDto,
    type ScanProgress,
    type UnlistenFn,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";
import { MaskedValue } from "@/components/masked-value";
import { isSensitive } from "@/lib/censor";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { FormField } from "@/components/ui/form-field";
import { Input } from "@/components/ui/input";
import { formatRelative, formatTimestamp } from "@/lib/format";
import { ConfidenceBadge } from "@/components/confidence-badge";

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

    // Estado local de edición (AC-8, AC-9).
    const [displayName, setDisplayName] = useState(
        detail?.device.display_name ?? "",
    );
    const [isTrusted, setIsTrusted] = useState(
        detail?.device.is_trusted ?? false,
    );
    const [notes, setNotes] = useState(detail?.device.notes ?? "");
    const [saving, setSaving] = useState(false);

    // Preview del estado de confianza derivado (TrustBadge).
    const trustBadgeDevice = useMemo(
        () => ({
            is_trusted: isTrusted,
            confidence: detail?.device.confidence ?? 0,
        }),
        [isTrusted, detail?.device.confidence],
    );

    const scanRef = useRef<HTMLDivElement>(null);

    function scanPortsCta() {
        scanRef.current?.scrollIntoView({
            behavior: "smooth",
            block: "start",
        });
        setTimeout(
            () => document.getElementById("detail-profile")?.focus(),
            100,
        );
    }

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

    // Listeners de progreso/heartbeat/cancel/finish
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
    }, [scanning, scanId, ip]);

    // Resetea el estado de edición solo cuando cambia el dispositivo o sus campos clave
    useEffect(() => {
        if (!detail) return;
        const dev = detail.device;
        setDisplayName((prev) =>
            prev === (dev.display_name ?? "") ? prev : (dev.display_name ?? ""),
        );
        setIsTrusted((prev) =>
            prev === (dev.is_trusted ?? false)
                ? prev
                : (dev.is_trusted ?? false),
        );
        setNotes((prev) =>
            prev === (dev.notes ?? "") ? prev : (dev.notes ?? ""),
        );
    }, [
        detail?.device.id,
        detail?.device.display_name,
        detail?.device.is_trusted,
        detail?.device.notes,
    ]);

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

    async function handleSaveEdit() {
        if (!detail) return;
        const d = detail.device;
        const dirtyName = displayName !== (d.display_name ?? "");
        const dirtyTrusted = isTrusted !== (d.is_trusted ?? false);
        const dirtyNotes = notes !== (d.notes ?? "");
        setSaving(true);
        try {
            await updateDevice(d.id, {
                displayName: dirtyName ? displayName.trim() : undefined,
                isTrusted: dirtyTrusted ? isTrusted : undefined,
                notes: dirtyNotes ? notes.trim() : undefined,
            });
            toast("Dispositivo actualizado.", "success");
            try {
                await getDevice(decodeURIComponent(ip)).then(setDetail);
            } catch {
                toast("Error recargando el dispositivo.", "default");
            }
        } catch {
            toast("No se pudo actualizar el dispositivo.", "error");
        } finally {
            setSaving(false);
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
        <div
            className={cn("flex flex-col gap-6", SECTION_GAP)}
            aria-busy={scanning}
        >
            <div>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => navigate("/devices")}
                    className="w-fit gap-1.5 pl-0 hover:bg-transparent text-muted-foreground hover:text-foreground mb-4"
                >
                    <ArrowLeft className="h-4 w-4" aria-hidden />
                    Volver a dispositivos
                </Button>

                {/* Cabecera Principal del Dispositivo */}
                <div className="glass-panel p-6 rounded-xl border border-border/40 flex flex-col md:flex-row md:items-center justify-between gap-6">
                    <div className="flex items-center gap-4 min-w-0">
                        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20">
                            <Icon className="h-8 w-8" aria-hidden />
                        </div>
                        <div className="flex flex-col min-w-0 gap-1">
                            <h1 className="text-xl font-bold text-foreground truncate">
                                <MaskedValue
                                    field={
                                        d.display_name
                                            ? "display_name"
                                            : d.hostname
                                              ? "hostname"
                                              : d.primary_ip
                                                ? "primary_ip"
                                                : "id"
                                    }
                                    value={
                                        d.display_name ??
                                        d.hostname ??
                                        d.primary_ip ??
                                        d.id
                                    }
                                />
                            </h1>
                            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                                <span className="font-mono bg-muted px-1.5 py-0.5 rounded text-[11px] font-semibold">
                                    <MaskedValue
                                        field="primary_ip"
                                        value={d.primary_ip ?? "—"}
                                    />
                                </span>
                                {d.primary_mac && (
                                    <span className="font-mono uppercase text-[11px]">
                                        <MaskedValue
                                            field="primary_mac"
                                            value={d.primary_mac}
                                            mono
                                        />
                                    </span>
                                )}
                                {d.vendor && (
                                    <span
                                        className="truncate max-w-[150px]"
                                        title={d.vendor}
                                    >
                                        {d.vendor}
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                        {isKnownDeviceType(d.device_type) && (
                            <Badge
                                variant="secondary"
                                className="px-2.5 py-1 text-xs font-semibold"
                            >
                                {deviceLabel(d.device_type)}
                            </Badge>
                        )}
                        <OnlineBadge
                            isOnline={d.is_online}
                            className="px-2.5 py-1 text-xs"
                        />
                        <TrustBadge
                            device={trustBadgeDevice}
                            className="px-2.5 py-1 text-xs"
                        />
                    </div>
                </div>
            </div>

            {/* Layout en Grid Principal */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
                {/* Columna Izquierda: Información y Gestión */}
                <div className="flex flex-col gap-6 lg:col-span-1">
                    {/* Tarjeta: Detalles Técnicos */}
                    <Card className="glass-panel border-border/40">
                        <CardHeader className="pb-3 border-b border-border/10">
                            <CardTitle className="text-sm font-semibold flex items-center gap-2 text-muted-foreground uppercase tracking-wider">
                                <Info
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Información técnica
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="pt-4 flex flex-col gap-4">
                            <Field
                                label="Nombre de red (hostname)"
                                value={d.hostname ?? "—"}
                                field="hostname"
                                glossaryKey="hostname"
                            />
                            {d.display_name && (
                                <Field
                                    label="Nombre personalizado"
                                    value={d.display_name}
                                    field="display_name"
                                />
                            )}
                            <Field
                                label="Dirección MAC"
                                value={d.primary_mac ?? "—"}
                                mono
                                field="primary_mac"
                                glossaryKey="mac"
                            />
                            <Field
                                label="Fabricante"
                                value={d.vendor ?? "—"}
                                glossaryKey="vendor"
                            />
                            <Field
                                label="Familia S.O."
                                value={d.os_family ?? "—"}
                            />
                            <Field
                                label="Primera detección"
                                value={formatTimestamp(d.first_seen_at)}
                            />
                            <Field
                                label="Última detección"
                                value={formatTimestamp(d.last_seen_at)}
                                title={formatRelative(d.last_seen_at)}
                            />

                            {/* Confianza Score */}
                            <div className="flex flex-col gap-1.5 border-t border-border/10 pt-4">
                                <span className="flex items-center gap-1 text-xs text-muted-foreground">
                                    Confianza
                                    <InfoTooltip
                                        term="Confianza"
                                        glossaryKey="confianza"
                                    />
                                </span>
                                <div className="flex items-center gap-3">
                                    <ConfidenceBadge
                                        value={d.confidence}
                                        className="h-6 text-xs font-semibold px-2"
                                    />
                                    <Progress
                                        value={d.confidence}
                                        className="h-1.5 flex-1 bg-muted"
                                    />
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Tarjeta: Gestión */}
                    <Card className="glass-panel border-border/40">
                        <CardHeader className="pb-3 border-b border-border/10">
                            <CardTitle className="text-sm font-semibold flex items-center gap-2 text-muted-foreground uppercase tracking-wider">
                                <Settings
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Gestión y etiquetas
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="pt-4 flex flex-col gap-4">
                            <FormField
                                label="Nombre personalizado"
                                htmlFor="detail-display-name"
                                helper="Etiqueta para identificar el equipo"
                            >
                                <Input
                                    id="detail-display-name"
                                    value={displayName}
                                    onChange={(e) =>
                                        setDisplayName(e.target.value)
                                    }
                                    placeholder={
                                        d.hostname ?? d.primary_ip ?? ""
                                    }
                                    className="bg-background/50 border-border/30"
                                />
                            </FormField>

                            <FormField
                                label="Confiable"
                                htmlFor="detail-is-trusted"
                                helper="Marcar este dispositivo como seguro"
                            >
                                <Button
                                    id="detail-is-trusted"
                                    type="button"
                                    variant={isTrusted ? "default" : "outline"}
                                    onClick={() => setIsTrusted((v) => !v)}
                                    aria-pressed={isTrusted}
                                    className="w-full gap-1.5 justify-center font-medium"
                                >
                                    {isTrusted
                                        ? "Dispositivo confiable"
                                        : "Marcar como confiable"}
                                </Button>
                            </FormField>

                            <FormField
                                label="Notas internas"
                                htmlFor="detail-notes"
                                helper="Notas no compartidas en la red"
                            >
                                <Input
                                    id="detail-notes"
                                    value={notes}
                                    onChange={(e) => setNotes(e.target.value)}
                                    placeholder="Notas de mantenimiento o ubicación..."
                                    className="bg-background/50 border-border/30"
                                />
                            </FormField>

                            <Button
                                onClick={handleSaveEdit}
                                disabled={saving}
                                className="w-full mt-2 gap-1.5"
                            >
                                {saving ? (
                                    <>
                                        <Loader2
                                            className="h-4 w-4 animate-spin"
                                            aria-hidden
                                        />
                                        Guardando…
                                    </>
                                ) : (
                                    "Guardar cambios"
                                )}
                            </Button>
                        </CardContent>
                    </Card>
                </div>

                {/* Columna Derecha: Escaneo y Servicios */}
                <div className="flex flex-col gap-6 lg:col-span-2">
                    {/* Tarjeta: Escaneo de Puertos */}
                    <div ref={scanRef}>
                        <Card className="glass-panel border-border/40">
                            <CardHeader className="pb-3 border-b border-border/10">
                                <CardTitle className="text-sm font-semibold flex items-center gap-2 text-muted-foreground uppercase tracking-wider">
                                    <Radar
                                        className="h-4 w-4 text-primary"
                                        aria-hidden
                                    />
                                    Escaneo de puertos
                                </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-4 flex flex-col gap-4">
                                <div className="flex flex-wrap items-end justify-between gap-4">
                                    <FormField
                                        label="Perfil de escaneo"
                                        htmlFor="detail-profile"
                                        helper="Intensidad del barrido"
                                        className="flex-1 min-w-[200px]"
                                    >
                                        <ProfileSelect
                                            value={profile}
                                            onChange={setProfile}
                                            className="w-full bg-background/50 border-border/30"
                                            id="detail-profile"
                                            disabled={scanning}
                                        />
                                    </FormField>
                                    <div className="flex gap-2 shrink-0">
                                        <Button
                                            onClick={handleScanPorts}
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
                                                className="gap-1.5"
                                            >
                                                <Square
                                                    className="h-4 w-4"
                                                    aria-hidden
                                                />
                                                Cancelar
                                            </Button>
                                        )}
                                    </div>
                                </div>

                                {scanning && (
                                    <div
                                        className="flex flex-col gap-4 border-t border-border/10 pt-4"
                                        aria-live="polite"
                                        aria-atomic="true"
                                    >
                                        <div className="flex flex-col md:flex-row items-center gap-6 justify-center bg-muted/20 p-4 rounded-lg border border-border/10">
                                            {/* Animación Radar Sonar */}
                                            <div className="relative flex items-center justify-center h-16 w-16 shrink-0">
                                                <div className="absolute h-14 w-14 rounded-full border border-primary/20 animate-ping" />
                                                <div className="absolute h-10 w-10 rounded-full border border-primary/40 animate-pulse" />
                                                <Radar
                                                    className="h-6 w-6 text-primary animate-spin"
                                                    style={{
                                                        animationDuration: "3s",
                                                    }}
                                                />
                                            </div>
                                            <div className="flex-1 w-full flex flex-col gap-2">
                                                <Progress
                                                    value={pct}
                                                    indeterminate={pct === 0}
                                                    className="h-2"
                                                />
                                                <div className="flex justify-between text-xs text-muted-foreground font-medium">
                                                    <span>
                                                        {progress
                                                            ? `${progress.ports_tested}/${progress.ports_total} puertos · ${pct}%`
                                                            : "Inicializando escaneo…"}
                                                        {progress?.latest_open_port
                                                            ? ` · último abierto: ${progress.latest_open_port}`
                                                            : ""}
                                                    </span>
                                                    <span>
                                                        {Math.round(
                                                            elapsed / 100,
                                                        ) / 10}
                                                        s /{" "}
                                                        {Math.round(
                                                            remainMs / 100,
                                                        ) / 10}
                                                        s
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>

                    {/* Tarjeta: Servicios */}
                    <Card className="glass-panel border-border/40">
                        <CardHeader className="pb-3 border-b border-border/10 flex flex-row items-center justify-between">
                            <CardTitle className="text-sm font-semibold flex items-center gap-2 text-muted-foreground uppercase tracking-wider">
                                <Server
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Servicios activos ({detail.services.length})
                            </CardTitle>
                            {detail.services.length > 0 && (
                                <div className="flex gap-2">
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() => handleExport("csv")}
                                        className="h-7 gap-1 px-2.5 text-xs"
                                    >
                                        <Download
                                            className="h-3 w-3"
                                            aria-hidden
                                        />
                                        CSV
                                    </Button>
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() => handleExport("json")}
                                        className="h-7 gap-1 px-2.5 text-xs"
                                    >
                                        <Download
                                            className="h-3 w-3"
                                            aria-hidden
                                        />
                                        JSON
                                    </Button>
                                </div>
                            )}
                        </CardHeader>
                        <CardContent className="pt-4">
                            {detail.services.length === 0 ? (
                                <EmptyState
                                    icon={Cpu}
                                    title="Sin servicios"
                                    description="Aún no hay servicios detectados. Realiza un escaneo de puertos en este dispositivo para descubrir servicios activos."
                                    action={
                                        <Button
                                            size="sm"
                                            onClick={scanPortsCta}
                                            className="gap-1.5"
                                        >
                                            <Play
                                                className="h-3.5 w-3.5"
                                                aria-hidden
                                            />
                                            Escanear puertos ahora
                                        </Button>
                                    }
                                />
                            ) : (
                                <div className="overflow-x-auto rounded-md border border-border/40">
                                    <Table>
                                        <TableHeader>
                                            <TableRow className="hover:bg-transparent">
                                                <TableHead>
                                                    <span className="inline-flex items-center gap-1">
                                                        Protocolo
                                                        <InfoTooltip
                                                            term="Protocolo"
                                                            glossaryKey="protocolo"
                                                        />
                                                    </span>
                                                </TableHead>
                                                <TableHead>
                                                    <span className="inline-flex items-center gap-1">
                                                        Puerto
                                                        <InfoTooltip
                                                            term="Puerto"
                                                            glossaryKey="puerto"
                                                        />
                                                    </span>
                                                </TableHead>
                                                <TableHead>
                                                    <span className="inline-flex items-center gap-1">
                                                        Servicio
                                                        <InfoTooltip
                                                            term="Servicio"
                                                            glossaryKey="servicio"
                                                        />
                                                    </span>
                                                </TableHead>
                                                <TableHead>Versión</TableHead>
                                                <TableHead>Estado</TableHead>
                                                <TableHead>
                                                    <span className="inline-flex items-center gap-1">
                                                        Banner
                                                        <InfoTooltip
                                                            term="Banner"
                                                            glossaryKey="banner"
                                                        />
                                                    </span>
                                                </TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {detail.services.map((s) => {
                                                const StateIcon =
                                                    serviceStateIcon(s.state);
                                                return (
                                                    <TableRow
                                                        key={s.id}
                                                        className="hover:bg-muted/30"
                                                    >
                                                        <TableCell className="uppercase font-semibold text-xs text-foreground/80">
                                                            {s.protocol}
                                                        </TableCell>
                                                        <TableCell className="font-mono text-xs font-semibold">
                                                            {s.port}
                                                        </TableCell>
                                                        <TableCell className="text-xs">
                                                            {s.service_name ??
                                                                "—"}
                                                        </TableCell>
                                                        <TableCell className="text-xs text-muted-foreground">
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
                                                                className="gap-1 h-5 px-2 text-[10px] font-semibold"
                                                            >
                                                                <StateIcon
                                                                    className="h-2.5 w-2.5"
                                                                    aria-hidden
                                                                />
                                                                {serviceStateLabel(
                                                                    s.state,
                                                                )}
                                                            </Badge>
                                                        </TableCell>
                                                        <TableCell
                                                            className="font-mono text-xs text-muted-foreground/80 truncate max-w-[150px]"
                                                            title={
                                                                s.banner ?? ""
                                                            }
                                                        >
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
                    </Card>
                </div>
            </div>
        </div>
    );
}

function Field({
    label,
    value,
    mono,
    field,
    glossaryKey,
    title,
}: {
    label: string;
    value: string;
    mono?: boolean;
    field?: string;
    glossaryKey?: string;
    title?: string;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="flex items-center gap-1 text-xs text-muted-foreground">
                {label}
                {glossaryKey && (
                    <InfoTooltip term={label} glossaryKey={glossaryKey} />
                )}
            </span>
            {field && isSensitive(field) ? (
                <MaskedValue field={field} value={value} mono={mono} />
            ) : (
                <span
                    className={cn("font-medium", mono && "font-mono text-xs")}
                    title={title}
                >
                    {value}
                </span>
            )}
        </div>
    );
}
