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
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
    Wifi,
    Radio,
    Play,
    X,
    Network as NetworkIcon,
    Cpu,
    Router as RouterIcon,
    AlertCircle,
    RefreshCw,
    ChevronDown,
    Activity,
    Shield,
} from "lucide-react";
import {
    detectInterface,
    getSettings,
    listDevices,
    type Device,
    type LanInterfaceDto,
} from "@/lib/tauri";
import { useLastScan, useScan } from "@/App";
import { useNetworkName } from "@/lib/use-network-name";
import { MaskedValue } from "@/components/masked-value";
import { isSensitive } from "@/lib/censor";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { FormField } from "@/components/ui/form-field";
import { SECTION_GAP } from "@/lib/design-tokens";
import { cn } from "@/lib/utils";

export function Dashboard() {
    const { lastScan } = useLastScan();
    const { name: netName, cidr } = useNetworkName();
    const { scanning, progress, startScan, cancel } = useScan();
    const [iface, setIface] = useState<LanInterfaceDto | null>(null);
    const [devices, setDevices] = useState<Device[]>([]);
    const [profile, setProfile] = useState("normal");
    const [error, setError] = useState<string | null>(null);
    const [openNet, setOpenNet] = useState(false); // Collapsed by default to avoid info-overload

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
        <div className={cn("flex flex-col", SECTION_GAP)} aria-busy={scanning}>
            <section aria-label="Red activa">
                <Card className="overflow-hidden shadow-sm">
                    <Collapsible open={openNet} onOpenChange={setOpenNet}>
                        <CollapsibleTrigger asChild>
                            <button className="flex w-full items-center justify-between p-5 text-left hover:bg-muted/30 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                <div className="flex flex-col gap-1 min-w-0">
                                    <div className="flex items-center gap-2 text-base md:text-lg font-bold tracking-tight">
                                        <div className="relative flex h-2 w-2">
                                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                                        </div>
                                        <span>Red activa:</span>
                                        <span className="text-primary truncate font-extrabold max-w-[200px] sm:max-w-xs md:max-w-md">
                                            {netName ||
                                                cidr ||
                                                iface?.name ||
                                                "Detectando..."}
                                        </span>
                                    </div>
                                    <p className="text-xs text-muted-foreground mt-0.5">
                                        Haz clic para expandir o contraer la
                                        información técnica de red
                                    </p>
                                </div>
                                <ChevronDown
                                    className={cn(
                                        "h-5 w-5 text-muted-foreground transition-transform duration-300",
                                        openNet && "transform rotate-180",
                                    )}
                                    aria-hidden
                                />
                            </button>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="border-t border-border/20">
                            <CardContent className="grid gap-6 p-5 sm:grid-cols-2 lg:grid-cols-3 bg-muted/10">
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
                                    suffix={
                                        iface
                                            ? `/${iface.prefix_len}`
                                            : undefined
                                    }
                                    glossaryKey="cidr"
                                />
                                <Info
                                    label="Gateway"
                                    value={iface?.gateway_ip ?? "—"}
                                    icon={RouterIcon}
                                    field={iface ? "gateway_ip" : undefined}
                                    glossaryKey="gateway"
                                />
                                <Info
                                    label="MAC"
                                    value={iface?.mac ?? "—"}
                                    icon={NetworkIcon}
                                    field={iface ? "mac" : undefined}
                                    glossaryKey="mac"
                                />
                                <Info
                                    label="DNS"
                                    value={
                                        iface
                                            ? iface.dns_servers.join(", ") ||
                                              "—"
                                            : "—"
                                    }
                                    icon={Wifi}
                                    field={
                                        iface && iface.dns_servers.length > 0
                                            ? "dns_servers"
                                            : undefined
                                    }
                                    glossaryKey="dns"
                                />
                            </CardContent>
                        </CollapsibleContent>
                    </Collapsible>
                </Card>
            </section>

            <section aria-label="Resumen">
                <div className="grid gap-4 sm:grid-cols-3">
                    <Stat
                        label="Dispositivos detectados"
                        value={devices.length}
                        icon={NetworkIcon}
                        tone="primary"
                    />
                    <Stat
                        label="Activos en último scan"
                        value={lastScan?.hosts_alive ?? 0}
                        icon={Activity}
                        tone="success"
                    />
                    <Stat
                        label="Nuevos dispositivos"
                        value={lastScan?.hosts_new ?? 0}
                        icon={Shield}
                        tone="warning"
                    />
                </div>
            </section>

            <section aria-label="Descubrir dispositivos">
                <Card className="shadow-sm overflow-hidden">
                    <CardHeader className="p-4">
                        <CardTitle className="flex items-center gap-2 text-base md:text-lg font-bold">
                            <Radio
                                className="h-5 w-5 text-primary"
                                aria-hidden
                            />
                            Descubrir dispositivos
                        </CardTitle>
                        <CardDescription>
                            Explora y encuentra los dispositivos en tu red local
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-6 p-4 pt-0">
                        {scanning ? (
                            <div className="flex flex-col items-center justify-center py-6 gap-4 border border-dashed border-primary/20 bg-primary/5 rounded-lg">
                                <div className="relative flex h-24 w-24 items-center justify-center">
                                    <div className="absolute inset-0 rounded-full border border-primary/30 animate-radar-ring-1" />
                                    <div className="absolute inset-0 rounded-full border border-primary/20 animate-radar-ring-2" />
                                    <div className="absolute inset-0 rounded-full border border-primary/10 animate-radar-ring-3" />
                                    <div className="z-10 flex h-12 w-12 items-center justify-center rounded-full bg-primary/20 text-primary shadow-[0_0_15px_rgba(56,189,248,0.25)] animate-pulse">
                                        <Radio className="h-6 w-6" />
                                    </div>
                                </div>
                                <div className="text-center px-4">
                                    <p className="text-sm font-bold text-primary animate-pulse">
                                        Escaneando red local...
                                    </p>
                                    <p className="text-xs text-muted-foreground mt-1 max-w-[280px] sm:max-w-md">
                                        {progress && progress.total > 0
                                            ? `${progress.swept} de ${progress.total} hosts analizados (${progress.percent}%)`
                                            : "Analizando puertos y detectando hostnames activos..."}
                                    </p>
                                </div>
                                <div className="w-full max-w-xs px-4">
                                    <Progress
                                        value={progress?.percent ?? undefined}
                                        indeterminate={
                                            !progress || progress.total === 0
                                        }
                                        className="h-1.5"
                                    />
                                </div>
                                <Button
                                    variant="destructive"
                                    size="sm"
                                    onClick={cancel}
                                    className="gap-1.5 mt-2 transition-all hover:bg-destructive/90"
                                >
                                    <X className="h-4 w-4" aria-hidden />
                                    Detener escaneo
                                </Button>
                            </div>
                        ) : (
                            <FormField
                                label="Configuración del perfil"
                                htmlFor="dash-profile"
                                helper="Selecciona el tipo de barrido de red. Los perfiles rápidos escanean puertos conocidos; los profundos revisan todo el rango."
                            >
                                <div className="flex flex-wrap items-center gap-3 mt-1.5">
                                    <ProfileSelect
                                        value={profile}
                                        onChange={setProfile}
                                        className="w-44"
                                        id="dash-profile"
                                    />
                                    <Button
                                        onClick={() => startScan(profile)}
                                        disabled={scanning}
                                        className="gap-1.5 shadow-sm transition-all hover:opacity-90 font-medium"
                                    >
                                        <Play className="h-4 w-4" aria-hidden />
                                        Iniciar descubrimiento
                                    </Button>
                                </div>
                            </FormField>
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
                        action={
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={refresh}
                                className="gap-1.5"
                            >
                                <RefreshCw className="h-4 w-4" aria-hidden />
                                Reintentar
                            </Button>
                        }
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
    glossaryKey,
}: {
    label: string;
    value: string;
    icon: typeof Wifi;
    field?: string;
    suffix?: string;
    glossaryKey?: string;
}) {
    return (
        <div className="flex flex-col gap-1">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground font-semibold">
                <Icon className="h-3.5 w-3.5 text-primary/70" aria-hidden />
                {label}
                {glossaryKey && (
                    <InfoTooltip term={label} glossaryKey={glossaryKey} />
                )}
            </span>
            {field && isSensitive(field) ? (
                <span className="flex items-baseline gap-0.5 font-bold text-foreground mt-0.5">
                    <MaskedValue field={field} value={value} />
                    {suffix && (
                        <span className="text-[10px] text-muted-foreground font-normal">
                            {suffix}
                        </span>
                    )}
                </span>
            ) : (
                <span className="font-bold text-foreground mt-0.5">
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
    tone = "primary",
}: {
    label: string;
    value: number;
    icon: typeof Wifi;
    tone?: "primary" | "success" | "warning";
}) {
    // #3: chip con fondo tonal derivado del token semántico (sin gradientes).
    const toneClasses = {
        primary: "bg-primary/10 text-primary",
        success: "bg-success/10 text-success",
        warning: "bg-warning/10 text-warning",
    } as const;
    return (
        <Card className="shadow-sm">
            <CardContent className="flex items-center gap-4 p-4">
                <div
                    className={cn(
                        "flex h-11 w-11 items-center justify-center rounded-lg",
                        toneClasses[tone],
                    )}
                >
                    <Icon className="h-5 w-5" aria-hidden />
                </div>
                <div className="flex flex-col">
                    <span className="text-xs font-semibold text-muted-foreground">
                        {label}
                    </span>
                    <span className="text-2xl font-extrabold tracking-tight mt-0.5">
                        {value}
                    </span>
                </div>
            </CardContent>
        </Card>
    );
}
