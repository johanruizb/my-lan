import { useEffect, useState } from "react";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProfileSelect } from "@/components/profile-select";
import { EmptyState } from "@/components/empty-state";
import { useToast } from "@/components/ui/toast";
import {
    Wifi,
    Radio,
    Loader2,
    Play,
    Network as NetworkIcon,
    Cpu,
    Router as RouterIcon,
    AlertCircle,
} from "lucide-react";
import {
    detectInterface,
    listDevices,
    runDiscovery,
    type Device,
    type LanInterfaceDto,
} from "@/lib/tauri";
import { useLastScan } from "@/App";

export function Dashboard() {
    const { toast } = useToast();
    const { lastScan, setLastScan } = useLastScan();
    const [iface, setIface] = useState<LanInterfaceDto | null>(null);
    const [devices, setDevices] = useState<Device[]>([]);
    const [profile, setProfile] = useState("normal");
    const [scanning, setScanning] = useState(false);
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
    }, []);

    async function handleScan() {
        setScanning(true);
        setError(null);
        try {
            const outcome = await runDiscovery(profile);
            setLastScan(outcome);
            const devicesRes = await listDevices().catch(() => [] as Device[]);
            setDevices(devicesRes);
            toast(
                `Escaneo completado: ${outcome.hosts_alive} hosts vivos, ${outcome.hosts_new} nuevos.`,
                "success",
            );
        } catch (e) {
            const msg = String(e);
            setError(msg);
            toast(`Error: ${msg}`, "error");
        } finally {
            setScanning(false);
        }
    }

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
                            value={
                                iface ? `${iface.ip}/${iface.prefix_len}` : "—"
                            }
                            icon={Cpu}
                        />
                        <Info
                            label="Gateway"
                            value={iface?.gateway_ip ?? "—"}
                            icon={RouterIcon}
                        />
                        <Info
                            label="MAC"
                            value={iface?.mac ?? "—"}
                            icon={NetworkIcon}
                        />
                        <Info
                            label="DNS"
                            value={
                                iface
                                    ? iface.dns_servers.join(", ") || "—"
                                    : "—"
                            }
                            icon={Wifi}
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
                            onClick={handleScan}
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
}: {
    label: string;
    value: string;
    icon: typeof Wifi;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <Icon className="h-3.5 w-3.5" aria-hidden />
                {label}
            </span>
            <span className="font-medium">{value}</span>
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
