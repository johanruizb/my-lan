import { useEffect, useRef, useState } from "react";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ProfileSelect } from "@/components/profile-select";
import { EmptyState } from "@/components/empty-state";
import {
    Wifi,
    Radio,
    Loader2,
    Play,
    X,
    Network as NetworkIcon,
    Cpu,
    Router as RouterIcon,
    AlertCircle,
} from "lucide-react";
import {
    detectInterface,
    getSettings,
    listDevices,
    type Device,
    type LanInterfaceDto,
} from "@/lib/tauri";
import { useLastScan, useScan } from "@/App";
import { MaskedValue } from "@/components/masked-value";
import { isSensitive } from "@/lib/censor";

export function Dashboard() {
    const { lastScan } = useLastScan();
    const { scanning, progress, startScan, cancel } = useScan();
    const [iface, setIface] = useState<LanInterfaceDto | null>(null);
    const [devices, setDevices] = useState<Device[]>([]);
    const [profile, setProfile] = useState("normal");
    const [error, setError] = useState<string | null>(null);

    async function refresh() {
        try {
            const [ifaceRes, devicesRes] = await Promise.all([
                detectInterface(),
                listDevices().catch(() => [] as Device[]),
            ]);
            setIface(ifaceRes);
            setDevices(devicesRes);
            setError(null);
        } catch (e) {
            setError(String(e));
        }
    }

    useEffect(() => {
        refresh();
        // Perfil inicial desde Ajustes (en vez del "normal" hardcodeado).
        getSettings()
            .then((s) => setProfile(s.default_profile))
            .catch(() => {});
    }, []);

    // Refresca el inventario cuando el scan compartido termina.
    const prevScanning = useRef(scanning);
    useEffect(() => {
        if (prevScanning.current && !scanning) refresh();
        prevScanning.current = scanning;
    }, [scanning]);

    return (
        <div className="flex flex-col gap-2" aria-busy={scanning}>
            <section aria-label="Red activa">
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Wifi
                                className="h-5 w-5 text-primary"
                                aria-hidden
                            />
                            Red activa
                        </CardTitle>
                        <CardDescription>
                            Interfaz detectada automáticamente como default
                            route.
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="grid gap-4 sm:grid-cols-3">
                        <Info
                            label="Interfaz"
                            value={iface?.name ?? "—"}
                            icon={NetworkIcon}
                        />
                        <Info
                            label="IP / CIDR"
                            value={iface?.ip ?? "—"}
                            icon={Cpu}
                            field={iface ? "ip" : undefined}
                            suffix={iface ? `/${iface.prefix_len}` : undefined}
                        />
                        <Info
                            label="Gateway"
                            value={iface?.gateway_ip ?? "—"}
                            icon={RouterIcon}
                            field={iface ? "gateway_ip" : undefined}
                        />
                        <Info
                            label="MAC"
                            value={iface?.mac ?? "—"}
                            icon={NetworkIcon}
                            field={iface ? "mac" : undefined}
                        />
                        <Info
                            label="DNS"
                            value={
                                iface
                                    ? iface.dns_servers.join(", ") || "—"
                                    : "—"
                            }
                            icon={Wifi}
                            field={
                                iface && iface.dns_servers.length > 0
                                    ? "dns_servers"
                                    : undefined
                            }
                        />
                    </CardContent>
                </Card>
            </section>

            <section aria-label="Resumen">
                <div className="grid gap-2 sm:grid-cols-3">
                    <Stat
                        label="Dispositivos"
                        value={devices.length}
                        icon={NetworkIcon}
                    />
                    <Stat
                        label="Hosts vivos (último scan)"
                        value={lastScan?.hosts_alive ?? 0}
                        icon={Radio}
                    />
                    <Stat
                        label="Nuevos (último scan)"
                        value={lastScan?.hosts_new ?? 0}
                        icon={Radio}
                    />
                </div>
            </section>

            <section aria-label="Escanear ahora">
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Radio
                                className="h-5 w-5 text-primary"
                                aria-hidden
                            />
                            Escanear ahora
                        </CardTitle>
                        <CardDescription>
                            Descubre los hosts de tu LAN con el perfil
                            seleccionado.
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="flex flex-wrap items-center gap-4">
                        <div className="flex flex-col gap-1.5">
                            <label
                                htmlFor="dash-profile"
                                className="text-xs text-muted-foreground"
                            >
                                Perfil
                            </label>
                            <ProfileSelect
                                value={profile}
                                onChange={setProfile}
                                className="w-40"
                                id="dash-profile"
                            />
                        </div>
                        <Button
                            onClick={() => startScan(profile)}
                            disabled={scanning}
                            className="mt-5"
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
                                    Escanear ahora
                                </>
                            )}
                        </Button>
                        {/* Cancelar compartido con /devices (AC-8): detiene el
                            scan y conserva los hosts ya hallados y persistidos. */}
                        {scanning && (
                            <Button
                                variant="outline"
                                onClick={cancel}
                                className="mt-5 gap-1.5"
                            >
                                <X className="h-4 w-4" aria-hidden />
                                Cancelar
                            </Button>
                        )}
                        {/* Mismo progreso compartido que /devices (AC-3/AC-11). */}
                        {scanning && (
                            <div
                                className="flex w-full flex-col gap-1.5"
                                aria-live="polite"
                            >
                                <div className="flex items-center justify-between text-xs text-muted-foreground">
                                    <span>Escaneando red…</span>
                                    <span>
                                        {progress && progress.total > 0
                                            ? `${progress.swept}/${progress.total} (${progress.percent}%)`
                                            : "Sondeando…"}
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
                    </CardContent>
                </Card>
            </section>

            {error && (
                <div role="alert" aria-live="polite">
                    <EmptyState
                        icon={AlertCircle}
                        title="Error de red"
                        description={error}
                        className="border-red-300 bg-red-50 text-red-900 dark:border-red-900 dark:bg-red-950 dark:text-red-100"
                    />
                </div>
            )}
        </div>
    );
}

function Info({
    label,
    value,
    icon: Icon,
    field,
    suffix,
}: {
    label: string;
    value: string;
    icon: typeof Wifi;
    field?: string;
    suffix?: string;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <Icon className="h-3.5 w-3.5" aria-hidden />
                {label}
            </span>
            {field && isSensitive(field) ? (
                <span className="flex items-baseline gap-0.5 font-medium">
                    <MaskedValue field={field} value={value} />
                    {suffix && <span>{suffix}</span>}
                </span>
            ) : (
                <span className="font-medium">
                    {value}
                    {suffix}
                </span>
            )}
        </div>
    );
}

function Stat({
    label,
    value,
    icon: Icon,
}: {
    label: string;
    value: number;
    icon: typeof Wifi;
}) {
    return (
        <Card>
            <CardContent className="flex items-center gap-3 p-4">
                <div className="flex h-9 w-9 items-center justify-center rounded-md bg-muted text-muted-foreground">
                    <Icon className="h-4 w-4" aria-hidden />
                </div>
                <div className="flex flex-col">
                    <span className="text-xs text-muted-foreground">
                        {label}
                    </span>
                    <span className="text-2xl font-bold leading-tight">
                        {value}
                    </span>
                </div>
            </CardContent>
        </Card>
    );
}
