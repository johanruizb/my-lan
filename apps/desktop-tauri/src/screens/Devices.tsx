import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
    Link,
    useLocation,
    useNavigate,
    useSearchParams,
} from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { CardHeader } from "@/components/ui/card-header";
import { Input } from "@/components/ui/input";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { EmptyState } from "@/components/empty-state";
import { deviceIcon, deviceLabel } from "@/components/device-icons";
import { useToast } from "@/components/ui/toast";
import { RelativeTime } from "@/components/relative-time";
import {
    LayoutGrid,
    Table as TableIcon,
    Search,
    Download,
    Loader2,
    Network as NetworkIcon,
    ShieldAlert,
    AlertCircle,
    Play,
    ChevronRight,
    RefreshCw,
    ChevronDown,
    ArrowLeft,
    FileText,
} from "lucide-react";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { CopyButton } from "@/components/copy-button";
import { exportDevices, listDevices, type Device } from "@/lib/tauri";
import { deviceKey, useScan } from "@/App";
import { MaskedValue } from "@/components/masked-value";
import { useCensorship } from "@/components/censorship-provider";
import { useWorkspace } from "@/components/workspace-provider";
import { ExportReportModal } from "@/components/export-report-modal";
import { maskValue } from "@/lib/censor";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { OnlineBadge } from "@/components/online-badge";
import { TrustBadge } from "@/components/trust-badge";
import { ConfidenceBadge } from "@/components/confidence-badge";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";

type View = "cards" | "table";

// Tarjetas que muestra la vista Red antes de enlazar a la página completa
// (3 filas en la rejilla de 3 columnas).
const PREVIEW_LIMIT = 9;

// Hook que sincroniza un valor string con un query param de la URL.
// Usa replace: true para no llenar el historial en cada cambio.
function useUrlState<T extends string>(
    key: string,
    defaultValue: T,
): [T, (value: T) => void] {
    const [searchParams, setSearchParams] = useSearchParams();
    const value = (searchParams.get(key) as T) ?? defaultValue;
    const setValue = useCallback(
        (next: T) => {
            setSearchParams(
                (prev) => {
                    const nextParams = new URLSearchParams(prev);
                    if (next === defaultValue) nextParams.delete(key);
                    else nextParams.set(key, next);
                    return nextParams;
                },
                { replace: true },
            );
        },
        [key, defaultValue, setSearchParams],
    );
    return [value, setValue];
}

