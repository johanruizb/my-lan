import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { CardHeader } from "@/components/ui/card-header";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/empty-state";
import { deviceIcon, deviceLabel } from "@/components/device-icons";
import { useToast } from "@/components/ui/toast";
import { formatRelative, formatTimestamp } from "@/lib/format";
import { RelativeTime } from "@/components/relative-time";
import {
    LayoutGrid,
    Table as TableIcon,
    Search,
    Download,
    Loader2,
    Network as NetworkIcon,
    ArrowRight,
    ShieldAlert,
    Play,
    X,
    ChevronDown,
    RefreshCw,
} from "lucide-react";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { exportDevices, listDevices, type Device } from "@/lib/tauri";
import { deviceKey, useScan } from "@/App";
import { MaskedValue } from "@/components/masked-value";
import { useCensorship } from "@/components/censorship-provider";
import { maskValue } from "@/lib/censor";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ConfidenceBadge } from "@/components/confidence-badge";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";

type View = "cards" | "table";

export function Devices() {
    const { toast } = useToast();
    const navigate = useNavigate();
    const { scanning, progress, devicesFound, startScan, cancel } = useScan();
    const { censorshipEnabled } = useCensorship();
    const [devices, setDevices] = useState<Device[]>([]);
    const [query, setQuery] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [view, setView] = useState<View>("cards");
    const [openFilters, setOpenFilters] = useState(true);

    async function refresh() {
        setLoading(true);
        try {
            const rows = await listDevices();
            setDevices(rows);
            setError(null);
        } catch (e) {
            setError(String(e));
            setDevices([]);
        } finally {
            setLoading(false);
        }
    }

    useEffect(() => {
        refresh();
    }, []);

    // Al terminar el scan refresca con la verdad canónica de la BD (AC-10).
    const prevScanning = useRef(scanning);
    useEffect(() => {
        if (prevScanning.current && !scanning) refresh();
        prevScanning.current = scanning;
    }, [scanning]);

    // Merge en sitio de los hosts hallados en vivo sobre el listado base, por
    // identidad primary_ip → primary_mac, sin duplicados (AC-5/AC-6).
    const merged = useMemo(() => {
        if (devicesFound.length === 0) return devices;
        const map = new Map<string, Device>();
        for (const d of devices) map.set(deviceKey(d), d);
        for (const d of devicesFound) map.set(deviceKey(d), d);
        return [...map.values()];
    }, [devices, devicesFound]);

    const filtered = useMemo(() => {
        const q = query.trim().toLowerCase();
        if (!q) return merged;
        return merged.filter((d) =>
            [d.primary_ip, d.primary_mac, d.hostname, d.display_name, d.vendor]
                .filter(Boolean)
                .some((v) => (v as string).toLowerCase().includes(q)),
        );
    }, [merged, query]);

    async function handleExport(format: string) {
        try {
            const path = await exportDevices(format);
            toast(`Dispositivos exportados a: ${path}`, "success");
        } catch (e) {
            toast(`Error exportando: ${e}`, "error");
        }
    }

    const go = (d: Device) =>
        navigate(`/devices/${encodeURIComponent(d.primary_ip ?? d.id)}`);

    return (
        <div className={cn("flex flex-col", SECTION_GAP)} aria-busy={loading}>
            <Card>
                <CardHeader variant="toolbar">
                    <CardTitle className="flex items-center gap-2">
                        <NetworkIcon
                            className="h-5 w-5 text-primary"
                            aria-hidden
                        />
                        Dispositivos ({filtered.length})
                    </CardTitle>
                    <div className="flex flex-wrap items-center gap-2">
                        {/* Toggle de vista tarjetas/tabla (AC-9). */}
                        <div
                            className="flex rounded-md border border-border p-0.5"
                            role="group"
                            aria-label="Modo de vista"
                        >
                            <Button
                                variant={
                                    view === "cards" ? "secondary" : "ghost"
                                }
                                size="sm"
                                onClick={() => setView("cards")}
                                aria-pressed={view === "cards"}
                                aria-label="Vista de tarjetas"
                                className="gap-1.5"
                            >
                                <LayoutGrid
                                    className="h-3.5 w-3.5"
                                    aria-hidden
                                />
                                Tarjetas
                            </Button>
                            <Button
                                variant={
                                    view === "table" ? "secondary" : "ghost"
                                }
                                size="sm"
                                onClick={() => setView("table")}
                                aria-pressed={view === "table"}
                                aria-label="Vista de tabla"
                                className="gap-1.5"
                            >
                                <TableIcon
                                    className="h-3.5 w-3.5"
                                    aria-hidden
                                />
                                Tabla
                            </Button>
                        </div>
                        {/* Botón único "Escanear" (perfil por defecto de Ajustes,
                            sin selector) y "Cancelar" mientras corre (AC-1/AC-2/AC-8). */}
                        {scanning ? (
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={cancel}
                                className="gap-1.5"
                            >
                                <X className="h-3.5 w-3.5" aria-hidden />
                                Cancelar
                            </Button>
                        ) : (
                            <Button
                                size="sm"
                                onClick={() => startScan()}
                                className="gap-1.5"
                            >
                                <Play className="h-3.5 w-3.5" aria-hidden />
                                Escanear
                            </Button>
                        )}
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleExport("csv")}
                            className="gap-1.5"
                        >
                            <Download className="h-3.5 w-3.5" aria-hidden />
                            CSV
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleExport("json")}
                            className="gap-1.5"
                        >
                            <Download className="h-3.5 w-3.5" aria-hidden />
                            JSON
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    <Collapsible
                        open={openFilters}
                        onOpenChange={setOpenFilters}
                        className="mb-4"
                    >
                        <CollapsibleTrigger className="flex w-fit items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground">
                            <Search className="h-4 w-4" aria-hidden />
                            Buscar y filtrar
                            <ChevronDown
                                className="h-4 w-4 transition-transform data-[state=closed]:-rotate-90"
                                aria-hidden
                            />
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <div className="relative mt-3 max-w-md">
                                <Search
                                    className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
                                    aria-hidden
                                />
                                <Input
                                    placeholder="Buscar por IP, MAC, hostname, vendor…"
                                    value={query}
                                    onChange={(e) => setQuery(e.target.value)}
                                    aria-label="Buscar dispositivos"
                                    className="pl-9"
                                />
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    {/* Barra de progreso del barrido (AC-3/AC-4): % cuando se
                        conoce el total, indeterminada si no. */}
                    {scanning && (
                        <div
                            className="mb-4 flex flex-col gap-1.5"
                            aria-live="polite"
                        >
                            <div className="flex items-center justify-between text-xs text-muted-foreground">
                                <span>Escaneando red…</span>
                                <span>
                                    {progress && progress.total > 0
                                        ? `${progress.swept}/${progress.total} (${progress.percent}%)`
                                        : "Explorando…"}
                                </span>
                            </div>
                            <Progress
                                value={progress?.percent ?? undefined}
                                indeterminate={
                                    !progress || progress.total === 0
                                }
                            />
                        </div>
                    )}

                    {error && (
                        <div role="alert" className="mb-4">
                            <EmptyState
                                icon={ShieldAlert}
                                title="No se pudieron cargar los dispositivos"
                                description={error}
                                action={
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={refresh}
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

                    {!error && loading && (
                        <div className="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
                            <Loader2
                                className="h-4 w-4 animate-spin"
                                aria-hidden
                            />
                            Cargando dispositivos…
                        </div>
                    )}

                    {!error && !loading && filtered.length === 0 && (
                        <EmptyState
                            icon={NetworkIcon}
                            title="Sin dispositivos"
                            description="Aún no hay dispositivos en tu red. Escanea para descubrirlos."
                            action={
                                <Button
                                    size="sm"
                                    onClick={() => startScan()}
                                    className="gap-1.5"
                                >
                                    <Play className="h-3.5 w-3.5" aria-hidden />
                                    Descubrir dispositivos
                                </Button>
                            }
                        />
                    )}

                    {/* Vista de tarjetas (AC-9). */}
                    {!error &&
                        !loading &&
                        filtered.length > 0 &&
                        view === "cards" && (
                            <div
                                className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3"
                                role="list"
                                aria-label="Lista de dispositivos"
                            >
                                {filtered.map((d) => {
                                    const Icon = deviceIcon(d.device_type);
                                    return (
                                        <Card
                                            key={d.id}
                                            role="listitem"
                                            className="cursor-pointer transition-colors hover:border-primary/50 focus-within:border-primary focus-within:ring-2 focus-within:ring-ring"
                                            onClick={() => go(d)}
                                            onKeyDown={(e) => {
                                                if (
                                                    e.key === "Enter" ||
                                                    e.key === " "
                                                ) {
                                                    e.preventDefault();
                                                    go(d);
                                                }
                                            }}
                                            tabIndex={0}
                                            aria-label={`Dispositivo ${censorshipEnabled ? maskValue("primary_ip", d.primary_ip ?? d.id) : (d.primary_ip ?? d.id)}: ${deviceLabel(d.device_type)}`}
                                        >
                                            <CardContent className="flex flex-col gap-3 p-4">
                                                <div className="flex items-start justify-between gap-2">
                                                    <div className="flex min-w-0 items-center gap-2.5">
                                                        <div className="flex h-10 w-10 items-center justify-center rounded-md bg-muted text-muted-foreground">
                                                            <Icon
                                                                className="h-5 w-5"
                                                                aria-hidden
                                                            />
                                                        </div>
                                                        <div className="flex min-w-0 flex-col">
                                                            <span className="truncate font-medium">
                                                                <MaskedValue
                                                                    field="primary_ip"
                                                                    value={
                                                                        d.primary_ip ??
                                                                        d.id
                                                                    }
                                                                />
                                                            </span>
                                                            <DeviceIdentity
                                                                hostname={
                                                                    d.hostname ??
                                                                    d.display_name
                                                                }
                                                                primaryMac={
                                                                    d.primary_mac
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                    <ArrowRight
                                                        className="h-4 w-4 text-muted-foreground"
                                                        aria-hidden
                                                    />
                                                </div>
                                                <div className="flex flex-wrap items-center gap-1.5">
                                                    <Badge
                                                        variant="secondary"
                                                        className="gap-1"
                                                    >
                                                        <Icon
                                                            className="h-3 w-3"
                                                            aria-hidden
                                                        />
                                                        {deviceLabel(
                                                            d.device_type,
                                                        )}
                                                    </Badge>
                                                    {Number(d.confidence) >
                                                        0 && (
                                                        <ConfidenceBadge
                                                            value={d.confidence}
                                                        />
                                                    )}
                                                </div>
                                                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
                                                    <Meta
                                                        label="Fabricante"
                                                        value={d.vendor ?? "—"}
                                                        glossaryKey="vendor"
                                                    />
                                                    <Meta
                                                        label="Visto"
                                                        value={formatRelative(
                                                            d.last_seen_at,
                                                        )}
                                                        title={formatTimestamp(
                                                            d.last_seen_at,
                                                        )}
                                                    />
                                                    <Meta
                                                        label="Confianza"
                                                        value={d.confidence}
                                                        glossaryKey="confianza"
                                                    />
                                                </dl>
                                            </CardContent>
                                        </Card>
                                    );
                                })}
                            </div>
                        )}

                    {/* Vista de tabla alternativa (AC-9). */}
                    {!error &&
                        !loading &&
                        filtered.length > 0 &&
                        view === "table" && (
                            <div className="overflow-x-auto rounded-md border border-border">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>IP</TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    MAC
                                                    <InfoTooltip
                                                        term="MAC"
                                                        glossaryKey="mac"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Nombre del equipo
                                                    <InfoTooltip
                                                        term="Nombre del equipo"
                                                        glossaryKey="hostname"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Fabricante
                                                    <InfoTooltip
                                                        term="Fabricante"
                                                        glossaryKey="vendor"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>Tipo</TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Confianza
                                                    <InfoTooltip
                                                        term="Confianza"
                                                        glossaryKey="confianza"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>Último visto</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {filtered.map((d) => (
                                            <TableRow
                                                key={d.id}
                                                className="cursor-pointer"
                                                onClick={() => go(d)}
                                                onKeyDown={(e) => {
                                                    if (e.key === "Enter")
                                                        go(d);
                                                }}
                                                tabIndex={0}
                                            >
                                                <TableCell>
                                                    <MaskedValue
                                                        field="primary_ip"
                                                        value={
                                                            d.primary_ip ?? "—"
                                                        }
                                                    />
                                                </TableCell>
                                                <TableCell className="font-mono text-xs">
                                                    <MaskedValue
                                                        field="primary_mac"
                                                        value={
                                                            d.primary_mac ?? "—"
                                                        }
                                                        mono
                                                    />
                                                </TableCell>
                                                <TableCell>
                                                    <MaskedValue
                                                        field="hostname"
                                                        value={
                                                            d.hostname ??
                                                            d.display_name ??
                                                            "—"
                                                        }
                                                    />
                                                </TableCell>
                                                <TableCell>
                                                    {d.vendor ?? "—"}
                                                </TableCell>
                                                <TableCell>
                                                    <Badge variant="secondary">
                                                        {deviceLabel(
                                                            d.device_type,
                                                        )}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell>
                                                    <ConfidenceBadge
                                                        value={d.confidence}
                                                    />
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    <RelativeTime
                                                        value={d.last_seen_at}
                                                    />
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        )}
                </CardContent>
            </Card>
        </div>
    );
}

function Meta({
    label,
    value,
    mono,
    glossaryKey,
    title,
}: {
    label: string;
    value: string;
    mono?: boolean;
    glossaryKey?: string;
    title?: string;
}) {
    return (
        <div className="flex flex-col">
            <dt className="flex items-center gap-1 text-muted-foreground">
                {label}
                {glossaryKey && (
                    <InfoTooltip term={label} glossaryKey={glossaryKey} />
                )}
            </dt>
            <dd
                className={`font-medium ${mono ? "font-mono" : ""}`}
                title={title}
            >
                {value}
            </dd>
        </div>
    );
}

function DeviceIdentity({
    hostname,
    primaryMac,
}: {
    hostname: string | null;
    primaryMac: string | null;
}) {
    const { censorshipEnabled } = useCensorship();
    const displayHostname = hostname?.trim();
    const displayMac = primaryMac?.trim();
    const hasHostname = Boolean(displayHostname);

    // Cuando censura está ON, el árbol de accesibilidad (title/aria-label) no
    // debe filtrar el valor real — se enmascara con maskValue.
    const hostnameTitle = censorshipEnabled
        ? displayHostname
            ? maskValue("hostname", displayHostname)
            : "Sin hostname"
        : displayHostname || "Sin hostname";
    const macTitle = censorshipEnabled
        ? displayMac
            ? maskValue("primary_mac", displayMac)
            : "Sin MAC"
        : displayMac || "Sin MAC";

    return (
        <div className="mt-0.5 flex min-w-0 flex-col gap-0.5 text-xs">
            <span
                className={`truncate ${
                    hasHostname
                        ? "text-muted-foreground"
                        : "text-muted-foreground/70"
                }`}
                title={hostnameTitle}
            >
                {displayHostname ? (
                    <MaskedValue field="hostname" value={displayHostname} />
                ) : (
                    "Sin hostname"
                )}
            </span>
            <span
                className="truncate font-mono text-[11px] uppercase tracking-normal text-muted-foreground/80"
                title={macTitle}
                aria-label={`MAC ${macTitle}`}
            >
                {displayMac ? (
                    <MaskedValue field="primary_mac" value={displayMac} mono />
                ) : (
                    "Sin MAC"
                )}
            </span>
        </div>
    );
}
