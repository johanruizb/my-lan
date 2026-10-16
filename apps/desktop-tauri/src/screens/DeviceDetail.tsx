import { useCallback, useEffect, useMemo, useState } from "react";
import { useParams, Link, useBlocker, useBeforeUnload } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/empty-state";
import {
    Dialog,
    DialogContent,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import {
    Collapsible,
    CollapsibleTrigger,
    CollapsibleContent,
} from "@/components/ui/collapsible";
import { CopyButton } from "@/components/copy-button";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
    deviceIcon,
    deviceLabel,
    isKnownDeviceType,
} from "@/components/device-icons";
import { OnlineBadge } from "@/components/online-badge";
import { TrustBadge } from "@/components/trust-badge";
import { Switch } from "@/components/ui/switch";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
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
    ChevronDown,
    Pencil,
    Clock,
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
    notifyScanFinished,
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

function ServiceStateBadge({ state }: { state: string }) {
    const s = state.toLowerCase();
    const StateIcon = serviceStateIcon(s);
    const label = serviceStateLabel(s);

    if (s === "open") {
        return (
            <Badge
                variant="outline"
                className="gap-1 h-5 px-2 text-[10px] font-semibold border-emerald-500/30 text-emerald-700 dark:text-emerald-400 bg-emerald-500/10"
            >
                <StateIcon className="h-2.5 w-2.5" aria-hidden />
                {label}
            </Badge>
        );
    }
    if (s === "closed") {
        return (
            <Badge
                variant="outline"
                className="gap-1 h-5 px-2 text-[10px] font-semibold border-destructive/30 text-red-700 dark:text-red-400 bg-destructive/10"
            >
                <StateIcon className="h-2.5 w-2.5" aria-hidden />
                {label}
            </Badge>
        );
    }
    return (
        <Badge
            variant="outline"
            className="gap-1 h-5 px-2 text-[10px] font-semibold border-amber-500/30 text-amber-800 dark:text-amber-400 bg-amber-500/10"
        >
            <StateIcon className="h-2.5 w-2.5" aria-hidden />
            {label}
        </Badge>
    );
}

