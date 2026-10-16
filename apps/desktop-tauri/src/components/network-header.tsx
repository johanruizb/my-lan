import { useEffect, useRef, useState } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ProfileSelect } from "@/components/profile-select";
import { MaskedValue } from "@/components/masked-value";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { Radio, RefreshCw, Square } from "lucide-react";
import { detectInterface, type LanInterfaceDto } from "@/lib/tauri";
import { useLastScan, useScan } from "@/App";
import { useNetworkName } from "@/lib/use-network-name";
import { isSensitive } from "@/lib/censor";
import { cn } from "@/lib/utils";

// Cabecera de la vista Red (ADR-0007): identidad de la red activa, lecturas
// técnicas siempre visibles (sin acordeón) y el ÚNICO punto de lanzamiento del
// descubrimiento. La franja inferior es la telemetría del barrido: progreso en
// vivo mientras escanea y resumen del último descubrimiento al terminar. Los
// hallazgos aparecen en la lista de dispositivos justo debajo.
export function NetworkHeader({
    profile,
    onProfileChange,
}: {
    profile: string;
    onProfileChange: (p: string) => void;
}) {
    const { lastScan } = useLastScan();
    const { name: netName, cidr } = useNetworkName();
    const { scanning, progress, devicesFound, startScan, cancel } = useScan();
    const [iface, setIface] = useState<LanInterfaceDto | null>(null);
    const [error, setError] = useState<string | null>(null);

    async function refresh() {
        try {
            setIface(await detectInterface());
            setError(null);
        } catch (e) {
            setError(String(e));
        }
    }

    useEffect(() => {
        refresh();
    }, []);

    // Re-detecta la interfaz cuando el scan compartido termina.
    const prevScanning = useRef(scanning);
    useEffect(() => {
        if (prevScanning.current && !scanning) refresh();
        prevScanning.current = scanning;
    }, [scanning]);

    const title = netName || cidr || iface?.name || "Detectando red…";
    const detected = Boolean(iface) && !error;

    return (
        <section aria-label="Red activa">
            <Card className="overflow-hidden">
                <div className="flex flex-col gap-4 p-4 md:flex-row md:items-start md:justify-between md:p-5">
                    <div className="flex min-w-0 flex-col gap-3">
                        <h2 className="flex min-w-0 items-center gap-2.5 text-lg font-semibold tracking-tight">
                            {/* Punto de estado semántico: red detectada / no. */}
                            <span
                                className={cn(
                                    "h-2 w-2 shrink-0 rounded-full",
                                    detected
                                        ? "bg-success"
                                        : "bg-muted-foreground/40",
                                )}
                                aria-hidden
                            />
                            <span className="truncate" title={title}>
                                {title}
                            </span>
                        </h2>

                        {error ? (
                            <div
                                role="alert"
                                className="flex flex-wrap items-center gap-3 text-sm text-destructive"
                            >
                                <span>
                                    No se detectó la interfaz de red: {error}
                                </span>
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
                                    Reintentar
                                </Button>
                            </div>
                        ) : (
                            <dl className="flex flex-wrap gap-x-8 gap-y-3">
                                <Readout label="Interfaz" value={iface?.name} />
                                <Readout
                                    label="IP"
                                    value={iface?.ip}
                                    suffix={
                                        iface
                                            ? `/${iface.prefix_len}`
                                            : undefined
                                    }
                                    field="ip"
                                    glossaryKey="cidr"
                                />
                                <Readout
                                    label="Gateway"
                                    value={iface?.gateway_ip ?? undefined}
                                    field="gateway_ip"
                                    glossaryKey="gateway"
                                />
                                <Readout
                                    label="MAC"
                                    value={iface?.mac ?? undefined}
                                    field="mac"
                                    glossaryKey="mac"
                                />
                                <Readout
                                    label="DNS"
                                    value={
                                        iface?.dns_servers.length
                                            ? iface.dns_servers.join(", ")
                                            : undefined
                                    }
                                    field="dns_servers"
                                    glossaryKey="dns"
                                />
                            </dl>
                        )}
                    </div>

                    <div className="flex shrink-0 items-center gap-2">
                        <ProfileSelect
                            value={profile}
                            onChange={onProfileChange}
                            disabled={scanning}
                            className="h-9 w-32"
                            id="discovery-profile"
                        />
                        {scanning ? (
                            <Button
                                variant="outline"
                                onClick={cancel}
                                className="gap-1.5 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
                            >
                                <Square
                                    className="h-3.5 w-3.5 fill-current"
                                    aria-hidden
                                />
                                Detener descubrimiento
                            </Button>
                        ) : (
                            <Button
                                onClick={() => startScan(profile)}
                                className="gap-1.5"
                            >
                                <Radio className="h-4 w-4" aria-hidden />
                                Descubrir dispositivos
                            </Button>
                        )}
                    </div>
                </div>

                {/* Franja de telemetría del barrido. */}
                {(scanning || lastScan) && (
                    <div
                        className="flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-border bg-muted/30 px-4 py-2.5 text-xs md:px-5"
                        role="status"
                        aria-live="polite"
                    >
                        {scanning ? (
                            <>
                                <span className="flex items-center gap-2 font-medium text-primary">
                                    <span className="relative flex h-2 w-2">
                                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-sky-400 opacity-75 motion-reduce:animate-none" />
                                        <span className="relative inline-flex h-2 w-2 rounded-full bg-sky-400" />
                                    </span>
                                    Descubriendo
                                </span>
                                <Progress
                                    value={progress?.percent ?? undefined}
                                    indeterminate={
                                        !progress || progress.total === 0
                                    }
                                    className="h-1.5 min-w-[8rem] flex-1"
                                    aria-label="Progreso del descubrimiento"
                                />
                                <span className="tabular-nums text-muted-foreground">
                                    {progress && progress.total > 0
                                        ? `${progress.swept} de ${progress.total} direcciones`
                                        : "Preparando barrido…"}
                                </span>
                                <span className="tabular-nums font-medium text-foreground">
                                    {devicesFound.length}{" "}
                                    {devicesFound.length === 1
                                        ? "encontrado"
                                        : "encontrados"}
                                </span>
                            </>
                        ) : (
                            lastScan && (
                                <span className="text-muted-foreground">
                                    Último descubrimiento:{" "}
                                    <span className="tabular-nums font-medium text-foreground">
                                        {lastScan.hosts_alive} activos
                                    </span>
                                    {", "}
                                    <span
                                        className={cn(
                                            "tabular-nums font-medium",
                                            lastScan.hosts_new > 0
                                                ? "text-amber-700 dark:text-amber-400"
                                                : "text-foreground",
                                        )}
                                    >
                                        {lastScan.hosts_new}{" "}
                                        {lastScan.hosts_new === 1
                                            ? "nuevo"
                                            : "nuevos"}
                                    </span>
                                    {" en "}
                                    <span className="tabular-nums">
                                        {(lastScan.duration_ms / 1000).toFixed(
                                            1,
                                        )}{" "}
                                        s
                                    </span>
                                </span>
                            )
                        )}
                    </div>
                )}
            </Card>
        </section>
    );
}

// Lectura técnica compacta (etiqueta arriba, valor mono abajo). Los campos
// sensibles pasan por MaskedValue para respetar el Modo Censura.
function Readout({
    label,
    value,
    suffix,
    field,
    glossaryKey,
}: {
    label: string;
    value: string | undefined;
    suffix?: string;
    field?: string;
    glossaryKey?: string;
}) {
    return (
        <div className="flex min-w-0 flex-col gap-0.5">
            <dt className="flex items-center gap-1 text-xs text-muted-foreground">
                {label}
                {glossaryKey && (
                    <InfoTooltip term={label} glossaryKey={glossaryKey} />
                )}
            </dt>
            <dd className="flex items-baseline font-mono text-sm font-medium tabular-nums text-foreground">
                {value === undefined ? (
                    <span className="text-muted-foreground">—</span>
                ) : field && isSensitive(field) ? (
                    <MaskedValue field={field} value={value} mono />
                ) : (
                    <span className="truncate">{value}</span>
                )}
                {value !== undefined && suffix && (
                    <span className="text-muted-foreground">{suffix}</span>
                )}
            </dd>
        </div>
    );
}