export function Devices({
    variant = "full",
}: {
    /**
     * `full`: página dedicada /devices (vistas, todos los filtros, export).
     * `preview`: incrustada en la vista Red bajo la cabecera de red; muestra
     * los primeros dispositivos con búsqueda + estado y enlaza a la página
     * completa conservando los filtros (ADR-0007).
     */
    variant?: "full" | "preview";
} = {}) {
    const preview = variant === "preview";
    const navigate = useNavigate();
    const location = useLocation();
    const { toast } = useToast();
    const { scanning, progress, devicesFound, startScan } = useScan();
    const { censorshipEnabled } = useCensorship();
    const [devices, setDevices] = useState<Device[]>([]);
    const [query, setQuery] = useUrlState<string>("q", "");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [urlView, setView] = useUrlState<View>("view", "cards");
    const view: View = preview ? "cards" : urlView;
    // Filtros de la lista (AC-14): tipo, estado online y confiable (manual
    // binario via is_trusted, ADR-0006). Por defecto "all" → se ven todos
    // (offline incluidos, AC-15). Sincronizados a URL para deep-linking.
    const [filterType, setFilterType] = useUrlState<string>("type", "all");
    const [filterOnline, setFilterOnline] = useUrlState<
        "all" | "online" | "offline"
    >("online", "all");
    const [filterTrust, setFilterTrust] = useUrlState<
        "all" | "trusted" | "untrusted"
    >("trust", "all");
    const [, setSearchParams] = useSearchParams();
    const { activeWorkspace } = useWorkspace();
    const [exportPdfOpen, setExportPdfOpen] = useState(false);

    const clearAllFilters = useCallback(() => {
        setSearchParams({}, { replace: true });
    }, [setSearchParams]);

    const refresh = useCallback(async () => {
        setLoading(true);
        try {
            const rows = await listDevices(activeWorkspace.id);
            setDevices(rows);
            setError(null);
        } catch (e) {
            setError(String(e));
            setDevices([]);
        } finally {
            setLoading(false);
        }
    }, [activeWorkspace.id]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    // Al terminar el scan refresca con la verdad canónica de la BD (AC-10).
    const prevScanning = useRef(scanning);
    useEffect(() => {
        if (prevScanning.current && !scanning) refresh();
        prevScanning.current = scanning;
    }, [scanning, refresh]);

    // Merge en sitio de los hosts hallados en vivo sobre el listado base, por
    // identidad primary_ip → primary_mac, sin duplicados (AC-5/AC-6).
    const merged = useMemo(() => {
        if (devicesFound.length === 0) return devices;
        const map = new Map<string, Device>();
        for (const d of devices) map.set(deviceKey(d), d);
        for (const d of devicesFound) map.set(deviceKey(d), d);
        return [...map.values()];
    }, [devices, devicesFound]);

    // Filtros componen AND con la búsqueda texto (AC-14). Single predicate
    // para una sola pasada sobre `merged` (fix review #4). En `preview` solo
    // se exponen búsqueda + estado: los filtros ocultos no deben aplicar.
    const typeFilter = preview ? "all" : filterType;
    const trustFilter = preview ? "all" : filterTrust;
    const filtered = useMemo(() => {
        const q = query.trim().toLowerCase();
        return merged.filter((d) => {
            if (q) {
                const matches = [
                    d.primary_ip,
                    d.primary_mac,
                    d.hostname,
                    d.display_name,
                    d.vendor,
                ]
                    .filter(Boolean)
                    .some((v) => (v as string).toLowerCase().includes(q));
                if (!matches) return false;
            }
            if (typeFilter !== "all" && d.device_type !== typeFilter)
                return false;
            // AC-15: "En línea" es opt-in; sin filtro activo se ven todos.
            if (filterOnline === "online" && !d.is_online) return false;
            if (filterOnline === "offline" && d.is_online) return false;
            // Confiable: etiqueta manual binaria (is_trusted, ADR-0006).
            if (trustFilter === "trusted" && !d.is_trusted) return false;
            if (trustFilter === "untrusted" && d.is_trusted) return false;
            return true;
        });
    }, [merged, query, typeFilter, filterOnline, trustFilter]);

    // Conteos sobre el inventario completo: el resumen que antes vivía en
    // tarjetas sueltas del Dashboard ahora acompaña a cada filtro.
    const counts = useMemo(() => {
        let online = 0;
        let trusted = 0;
        for (const d of merged) {
            if (d.is_online) online++;
            if (d.is_trusted) trusted++;
        }
        return {
            all: merged.length,
            online,
            offline: merged.length - online,
            trusted,
            untrusted: merged.length - trusted,
        };
    }, [merged]);

    // En `preview`: online primero y luego lo visto más reciente (los hallazgos
    // en vivo del descubrimiento suben arriba), recortado a PREVIEW_LIMIT.
    const visible = useMemo(() => {
        if (!preview) return filtered;
        return [...filtered]
            .sort(
                (a, b) =>
                    Number(b.is_online) - Number(a.is_online) ||
                    b.last_seen_at.localeCompare(a.last_seen_at),
            )
            .slice(0, PREVIEW_LIMIT);
    }, [preview, filtered]);

    // Enlace a la página dedicada conservando búsqueda y estado.
    const fullListPath = `/devices${location.search}`;

    async function handleExport(format: string) {
        try {
            const path = await exportDevices(format);
            toast(`Dispositivos exportados a: ${path}`, "success");
        } catch (e) {
            toast(`Error exportando: ${e}`, "error");
        }
    }

    return (
        <section
            aria-label="Dispositivos"
            className={cn("flex flex-col", SECTION_GAP)}
            aria-busy={loading}
        >
            {!preview && (
                <Link
                    to="/"
                    className="inline-flex w-fit items-center gap-1.5 rounded-md text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                    <ArrowLeft className="h-4 w-4" aria-hidden />
                    Volver a la red
                </Link>
            )}
            <Card className="border border-border/40 shadow-sm overflow-hidden">
                <CardHeader variant="toolbar">
                    <CardTitle className="flex items-center gap-2">
                        <NetworkIcon
                            className="h-5 w-5 text-primary"
                            aria-hidden
                        />
                        Dispositivos ({filtered.length})
                    </CardTitle>
                    {preview ? (
                        <Button
                            asChild
                            variant="outline"
                            size="sm"
                            className="gap-1.5"
                        >
                            <Link to={fullListPath}>
                                Ver todos
                                <ChevronRight
                                    className="h-3.5 w-3.5"
                                    aria-hidden
                                />
                            </Link>
                        </Button>
                    ) : (
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
                            {/* ADR-0007: el descubrimiento se lanza desde la cabecera
                            de la vista Red; aquí solo Refrescar la lista. Export
                            agrupado en un dropdown (#27) con formato CSV/JSON. */}
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={refresh}
                                className="gap-1.5"
                            >
                                <RefreshCw
                                    className="h-3.5 w-3.5"
                                    aria-hidden
                                />
                                Refrescar
                            </Button>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        className="gap-1.5"
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
                                        onSelect={() => handleExport("csv")}
                                    >
                                        CSV
                                    </DropdownMenuItem>
                                    <DropdownMenuItem
                                        onSelect={() => handleExport("json")}
                                    >
                                        JSON
                                    </DropdownMenuItem>
                                    <DropdownMenuItem
                                        onSelect={() => setExportPdfOpen(true)}
                                    >
                                        Informe Ejecutivo (PDF)
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                        </div>
                    )}
                </CardHeader>
                <CardContent>
                    {/* #26: Filtros siempre visibles (sin Collapsible). Búsqueda
                        por texto + filtros por tipo, estado y confiable. En
                        `preview` solo búsqueda + estado. */}
                    <div className="mb-4">
                        <div className="relative max-w-md">
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
                                name="device-search"
                                autoComplete="off"
                            />
                        </div>
                        {/* Filtros por tipo, estado y confiable (AC-14). */}
                        <div className="mt-3 flex flex-wrap items-end gap-3">
                            {!preview && (
                                <label className="flex flex-col gap-1 text-xs text-muted-foreground">
                                    Tipo
                                    <Select
                                        value={filterType}
                                        onValueChange={setFilterType}
                                    >
                                        <SelectTrigger
                                            className="w-40 mt-1"
                                            aria-label="Filtrar por tipo"
                                        >
                                            <SelectValue placeholder="Todos" />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="all">
                                                Todos
                                            </SelectItem>
                                            {DEVICE_TYPE_OPTIONS.map((t) => (
                                                <SelectItem key={t} value={t}>
                                                    {deviceLabel(t)}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </label>
                            )}
                            <FilterToggle
                                label="Estado"
                                ariaLabel="Filtrar por estado"
                                options={[
                                    {
                                        value: "all",
                                        label: "Todos",
                                        count: counts.all,
                                    },
                                    {
                                        value: "online",
                                        label: "Online",
                                        count: counts.online,
                                    },
                                    {
                                        value: "offline",
                                        label: "Offline",
                                        count: counts.offline,
                                    },
                                ]}
                                value={filterOnline}
                                onChange={setFilterOnline}
                            />
                            {!preview && (
                                <FilterToggle
                                    label="Confiable"
                                    ariaLabel="Filtrar por confiable"
                                    options={[
                                        {
                                            value: "all",
                                            label: "Todos",
                                            count: counts.all,
                                        },
                                        {
                                            value: "trusted",
                                            label: "Confiables",
                                            count: counts.trusted,
                                        },
                                        {
                                            value: "untrusted",
                                            label: "No confiables",
                                            count: counts.untrusted,
                                        },
                                    ]}
                                    value={filterTrust}
                                    onChange={setFilterTrust}
                                />
                            )}
                        </div>
                    </div>

                    {/* Barra de progreso del barrido (AC-3/AC-4): % cuando se
                        conoce el total, indeterminada si no. En `preview` la
                        muestra la cabecera de red. */}
                    {!preview && scanning && (
                        <div
                            className="mb-4 flex flex-col gap-1.5"
                            aria-live="polite"
                        >
                            <div className="flex items-center justify-between text-xs text-muted-foreground">
                                <span>Descubriendo red…</span>
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
                        <Alert
                            variant="destructive"
                            className="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3"
                        >
                            <div className="flex items-start gap-2">
                                <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
                                <div>
                                    <AlertTitle>
                                        No se pudieron cargar los dispositivos
                                    </AlertTitle>
                                    <AlertDescription className="text-xs">
                                        {error}
                                    </AlertDescription>
                                </div>
                            </div>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={refresh}
                                className="gap-1.5 shrink-0 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
                            >
                                <RefreshCw
                                    className="h-4 w-4"
                                    aria-hidden
                                />
                                Reintentar
                            </Button>
                        </Alert>
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

                    {!error && !loading && merged.length === 0 && scanning && (
                        <div className="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
                            <Loader2
                                className="h-4 w-4 animate-spin"
                                aria-hidden
                            />
                            Buscando dispositivos en tu red…
                        </div>
                    )}

                    {!error && !loading && merged.length === 0 && !scanning && (
                        <EmptyState
                            icon={NetworkIcon}
                            title="Sin dispositivos"
                            description={
                                preview
                                    ? "Aún no hay dispositivos en tu red. Pulsa Descubrir dispositivos para encontrarlos."
                                    : "Aún no hay dispositivos en tu red. Inicia un descubrimiento para encontrarlos."
                            }
                            action={
                                preview ? undefined : (
                                    <Button
                                        size="sm"
                                        onClick={() => startScan()}
                                        className="gap-1.5"
                                    >
                                        <Play
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Descubrir dispositivos
                                    </Button>
                                )
                            }
                        />
                    )}

                    {!error &&
                        !loading &&
                        merged.length > 0 &&
                        filtered.length === 0 && (
                            <EmptyState
                                icon={Search}
                                title="Sin resultados"
                                description="No se encontraron dispositivos que coincidan con la búsqueda o los filtros aplicados."
                                action={
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={clearAllFilters}
                                        className="gap-1.5"
                                    >
                                        <RefreshCw
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Limpiar filtros
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
                                className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
                                role="list"
                                aria-label="Lista de dispositivos"
                            >
                                {visible.map((d) => {
                                    const Icon = deviceIcon(d.device_type);
                                    const devicePath = `/devices/${encodeURIComponent(d.primary_ip ?? d.id)}`;
                                    return (
                                        <li
                                            key={d.id}
                                            role="listitem"
                                            className="list-none cv-auto h-full"
                                        >
                                            <Link
                                                to={devicePath}
                                                aria-label={`Dispositivo ${censorshipEnabled ? maskValue("primary_ip", d.primary_ip ?? d.id) : (d.primary_ip ?? d.id)}: ${deviceLabel(d.device_type)}`}
                                                className="block h-full rounded-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                            >
                                                <Card className="group relative flex h-full flex-col border border-border/40 overflow-hidden shadow-sm transition-[border-color,box-shadow] hover:border-primary/40 hover:shadow-md">
                                                    <CardContent className="flex flex-1 flex-col justify-between gap-3 p-4">
                                                        <div className="flex flex-col gap-2.5">
                                                            <div className="flex items-center justify-between gap-3">
                                                                <div className="flex min-w-0 items-center gap-3">
                                                                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
                                                                        <Icon
                                                                            className="h-5 w-5 text-primary"
                                                                            aria-hidden
                                                                        />
                                                                    </div>
                                                                    <div className="flex min-w-0 flex-col">
                                                                        <span className="truncate font-bold text-foreground text-sm">
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
                                                                        </span>
                                                                        <span className="truncate text-xs text-muted-foreground">
                                                                            {deviceLabel(
                                                                                d.device_type,
                                                                            )}
                                                                            {d.vendor
                                                                                ? ` · ${d.vendor}`
                                                                                : ""}
                                                                        </span>
                                                                    </div>
                                                                </div>
                                                                <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-border/50 bg-background/50 text-muted-foreground transition-[border-color,background-color,color,transform] duration-200 group-hover:border-primary/30 group-hover:bg-primary/5 group-hover:text-primary group-hover:translate-x-0.5">
                                                                    <ChevronRight
                                                                        className="h-4 w-4"
                                                                        aria-hidden
                                                                    />
                                                                </div>
                                                            </div>

                                                            <div className="flex min-w-0 items-center gap-2 font-mono text-[11px] text-muted-foreground min-h-[1.25rem]">
                                                                {(d.display_name ||
                                                                    d.hostname) && (
                                                                    <span className="truncate">
                                                                        <MaskedValue
                                                                            field="primary_ip"
                                                                            value={
                                                                                d.primary_ip ??
                                                                                d.id
                                                                            }
                                                                        />
                                                                    </span>
                                                                )}
                                                                {(d.display_name ||
                                                                    d.hostname) &&
                                                                    d.primary_mac && (
                                                                        <span
                                                                            aria-hidden
                                                                        >
                                                                            ·
                                                                        </span>
                                                                    )}
                                                                {d.primary_mac ? (
                                                                    <span className="truncate uppercase">
                                                                        <MaskedValue
                                                                            field="primary_mac"
                                                                            value={
                                                                                d.primary_mac
                                                                            }
                                                                            mono
                                                                        />
                                                                    </span>
                                                                ) : !(d.display_name || d.hostname) ? (
                                                                    <span className="truncate text-muted-foreground/60 text-[11px] font-sans">
                                                                        Sin MAC detectada
                                                                    </span>
                                                                ) : null}
                                                            </div>
                                                        </div>

                                                        {/* Estado, confianza y última vez visto unificados con continuidad visual */}
                                                        <div className="mt-auto pt-3 border-t border-border/30 flex flex-wrap items-center justify-between gap-y-1.5 gap-x-2 text-xs">
                                                            <div className="flex items-center gap-x-3">
                                                                <OnlineBadge
                                                                    isOnline={
                                                                        d.is_online
                                                                    }
                                                                />
                                                                <TrustBadge
                                                                    device={d}
                                                                />
                                                            </div>
                                                            <span className="whitespace-nowrap text-muted-foreground text-[11px]">
                                                                Visto{" "}
                                                                <RelativeTime
                                                                    value={
                                                                        d.last_seen_at
                                                                    }
                                                                />
                                                            </span>
                                                        </div>
                                                    </CardContent>
                                                </Card>
                                            </Link>
                                        </li>
                                    );
                                })}
                            </div>
                        )}

                    {/* Pie del preview: enlace a la página dedicada cuando la
                        lista está recortada. */}
                    {preview &&
                        !error &&
                        !loading &&
                        filtered.length > visible.length && (
                            <div className="mt-4 flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3 text-xs text-muted-foreground">
                                <span className="tabular-nums">
                                    Mostrando {visible.length} de{" "}
                                    {filtered.length}
                                </span>
                                <Link
                                    to={fullListPath}
                                    className="inline-flex items-center gap-1 rounded-sm font-medium text-primary hover:underline underline-offset-4 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                >
                                    Ver los {filtered.length} dispositivos
                                    <ChevronRight
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                </Link>
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
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Dispositivo
                                                    <InfoTooltip
                                                        term="Dispositivo"
                                                        glossaryKey="hostname"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>IP</TableHead>
                                            <TableHead>Estado</TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Certeza
                                                    <InfoTooltip
                                                        term="Certeza"
                                                        glossaryKey="certeza"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Confiable
                                                    <InfoTooltip
                                                        term="Confiable"
                                                        glossaryKey="confiable"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>
                                                <span className="inline-flex items-center gap-1">
                                                    Vendor
                                                    <InfoTooltip
                                                        term="Vendor"
                                                        glossaryKey="vendor"
                                                    />
                                                </span>
                                            </TableHead>
                                            <TableHead>Visto</TableHead>
                                            <TableHead className="w-[50px]"></TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {filtered.map((d) => {
                                            const Icon = deviceIcon(
                                                d.device_type,
                                            );
                                            return (
                                                <TableRow
                                                    key={d.id}
                                                    onClick={(e) => {
                                                        // Fila completa navega (como antes con el
                                                        // stretched link), salvo clicks en el enlace
                                                        // real o en valores enmascarados (tabindex).
                                                        if (
                                                            (
                                                                e.target as HTMLElement
                                                            ).closest(
                                                                "a, [tabindex], button",
                                                            )
                                                        )
                                                            return;
                                                        navigate(
                                                            `/devices/${encodeURIComponent(d.primary_ip ?? d.id)}`,
                                                        );
                                                    }}
                                                    className="group cursor-pointer cv-auto-row"
                                                >
                                                    <TableCell>
                                                        <div className="flex items-center gap-3">
                                                            <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground shrink-0">
                                                                <Icon
                                                                    className="h-4 w-4"
                                                                    aria-hidden
                                                                />
                                                            </div>
                                                            <div className="flex flex-col min-w-0">
                                                                <span className="font-semibold text-foreground truncate">
                                                                    {d.display_name ||
                                                                    d.hostname ? (
                                                                        <MaskedValue
                                                                            field={
                                                                                d.display_name
                                                                                    ? "display_name"
                                                                                    : "hostname"
                                                                            }
                                                                            value={
                                                                                d.display_name ??
                                                                                d.hostname ??
                                                                                ""
                                                                            }
                                                                        />
                                                                    ) : (
                                                                        <span>
                                                                            {deviceLabel(
                                                                                d.device_type,
                                                                            )}
                                                                        </span>
                                                                    )}
                                                                </span>
                                                                {d.primary_mac && (
                                                                    <div className="flex items-center gap-1 font-mono text-[10px] text-muted-foreground uppercase">
                                                                        <MaskedValue
                                                                            field="primary_mac"
                                                                            value={
                                                                                d.primary_mac
                                                                            }
                                                                            mono
                                                                        />
                                                                        <CopyButton
                                                                            value={
                                                                                d.primary_mac
                                                                            }
                                                                            label="MAC"
                                                                            className="opacity-0 group-hover:opacity-100 focus-visible:opacity-100 focus:opacity-100 h-4 w-4"
                                                                        />
                                                                    </div>
                                                                )}
                                                            </div>
                                                        </div>
                                                    </TableCell>
                                                    <TableCell className="font-mono text-xs">
                                                        <div className="flex items-center gap-1.5 font-mono text-xs">
                                                            <MaskedValue
                                                                field="primary_ip"
                                                                value={
                                                                    d.primary_ip ??
                                                                    "—"
                                                                }
                                                                mono
                                                            />
                                                            {d.primary_ip && (
                                                                <CopyButton
                                                                    value={
                                                                        d.primary_ip
                                                                    }
                                                                    label="IP"
                                                                    className="opacity-0 group-hover:opacity-100 focus-visible:opacity-100 focus:opacity-100 h-4 w-4"
                                                                />
                                                            )}
                                                        </div>
                                                    </TableCell>
                                                    <TableCell>
                                                        <OnlineBadge
                                                            isOnline={
                                                                d.is_online
                                                            }
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <ConfidenceBadge
                                                            value={d.confidence}
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <TrustBadge
                                                            device={d}
                                                        />
                                                    </TableCell>
                                                    <TableCell className="text-xs">
                                                        <div
                                                            className="flex items-center gap-1 max-w-[200px]"
                                                            title={
                                                                d.vendor ??
                                                                undefined
                                                            }
                                                        >
                                                            <span className="truncate text-foreground/90 font-medium">
                                                                {d.vendor ??
                                                                    "—"}
                                                            </span>
                                                            {d.vendor && (
                                                                <CopyButton
                                                                    value={
                                                                        d.vendor
                                                                    }
                                                                    label="Vendor"
                                                                    className="opacity-0 group-hover:opacity-100 focus-visible:opacity-100 focus:opacity-100 h-4 w-4 shrink-0"
                                                                />
                                                            )}
                                                        </div>
                                                    </TableCell>
                                                    <TableCell className="text-xs text-muted-foreground">
                                                        <RelativeTime
                                                            value={
                                                                d.last_seen_at
                                                            }
                                                        />
                                                    </TableCell>
                                                    <TableCell className="text-right">
                                                        <div className="flex justify-end">
                                                            <Link
                                                                to={`/devices/${encodeURIComponent(d.primary_ip ?? d.id)}`}
                                                                className="flex h-7 w-7 items-center justify-center rounded-full border border-transparent text-muted-foreground transition-[border-color,background-color,color,transform] duration-200 group-hover:border-primary/20 group-hover:bg-primary/5 group-hover:text-primary group-hover:translate-x-0.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                                                aria-label={`Abrir dispositivo ${d.primary_ip ?? d.id}: ${deviceLabel(d.device_type)}`}
                                                            >
                                                                <ChevronRight
                                                                    className="h-3.5 w-3.5"
                                                                    aria-hidden
                                                                />
                                                            </Link>
                                                        </div>
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
            {exportPdfOpen && (
                <ExportReportModal
                    open={exportPdfOpen}
                    onOpenChange={setExportPdfOpen}
                />
            )}
        </section>
    );
}

// Opciones del filtro Tipo (AC-14): las 12 variantes de `device-icons`.
const DEVICE_TYPE_OPTIONS = [
    "router",
    "phone",
    "tablet",
    "laptop",
    "desktop",
    "tv",
    "printer",
    "camera",
    "nas",
    "console",
    "iot",
    "unknown",
] as const;

// Grupo de botones toggle para filtros (AC-14). Sigue el patrón del toggle de
// vista tarjetas/tabla. Genérico sobre el tipo de valor para reutilizar en
// Estado y Confianza.
function FilterToggle<T extends string>({
    label,
    ariaLabel,
    options,
    value,
    onChange,
}: {
    label: string;
    ariaLabel: string;
    options: { value: T; label: string; ariaLabel?: string; count?: number }[];
    value: T;
    onChange: (v: T) => void;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="text-xs text-muted-foreground">{label}</span>
            <div
                className="flex rounded-md border border-border p-0.5"
                role="group"
                aria-label={ariaLabel}
            >
                {options.map((o) => (
                    <Button
                        key={o.value}
                        variant={value === o.value ? "secondary" : "ghost"}
                        size="sm"
                        onClick={() => onChange(o.value)}
                        aria-pressed={value === o.value}
                        aria-label={o.ariaLabel}
                        className="gap-1.5"
                    >
                        {o.label}
                        {o.count !== undefined && (
                            <span
                                aria-hidden
                                className={cn(
                                    "tabular-nums",
                                    value === o.value
                                        ? "text-foreground/70"
                                        : "text-muted-foreground",
                                )}
                            >
                                {o.count}
                            </span>
                        )}
                    </Button>
                ))}
            </div>
        </div>
    );
}