export function DeviceDetail() {
    const { ip = "" } = useParams();
    const { toast } = useToast();
    const [detail, setDetail] = useState<DeviceDetailDto | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [profile, setProfile] = useState("normal");

    const [scanning, setScanning] = useState(false);
    const [progress, setProgress] = useState<ScanProgress | null>(null);
    const [elapsed, setElapsed] = useState(0);
    const [scanTimeout, setScanTimeout] = useState(0);
    const [scanId, setScanId] = useState<string | null>(null);

    // Estado local de edición (#19 edición híbrida): is_trusted se persiste al
    // toggle (switch siempre visible, cambio inmediato); display_name + notes
    // usan modo edición (lectura por defecto, "Editar" revela inputs +
    // Guardar/Cancelar). Sin botón "Guardar cambios" global.
    const [displayName, setDisplayName] = useState(
        detail?.device.display_name ?? "",
    );
    const [isTrusted, setIsTrusted] = useState(
        detail?.device.is_trusted ?? false,
    );
    const [notes, setNotes] = useState(detail?.device.notes ?? "");
    const [saving, setSaving] = useState(false);
    const [editing, setEditing] = useState(false);

    // Aviso de cambios sin guardar: si el usuario edita display_name o notes
    // y navega away sin Guardar, useBlocker muestra un dialog de confirmación
    // (Web Interface Guidelines — destructive actions need confirmation).
    const isDirty =
        editing &&
        detail != null &&
        (displayName !== (detail.device.display_name ?? "") ||
            notes !== (detail.device.notes ?? ""));
    const blocker = useBlocker(isDirty);
    useBeforeUnload(
        useCallback(
            (e: BeforeUnloadEvent) => {
                if (isDirty) {
                    e.preventDefault();
                    e.returnValue = "";
                }
            },
            [isDirty],
        ),
    );

    // TrustBadge solo consume is_trusted (ADR-0006 / T6: manual binario).
    const trustBadgeDevice = useMemo(
        () => ({ is_trusted: isTrusted }),
        [isTrusted],
    );

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
                // Notificación OS nativa si la ventana no está enfocada
                // (AC-4/#24). Fallback silencioso al toast si falla (permiso
                // denegado o daemon no disponible): el toast ya avisó.
                if (document.hidden) {
                    const dev = detail?.device;
                    const label =
                        dev?.display_name ??
                        dev?.hostname ??
                        decodeURIComponent(ip);
                    notifyScanFinished(
                        "MyLAN",
                        `Escaneo de puertos completado en ${label}.`,
                    ).catch(() => {});
                }
                getDevice(decodeURIComponent(ip))
                    .then(setDetail)
                    .catch(() => {});
            }
        }).then((u) => unlisteners.push(u));

        return () => {
            unlisteners.forEach((u) => u());
        };
    }, [scanning, scanId, ip, detail, toast]);

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

    useEffect(() => {
        setEditing(false);
    }, [detail?.device.id]);

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

    async function handleTrustToggle(nextTrusted: boolean) {
        if (!detail) return;
        const prev = isTrusted;
        setIsTrusted(nextTrusted);
        try {
            await updateDevice(detail.device.id, { isTrusted: nextTrusted });
            toast(
                nextTrusted
                    ? "Dispositivo marcado como confiable."
                    : "Dispositivo desmarcado.",
                "success",
            );
            getDevice(decodeURIComponent(ip))
                .then(setDetail)
                .catch(() => {});
        } catch {
            setIsTrusted(prev);
            toast("No se pudo actualizar el dispositivo.", "error");
        }
    }

    async function handleSaveEdit() {
        if (!detail) return;
        const d = detail.device;
        const dirtyName = displayName !== (d.display_name ?? "");
        const dirtyNotes = notes !== (d.notes ?? "");
        setSaving(true);
        try {
            await updateDevice(d.id, {
                displayName: dirtyName ? displayName.trim() : undefined,
                notes: dirtyNotes ? notes.trim() : undefined,
            });
            toast("Dispositivo actualizado.", "success");
            setEditing(false);
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

    function handleCancelEdit() {
        if (!detail) return;
        const dev = detail.device;
        setDisplayName(dev.display_name ?? "");
        setNotes(dev.notes ?? "");
        setEditing(false);
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
                    <Link
                        to="/devices"
                        className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background h-8 px-3 text-xs font-medium hover:bg-muted hover:text-foreground transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                        <ArrowLeft className="h-4 w-4" aria-hidden />
                        Volver a dispositivos
                    </Link>
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
    const knownType = isKnownDeviceType(d.device_type);
    const openServices = detail.services.filter(
        (s) => s.state.toLowerCase() === "open",
    );
    const closedOrFilteredServices = detail.services.filter(
        (s) => s.state.toLowerCase() !== "open",
    );

    return (
        <div
            className={cn("flex flex-col", SECTION_GAP)}
            aria-busy={scanning}
        >
            <Link
                to="/devices"
                className="inline-flex w-fit items-center gap-1.5 rounded-md text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
                <ArrowLeft className="h-4 w-4" aria-hidden />
                Volver a dispositivos
            </Link>

            {/* #18: Dossier de identidad sin duplicar MAC/Vendor (que viven en la matriz técnica inferior). */}
            <div className="rounded-xl border border-border bg-card p-5 shadow-xs">
                <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                    <div className="flex items-start sm:items-center gap-4 min-w-0">
                        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20">
                            <Icon className="h-6 w-6" aria-hidden />
                        </div>
                        <div className="flex flex-col min-w-0 gap-1">
                            <div className="flex flex-wrap items-center gap-2">
                                <h1 className="text-xl font-bold tracking-tight text-foreground truncate flex items-center gap-1.5 font-mono tabular-nums">
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
                                    {!d.display_name &&
                                        !d.hostname &&
                                        d.primary_ip && (
                                            <CopyButton
                                                value={d.primary_ip}
                                                label="IP"
                                                className="h-5 w-5"
                                            />
                                        )}
                                </h1>
                                <OnlineBadge isOnline={d.is_online} />
                                <TrustBadge device={trustBadgeDevice} />
                                {knownType && (
                                    <Badge
                                        variant="secondary"
                                        className="h-5 px-2 text-[11px] font-medium"
                                    >
                                        {deviceLabel(d.device_type)}
                                    </Badge>
                                )}
                            </div>

                            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                                {(d.display_name || d.hostname) &&
                                    d.primary_ip && (
                                        <div className="flex items-center gap-1 font-mono bg-muted px-1.5 py-0.5 rounded text-[11px] font-semibold text-foreground/90 tabular-nums">
                                            <MaskedValue
                                                field="primary_ip"
                                                value={d.primary_ip}
                                            />
                                            <CopyButton
                                                value={d.primary_ip}
                                                label="IP"
                                                className="h-4 w-4"
                                            />
                                        </div>
                                    )}
                                <span className="inline-flex items-center gap-1">
                                    <Clock
                                        className="h-3 w-3 text-muted-foreground/70"
                                        aria-hidden
                                    />
                                    Última actividad:{" "}
                                    <strong className="font-medium text-foreground/85">
                                        {formatRelative(d.last_seen_at)}
                                    </strong>
                                </span>
                                <span
                                    className="text-border select-none"
                                    aria-hidden
                                >
                                    ·
                                </span>
                                <span className="tabular-nums">
                                    <strong className="font-mono font-semibold text-foreground/85">
                                        {openServices.length}
                                    </strong>{" "}
                                    {openServices.length === 1
                                        ? "servicio abierto"
                                        : "servicios abiertos"}
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* Control rápido de confianza integrado en la cabecera del host */}
                    <div className="flex flex-wrap items-center justify-between sm:justify-end gap-3 border-t border-border/60 pt-3 lg:border-t-0 lg:pt-0 shrink-0">
                        <div className="flex items-center gap-3 rounded-lg border border-border/70 bg-muted/20 px-3.5 py-2">
                            <div className="flex flex-col">
                                <label
                                    htmlFor="detail-is-trusted"
                                    className="text-xs font-semibold text-foreground cursor-pointer leading-tight"
                                >
                                    Dispositivo confiable
                                </label>
                                <span className="text-[11px] text-muted-foreground leading-tight">
                                    {isTrusted
                                        ? "Marcado como seguro en tu red"
                                        : "Sin verificar manualmente"}
                                </span>
                            </div>
                            <Switch
                                id="detail-is-trusted"
                                checked={isTrusted}
                                onCheckedChange={handleTrustToggle}
                            />
                        </div>
                    </div>
                </div>
            </div>

            {/* Banco de trabajo principal: 5 columnas Ficha Técnica y Gestión | 7 columnas Auditoría de Puertos y Servicios */}
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
                {/* Columna izquierda: Ficha Técnica en cuadrícula 2x3 + Gestión y etiquetas */}
                <div className="flex flex-col gap-6 lg:col-span-5">
                    <Card>
                        <CardHeader className="px-5 py-3.5 border-b border-border/60">
                            <CardTitle className="text-sm font-semibold flex items-center gap-2 text-foreground">
                                <Info
                                    className="h-4 w-4 text-primary shrink-0"
                                    aria-hidden
                                />
                                Información técnica
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-5 flex flex-col gap-3">
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <Field
                                    label="Dirección MAC"
                                    value={d.primary_mac ?? "—"}
                                    mono
                                    field="primary_mac"
                                    glossaryKey="mac"
                                    copyable
                                />
                                <Field
                                    label="Vendor"
                                    value={d.vendor ?? "—"}
                                    glossaryKey="vendor"
                                    title={d.vendor ?? undefined}
                                    copyable
                                />
                                <Field
                                    label="Nombre de red (hostname)"
                                    value={d.hostname ?? "—"}
                                    field="hostname"
                                    glossaryKey="hostname"
                                />
                                <Field
                                    label="Familia S.O."
                                    value={d.os_family ?? "—"}
                                />
                                <Field
                                    label="Primera detección"
                                    value={formatTimestamp(d.first_seen_at)}
                                    mono
                                />
                                <Field
                                    label="Última detección"
                                    value={formatTimestamp(d.last_seen_at)}
                                    title={formatRelative(d.last_seen_at)}
                                    mono
                                />
                            </div>

                            {/* #31: Certeza — barra + porcentaje compactos en fila horizontal */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border border-border/60 bg-muted/15 px-3.5 py-2.5">
                                <span className="flex items-center gap-1 text-xs font-medium text-muted-foreground shrink-0">
                                    Certeza
                                    <InfoTooltip
                                        term="Certeza"
                                        glossaryKey="certeza"
                                    />
                                </span>
                                <div className="flex-1 max-w-[220px]">
                                    <ConfidenceBadge
                                        value={d.confidence}
                                        showLabel={false}
                                    />
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="px-5 py-3.5 border-b border-border/60 flex flex-row items-center justify-between gap-2">
                            <CardTitle className="text-sm font-semibold flex items-center gap-2 text-foreground">
                                <Settings
                                    className="h-4 w-4 text-primary shrink-0"
                                    aria-hidden
                                />
                                Gestión y etiquetas
                            </CardTitle>
                            {!editing && (
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => setEditing(true)}
                                    className="h-7 px-2.5 text-xs gap-1.5"
                                >
                                    <Pencil
                                        className="h-3 w-3"
                                        aria-hidden
                                    />
                                    Editar
                                </Button>
                            )}
                        </CardHeader>
                        <CardContent className="p-5 flex flex-col gap-4">
                            {/* #19: display_name + notes — modo edición (lectura por
                                defecto, botón "Editar" revela inputs + Guardar/Cancelar). */}
                            {editing ? (
                                <>
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
                                            name="display-name"
                                            autoComplete="off"
                                        />
                                    </FormField>

                                    <FormField
                                        label="Notas internas"
                                        htmlFor="detail-notes"
                                        helper="Notas no compartidas en la red"
                                    >
                                        <Input
                                            id="detail-notes"
                                            value={notes}
                                            onChange={(e) =>
                                                setNotes(e.target.value)
                                            }
                                            placeholder="Notas de mantenimiento o ubicación…"
                                            name="device-notes"
                                            autoComplete="off"
                                        />
                                    </FormField>

                                    <div className="flex gap-2 pt-1">
                                        <Button
                                            size="sm"
                                            onClick={handleSaveEdit}
                                            disabled={saving}
                                            className="gap-1.5"
                                        >
                                            {saving ? (
                                                <>
                                                    <Loader2
                                                        className="h-3.5 w-3.5 animate-spin"
                                                        aria-hidden
                                                    />
                                                    Guardando…
                                                </>
                                            ) : (
                                                "Guardar"
                                            )}
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="outline"
                                            onClick={handleCancelEdit}
                                            disabled={saving}
                                            className="gap-1.5"
                                        >
                                            Cancelar
                                        </Button>
                                    </div>
                                </>
                            ) : (
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <Field
                                        label="Nombre personalizado"
                                        value={d.display_name ?? "—"}
                                        field="display_name"
                                    />
                                    <Field
                                        label="Notas internas"
                                        value={d.notes ?? "—"}
                                    />
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                {/* Columna derecha: Consola unificada de Escaneo de Puertos y Servicios */}
                <div className="flex flex-col gap-4 lg:col-span-7">
                    <Card>
                        <CardHeader className="px-5 py-4 border-b border-border/60 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                            <div className="flex flex-col gap-0.5">
                                <CardTitle className="text-sm font-semibold flex items-center gap-2 text-foreground">
                                    <Server
                                        className="h-4 w-4 text-primary shrink-0"
                                        aria-hidden
                                    />
                                    Servicios abiertos ({openServices.length})
                                </CardTitle>
                                <p className="text-xs text-muted-foreground">
                                    Sondeo de puertos TCP y detección de
                                    servicios en este host.
                                </p>
                            </div>

                            {/* Barra de control de escaneo de puertos integrada */}
                            <div className="flex flex-wrap items-center gap-2">
                                <label
                                    htmlFor="detail-profile"
                                    className="sr-only"
                                >
                                    Perfil de escaneo
                                </label>
                                <ProfileSelect
                                    value={profile}
                                    onChange={setProfile}
                                    className="h-8 w-[150px] bg-background text-xs"
                                    id="detail-profile"
                                    disabled={scanning}
                                />
                                <Button
                                    size="sm"
                                    onClick={handleScanPorts}
                                    disabled={scanning}
                                    className="h-8 gap-1.5"
                                >
                                    {scanning ? (
                                        <>
                                            <Loader2
                                                className="h-3.5 w-3.5 animate-spin"
                                                aria-hidden
                                            />
                                            Escaneando…
                                        </>
                                    ) : (
                                        <>
                                            <Play
                                                className="h-3.5 w-3.5"
                                                aria-hidden
                                            />
                                            Escanear puertos
                                        </>
                                    )}
                                </Button>
                                {scanning && (
                                    <Button
                                        size="sm"
                                        variant="destructive"
                                        onClick={handleCancel}
                                        className="h-8 gap-1.5"
                                    >
                                        <Square
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Cancelar
                                    </Button>
                                )}
                                {/* #27: 1 botón Exportar + dropdown CSV/JSON */}
                                {detail.services.length > 0 && (
                                    <DropdownMenu>
                                        <DropdownMenuTrigger asChild>
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                className="h-8 gap-1.5"
                                            >
                                                <Download
                                                    className="h-3.5 w-3.5"
                                                    aria-hidden
                                                />
                                                Exportar
                                                <ChevronDown
                                                    className="h-3.5 w-3.5"
                                                    aria-hidden
                                                />
                                            </Button>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent align="end">
                                            <DropdownMenuLabel>
                                                Formato de exportación
                                            </DropdownMenuLabel>
                                            <DropdownMenuSeparator />
                                            <DropdownMenuItem
                                                onSelect={() =>
                                                    handleExport("csv")
                                                }
                                            >
                                                CSV
                                            </DropdownMenuItem>
                                            <DropdownMenuItem
                                                onSelect={() =>
                                                    handleExport("json")
                                                }
                                            >
                                                JSON
                                            </DropdownMenuItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                )}
                            </div>
                        </CardHeader>

                        {/* Telemetría en vivo durante el escaneo de puertos */}
                        {scanning && (
                            <div
                                className="border-b border-border/60 bg-primary/5 px-5 py-3.5"
                                aria-live="polite"
                                aria-atomic="true"
                            >
                                <div className="flex items-center gap-3.5">
                                    <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 border border-primary/20">
                                        <Radar
                                            className="h-4 w-4 text-primary motion-safe:animate-spin"
                                            style={{
                                                animationDuration: "3s",
                                            }}
                                            aria-hidden
                                        />
                                    </div>
                                    <div className="flex-1 min-w-0 flex flex-col gap-1.5">
                                        <Progress
                                            value={pct}
                                            indeterminate={pct === 0}
                                            className="h-1.5"
                                        />
                                        <div className="flex justify-between text-xs text-muted-foreground font-medium tabular-nums">
                                            <span>
                                                {progress
                                                    ? `${progress.ports_tested}/${progress.ports_total} puertos · ${pct}%`
                                                    : "Inicializando escaneo…"}
                                                {progress?.latest_open_port
                                                    ? ` · último abierto: ${progress.latest_open_port}`
                                                    : ""}
                                            </span>
                                            <span className="font-mono">
                                                {Math.round(elapsed / 100) / 10}
                                                s /{" "}
                                                {Math.round(remainMs / 100) /
                                                    10}
                                                s
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        <CardContent className="p-5">
                            {detail.services.length === 0 ? (
                                <EmptyState
                                    icon={Cpu}
                                    title="Sin servicios"
                                    description="Aún no hay servicios detectados. Selecciona un perfil y pulsa Escanear puertos en la barra superior para identificar servicios activos en este dispositivo."
                                />
                            ) : openServices.length === 0 ? (
                                <div className="flex flex-col items-center justify-center py-8 px-4 text-center text-sm text-muted-foreground">
                                    <ShieldAlert
                                        className="h-8 w-8 text-muted-foreground/60 mb-2"
                                        aria-hidden
                                    />
                                    <p className="font-medium text-foreground">
                                        Sin servicios abiertos
                                    </p>
                                    <p className="text-xs text-muted-foreground mt-1 max-w-sm">
                                        No se detectaron puertos abiertos en
                                        este dispositivo durante el último
                                        escaneo.
                                    </p>
                                </div>
                            ) : (
                                <div className="overflow-x-auto rounded-lg border border-border/60">
                                    <Table>
                                        <TableHeader>
                                            <TableRow className="hover:bg-transparent">
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
                                                        Protocolo
                                                        <InfoTooltip
                                                            term="Protocolo"
                                                            glossaryKey="protocolo"
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
                                                <TableHead>Estado</TableHead>
                                                <TableHead>Versión</TableHead>
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
                                            {openServices.map((s) => {
                                                return (
                                                    <TableRow
                                                        key={s.id}
                                                        className="hover:bg-muted/30"
                                                    >
                                                        <TableCell className="font-mono text-xs font-semibold tabular-nums">
                                                            {s.port}
                                                        </TableCell>
                                                        <TableCell className="uppercase font-semibold text-xs text-foreground/80">
                                                            {s.protocol}
                                                        </TableCell>
                                                        <TableCell className="text-xs font-medium">
                                                            {s.service_name ??
                                                                "—"}
                                                        </TableCell>
                                                        <TableCell>
                                                            <ServiceStateBadge
                                                                state={s.state}
                                                            />
                                                        </TableCell>
                                                        <TableCell className="text-xs text-muted-foreground">
                                                            {s.version ?? "—"}
                                                        </TableCell>
                                                        <TableCell>
                                                            <BannerCell
                                                                banner={
                                                                    s.banner
                                                                }
                                                            />
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

                    {closedOrFilteredServices.length > 0 && (
                        <Collapsible className="rounded-xl border border-border bg-card overflow-hidden">
                            <CollapsibleTrigger className="flex w-full items-center justify-between px-5 py-3.5 text-sm font-medium hover:bg-muted/40 transition-colors [&[data-state=open]>svg]:rotate-180">
                                <span className="flex items-center gap-2 font-semibold text-muted-foreground">
                                    <ShieldAlert
                                        className="h-4 w-4 text-muted-foreground"
                                        aria-hidden
                                    />
                                    Puertos cerrados o filtrados (
                                    {closedOrFilteredServices.length})
                                </span>
                                <ChevronDown
                                    className="h-4 w-4 text-muted-foreground transition-transform duration-200"
                                    aria-hidden
                                />
                            </CollapsibleTrigger>
                            <CollapsibleContent className="px-5 pb-5 pt-0">
                                <div className="overflow-x-auto rounded-lg border border-border/60 mt-2">
                                    <Table>
                                        <TableHeader>
                                            <TableRow className="hover:bg-transparent">
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
                                                        Protocolo
                                                        <InfoTooltip
                                                            term="Protocolo"
                                                            glossaryKey="protocolo"
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
                                                <TableHead>Estado</TableHead>
                                                <TableHead>Versión</TableHead>
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
                                            {closedOrFilteredServices.map(
                                                (s) => {
                                                    return (
                                                        <TableRow
                                                            key={s.id}
                                                            className="hover:bg-muted/30"
                                                        >
                                                            <TableCell className="font-mono text-xs font-semibold tabular-nums">
                                                                {s.port}
                                                            </TableCell>
                                                            <TableCell className="uppercase font-semibold text-xs text-foreground/80">
                                                                {s.protocol}
                                                            </TableCell>
                                                            <TableCell className="text-xs">
                                                                {s.service_name ??
                                                                    "—"}
                                                            </TableCell>
                                                            <TableCell>
                                                                <ServiceStateBadge
                                                                    state={
                                                                        s.state
                                                                    }
                                                                />
                                                            </TableCell>
                                                            <TableCell className="text-xs text-muted-foreground">
                                                                {s.version ??
                                                                    "—"}
                                                            </TableCell>
                                                            <TableCell>
                                                                <BannerCell
                                                                    banner={
                                                                        s.banner
                                                                    }
                                                                />
                                                            </TableCell>
                                                        </TableRow>
                                                    );
                                                },
                                            )}
                                        </TableBody>
                                    </Table>
                                </div>
                            </CollapsibleContent>
                        </Collapsible>
                    )}
                </div>
            </div>

            {/* Dialog de confirmación: cambios sin guardar antes de navegar */}
            {blocker.state === "blocked" && (
                <Dialog
                    open
                    onOpenChange={(o) => {
                        if (!o) blocker.reset();
                    }}
                >
                    <DialogContent className="max-w-sm">
                        <DialogTitle>¿Descartar cambios?</DialogTitle>
                        <DialogDescription>
                            Tienes cambios sin guardar en este dispositivo. Si
                            navegas ahora, se perderán.
                        </DialogDescription>
                        <div className="flex justify-end gap-2">
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => blocker.reset()}
                            >
                                Seguir editando
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => blocker.proceed()}
                            >
                                Descartar y salir
                            </Button>
                        </div>
                    </DialogContent>
                </Dialog>
            )}
        </div>
    );
}

function BannerCell({ banner }: { banner: string | null }) {
    const [expanded, setExpanded] = useState(false);
    if (!banner)
        return <span className="text-xs text-muted-foreground">—</span>;

    const isLong = banner.length > 25;

    return (
        <div
            className={cn(
                "flex items-start min-w-0 gap-1.5 font-mono text-xs text-muted-foreground/90",
                expanded ? "max-w-xl" : "max-w-[320px]",
            )}
        >
            <span
                className={cn(
                    "min-w-0 transition-all",
                    !expanded &&
                        isLong &&
                        "truncate max-w-[220px] inline-block",
                    expanded && "break-all whitespace-pre-wrap",
                )}
                title={banner}
            >
                {banner}
            </span>
            <div className="flex items-center shrink-0 gap-0.5 mt-0.5">
                {isLong && (
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            setExpanded(!expanded);
                        }}
                        className="text-[10px] px-1 py-0.5 rounded text-primary hover:bg-muted font-sans font-medium"
                        title={expanded ? "Contraer banner" : "Expandir banner"}
                    >
                        {expanded ? "Menos" : "Más"}
                    </button>
                )}
                <CopyButton value={banner} label="Banner" className="h-5 w-5" />
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
    copyable,
}: {
    label: string;
    value: string;
    mono?: boolean;
    field?: string;
    glossaryKey?: string;
    title?: string;
    copyable?: boolean;
}) {
    return (
        <div className="flex flex-col gap-1 rounded-lg border border-border/60 bg-muted/15 px-3 py-2.5 min-w-0">
            <span className="flex items-center gap-1 text-[11px] font-medium text-muted-foreground">
                {label}
                {glossaryKey && (
                    <InfoTooltip term={label} glossaryKey={glossaryKey} />
                )}
            </span>
            <div className="flex items-center justify-between gap-1.5 min-w-0">
                <div className="min-w-0 truncate text-sm">
                    {field && isSensitive(field) ? (
                        <MaskedValue field={field} value={value} mono={mono} />
                    ) : (
                        <span
                            className={cn(
                                "font-medium text-foreground break-words",
                                mono && "font-mono text-xs tabular-nums",
                            )}
                            title={title ?? (value !== "—" ? value : undefined)}
                        >
                            {value}
                        </span>
                    )}
                </div>
                {copyable && value && value !== "—" && (
                    <CopyButton
                        value={value}
                        label={label}
                        className="h-5 w-5 shrink-0"
                    />
                )}
            </div>
        </div>
    );
}
