import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/empty-state";
import { deviceIcon, deviceLabel } from "@/components/device-icons";
import { useToast } from "@/components/ui/toast";
import {
    LayoutGrid,
    Table as TableIcon,
    Search,
    Download,
    Loader2,
    Network as NetworkIcon,
    ArrowRight,
    ShieldAlert,
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

type View = "cards" | "table";

dayjs.extend(relativeTime);

export function Devices() {
    const { toast } = useToast();
    const navigate = useNavigate();
    const [devices, setDevices] = useState<Device[]>([]);
    const [query, setQuery] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [view, setView] = useState<View>("cards");

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

    const filtered = useMemo(() => {
        const q = query.trim().toLowerCase();
        if (!q) return devices;
        return devices.filter((d) =>
            [d.primary_ip, d.primary_mac, d.hostname, d.display_name, d.vendor]
                .filter(Boolean)
                .some((v) => (v as string).toLowerCase().includes(q)),
        );
    }, [devices, query]);

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
        <div className="flex flex-col gap-4" aria-busy={loading}>
            <Card>
                <CardHeader className="flex-row items-center justify-between">
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
                    <div className="relative mb-4 max-w-md">
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

                    {error && (
                        <div role="alert" className="mb-4">
                            <EmptyState
                                icon={ShieldAlert}
                                title="No se pudieron cargar los dispositivos"
                                description={error}
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
                            description="Ejecuta un escaneo desde el Dashboard para descubrir los hosts de tu red."
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
                                            aria-label={`Dispositivo ${d.primary_ip ?? d.id}: ${deviceLabel(d.device_type)}`}
                                        >
                                            <CardContent className="flex flex-col gap-3 p-4">
                                                <div className="flex items-start justify-between gap-2">
                                                    <div className="flex items-center gap-2.5">
                                                        <div className="flex h-10 w-10 items-center justify-center rounded-md bg-muted text-muted-foreground">
                                                            <Icon
                                                                className="h-5 w-5"
                                                                aria-hidden
                                                            />
                                                        </div>
                                                        <div className="flex flex-col">
                                                            <span className="font-medium">
                                                                {d.primary_ip ??
                                                                    d.id}
                                                            </span>
                                                            <span className="text-xs text-muted-foreground">
                                                                {`${
                                                                    d.hostname ??
                                                                    d.display_name ??
                                                                    "—"
                                                                } · ${d.primary_mac}`}
                                                            </span>
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
                                                    {d.confidence &&
                                                        d.confidence !==
                                                            "0" && (
                                                            <Badge variant="outline">
                                                                conf.{" "}
                                                                {d.confidence}
                                                            </Badge>
                                                        )}
                                                </div>
                                                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
                                                    <Meta
                                                        label="Vendor"
                                                        value={d.vendor ?? "—"}
                                                    />
                                                    <Meta
                                                        label="Visto"
                                                        value={dayjs(
                                                            d.last_seen_at,
                                                        ).fromNow()}
                                                    />
                                                    <Meta
                                                        label="Confianza"
                                                        value={d.confidence}
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
                                            <TableHead>MAC</TableHead>
                                            <TableHead>Hostname</TableHead>
                                            <TableHead>Vendor</TableHead>
                                            <TableHead>Tipo</TableHead>
                                            <TableHead>Confianza</TableHead>
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
                                                    {d.primary_ip ?? "—"}
                                                </TableCell>
                                                <TableCell className="font-mono text-xs">
                                                    {d.primary_mac ?? "—"}
                                                </TableCell>
                                                <TableCell>
                                                    {d.hostname ??
                                                        d.display_name ??
                                                        "—"}
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
                                                    {d.confidence}
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    {d.last_seen_at}
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
}: {
    label: string;
    value: string;
    mono?: boolean;
}) {
    return (
        <div className="flex flex-col">
            <dt className="text-muted-foreground">{label}</dt>
            <dd className={`font-medium ${mono ? "font-mono" : ""}`}>
                {value}
            </dd>
        </div>
    );
}
