import { useEffect, useMemo, useRef, useState } from "react";
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
import { OnlineBadge } from "@/components/online-badge";
import { TrustBadge } from "@/components/trust-badge";
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
    const [openInfo, setOpenInfo] = useState(true);
    const [openScan, setOpenScan] = useState(true);
    const [openServices, setOpenServices] = useState(true);

    // Estado local de edición (AC-8, AC-9). Debe declararse antes de los
    // early-returns para respetar las reglas de hooks. Se resetea cuando
    // `detail` cambia (re-fetch tras guardar) vía el effect más abajo.
    const [displayName, setDisplayName] = useState(
        detail?.device.display_name ?? "",
    );
    const [isTrusted, setIsTrusted] = useState(
        detail?.device.is_trusted ?? false,
    );
    const [notes, setNotes] = useState(detail?.device.notes ?? "");
    const [saving, setSaving] = useState(false);

    // Preview del estado de confianza derivado (TrustBadge). Memoizado para
    // mantener referencia estable y no derrotar React.memo en re-renders por
    // edits no relacionados (fix review #5). Declarado antes de los early-returns
    // para respetar el orden de hooks (igual que los useState de edición).
    const trustBadgeDevice = useMemo(
        () => ({
            is_trusted: isTrusted,
            confidence: detail?.device.confidence ?? "0",
        }),
        [isTrusted, detail?.device.confidence],
    );

    const scanRef = useRef<HTMLDivElement>(null);

    function scanPortsCta() {
        setOpenScan(true);
        scanRef.current?.scrollIntoView({
            behavior: "smooth",
            block: "start",
        });
        // Enfoca el primer control del formulario tras la animación de apertura.
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

    // Resetea el estado de edición solo cuando cambia la identidad del device
    // (no en cada refresh de `detail`, p.ej. online-status), con guards para
    // no descartar edits en progreso ni disparar re-renders redundantes. El
    // fallback `?? false` en is_trusted iguala el inicializador de useState
    // (fix review #3/#6).
    useEffect(() => {
        if (!detail) return;
        const dev = detail.device;
        setDisplayName((prev) =>
            prev === (dev.display_name ?? "") ? prev : dev.display_name ?? "",
        );
        setIsTrusted((prev) =>
            prev === (dev.is_trusted ?? false) ? prev : dev.is_trusted ?? false,
        );
        setNotes((prev) =>
            prev === (dev.notes ?? "") ? prev : dev.notes ?? "",
        );
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

    // Edición por `d.id` (UUID), no por hostname/IP (AC-9): un dispositivo
    // sin hostname es editable. Se trackea dirty-state por campo (vs valor
    // inicial del device): un campo modificado se envía siempre (incluso
    // vacío → backend `Some("")` limpia); un campo sin tocar se envía
    // `undefined` (backend `None` = no sobrescribe). Así el usuario puede
    // revertir un nombre personalizado a vacío (fix review MEDIUM).
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
        <div className={cn("flex flex-col", SECTION_GAP)} aria-busy={scanning}>
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
                                    <MaskedValue
                                        field="primary_ip"
                                        value={d.primary_ip ?? d.id}
                                    />
                                </span>
                                <Badge variant="secondary" className="ml-1">
                                    {deviceLabel(d.device_type)}
                                </Badge>
                                <OnlineBadge
                                    isOnline={d.is_online}
                                    className="ml-1"
                                />
                                <ChevronDown
                                    className="ml-auto h-4 w-4 transition-transform data-[state=closed]:-rotate-90"
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
                                field="primary_mac"
                                glossaryKey="mac"
                            />
                            <Field
                                label="Nombre del equipo"
                                value={d.hostname ?? d.display_name ?? "—"}
                                field="hostname"
                                glossaryKey="hostname"
                            />
                            <Field
                                label="Fabricante"
                                value={d.vendor ?? "—"}
                                glossaryKey="vendor"
                            />
                            <Field
                                label="Tipo"
                                value={deviceLabel(d.device_type)}
                            />
                            <div className="flex flex-col gap-1">
                                <span className="flex items-center gap-1 text-xs text-muted-foreground">
                                    Confianza
                                    <InfoTooltip
                                        term="Confianza"
                                        glossaryKey="confianza"
                                    />
                                </span>
                                <ConfidenceBadge value={d.confidence} />
                            </div>
                            <Field
                                label="Último visto"
                                value={formatRelative(d.last_seen_at)}
                                title={formatTimestamp(d.last_seen_at)}
                            />
                            {/* Formulario de edición (AC-8, AC-9). Edición por
                                d.id (UUID), no por hostname/IP — un dispositivo
                                sin hostname es editable. */}
                            <div className="mt-2 border-t pt-4 sm:col-span-2">
                                <div className="grid gap-3 sm:grid-cols-2">
                                    <FormField
                                        label="Nombre personalizado"
                                        htmlFor="detail-display-name"
                                    >
                                        {/* El input de edición muestra el valor
                                            real; la censura aplica al
                                            exportar/compartir, no a la edición
                                            local del propietario (AC-17). */}
                                        <Input
                                            id="detail-display-name"
                                            value={displayName}
                                            onChange={(e) =>
                                                setDisplayName(e.target.value)
                                            }
                                            placeholder={
                                                d.hostname ?? d.primary_ip ?? ""
                                            }
                                        />
                                    </FormField>
                                    <FormField
                                        label="Confiable"
                                        htmlFor="detail-is-trusted"
                                        helper="Marca el dispositivo como confiable"
                                    >
                                        <Button
                                            id="detail-is-trusted"
                                            type="button"
                                            variant={
                                                isTrusted
                                                    ? "default"
                                                    : "outline"
                                            }
                                            onClick={() =>
                                                setIsTrusted((v) => !v)
                                            }
                                            aria-pressed={isTrusted}
                                            className="w-fit gap-1.5"
                                        >
                                            {isTrusted ? "Sí" : "No"}
                                        </Button>
                                    </FormField>
                                    <FormField
                                        label="Notas"
                                        htmlFor="detail-notes"
                                        helper="Notas internas (no se comparten)"
                                    >
                                        {/* notes NO es sensible (censor.ts:17),
                                            visible directo. */}
                                        <Input
                                            id="detail-notes"
                                            value={notes}
                                            onChange={(e) =>
                                                setNotes(e.target.value)
                                            }
                                            placeholder="Notas sobre este dispositivo"
                                        />
                                    </FormField>
                                    <FormField
                                        label="Tipo"
                                        helper="Auto-detectado"
                                    >
                                        <div className="flex h-9 items-center gap-2 text-sm">
                                            <Icon
                                                className="h-4 w-4 text-muted-foreground"
                                                aria-hidden
                                            />
                                            <span>
                                                {deviceLabel(d.device_type)}
                                            </span>
                                        </div>
                                    </FormField>
                                </div>
                                <div className="mt-3 flex items-center gap-3">
                                    <Button
                                        onClick={handleSaveEdit}
                                        disabled={saving}
                                        className="gap-1.5"
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
                                            "Guardar"
                                        )}
                                    </Button>
                                    <TrustBadge device={trustBadgeDevice} />
                                </div>
                            </div>
                        </CardContent>
                    </CollapsibleContent>
                </Card>
            </Collapsible>

            {/* Sección colapsable: escaneo de puertos (AC-10). */}
            <div ref={scanRef}>
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
                                        className="ml-auto h-4 w-4 transition-transform data-[state=closed]:-rotate-90"
                                        aria-hidden
                                    />
                                </CardTitle>
                            </CardHeader>
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <CardContent className="flex flex-col gap-4">
                                <FormField
                                    label="Perfil"
                                    htmlFor="detail-profile"
                                    helper="Tipo de barrido de puertos"
                                >
                                    <div className="flex flex-wrap items-end gap-3">
                                        <ProfileSelect
                                            value={profile}
                                            onChange={setProfile}
                                            className="w-40"
                                            id="detail-profile"
                                            disabled={scanning}
                                        />
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
                                </FormField>

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
                                                {Math.round(elapsed / 100) / 10}
                                                s /{" "}
                                                {Math.round(remainMs / 100) /
                                                    10}
                                                s
                                            </span>
                                        </div>
                                    </div>
                                )}
                            </CardContent>
                        </CollapsibleContent>
                    </Card>
                </Collapsible>
            </div>

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
                                    className="ml-auto h-4 w-4 transition-transform data-[state=closed]:-rotate-90"
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
                                    description="Aún no hay servicios detectados. Escanea los puertos de este dispositivo."
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
                                            Escanear puertos
                                        </Button>
                                    }
                                />
                            ) : (
                                <div className="overflow-x-auto rounded-md border border-border">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
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
                                                <TableHead>Producto</TableHead>
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
