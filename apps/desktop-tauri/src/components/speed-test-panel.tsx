import { useEffect, useRef, useState } from "react";
import {
    Activity,
    ArrowDown,
    ArrowUp,
    Globe,
    Loader2,
    Play,
    Radio,
    RefreshCw,
    Server,
    Square,
    Wifi,
    Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { useToast } from "@/components/ui/toast";
import { useCensorship } from "@/components/censorship-provider";
import {
    cancelTool,
    getNetworkHealthSamples,
    getRecentSpeedTests,
    onSpeedTestDone,
    onSpeedTestProgress,
    runHealthProbe,
    startSpeedTest,
    type NetworkHealthSample,
    type SpeedTestProgress,
    type SpeedTestResult,
} from "@/lib/tauri";
import { maskIp } from "@/lib/censor";

export function SpeedTestPanel() {
    const { toast } = useToast();
    const { censorshipEnabled } = useCensorship();

    const [speedTesting, setSpeedTesting] = useState(false);
    const [speedProgress, setSpeedProgress] = useState<SpeedTestProgress | null>(null);
    const [speedResult, setSpeedResult] = useState<SpeedTestResult | null>(null);
    const [speedHistory, setSpeedHistory] = useState<SpeedTestResult[]>([]);
    const [healthSamples, setHealthSamples] = useState<NetworkHealthSample[]>([]);
    const [healthProbing, setHealthProbing] = useState(false);
    const speedSessionRef = useRef<string | null>(null);
    const toastedSessionsRef = useRef<Set<string>>(new Set());

    useEffect(() => {
        loadData();
    }, []);

    async function loadData() {
        try {
            if (typeof getRecentSpeedTests === "function") {
                const recent = await getRecentSpeedTests({ limit: 10 });
                setSpeedHistory(recent || []);
                if (recent && recent.length > 0) {
                    setSpeedResult(recent[0]);
                }
            }
            if (typeof getNetworkHealthSamples === "function") {
                const samples = await getNetworkHealthSamples(undefined, 10);
                setHealthSamples(samples || []);
            }
        } catch {
            // Silencioso ante mock / offline
        }
    }

    useEffect(() => {
        // Patrón disposed: la Promise de listen() resuelve tras el round-trip
        // IPC, DESPUÉS del primer cleanup de StrictMode. Sin este flag, el
        // unlisten llega tarde y queda registrado (listener zombie que
        // duplica cada evento). Si la Promise resuelve tras el unmount
        // (disposed), desregistramos inmediatamente.
        let disposed = false;
        const unlistens: (() => void)[] = [];
        const register = (
            promiseLike: Promise<unknown> | undefined,
            pending: (() => void)[],
        ) => {
            if (typeof promiseLike?.then !== "function") return;
            promiseLike
                .then((u) => {
                    if (disposed) (u as (() => void) | undefined)?.();
                    else if (typeof u === "function") pending.push(u);
                })
                .catch(() => {});
        };

        register(
            onSpeedTestProgress?.((payload) => {
                if (payload.session_id === speedSessionRef.current) {
                    setSpeedProgress(payload.progress);
                }
            }),
            unlistens,
        );

        register(
            onSpeedTestDone?.((payload) => {
                if (payload.session_id === speedSessionRef.current) {
                    if (toastedSessionsRef.current.has(payload.session_id)) return;
                    toastedSessionsRef.current.add(payload.session_id);
                    setSpeedTesting(false);
                    if (payload.result) {
                        setSpeedResult(payload.result);
                        setSpeedHistory((prev) => [payload.result!, ...prev]);
                        toast("Test de velocidad completado exitosamente.", "success");
                    } else if (payload.error) {
                        toast(`Test de velocidad finalizado o cancelado: ${payload.error}`, "info");
                    }
                }
            }),
            unlistens,
        );

        return () => {
            disposed = true;
            unlistens.forEach((u) => u());
            if (speedSessionRef.current) {
                try {
                    cancelTool(speedSessionRef.current)?.catch?.(() => {});
                } catch {
                    // Ignore
                }
            }
        };
    }, [toast]);

    async function handleStartSpeedTest() {
        const sessionId = `speed-${Date.now()}`;
        speedSessionRef.current = sessionId;
        toastedSessionsRef.current.clear();
        setSpeedTesting(true);
        setSpeedProgress(null);

        try {
            await startSpeedTest(sessionId);
        } catch (e) {
            setSpeedTesting(false);
            toast(`Error al iniciar test de velocidad: ${e}`, "error");
        }
    }

    async function handleCancelSpeedTest() {
        if (speedSessionRef.current) {
            try {
                await cancelTool(speedSessionRef.current);
                setSpeedTesting(false);
                toast("Test de velocidad cancelado por el usuario.", "info");
            } catch (e) {
                toast(`Error al cancelar: ${e}`, "error");
            }
        }
    }

    async function handleRunHealthProbe() {
        setHealthProbing(true);
        try {
            const samples = await runHealthProbe();
            setHealthSamples((prev) => [...samples, ...prev]);
            toast("Sondeo de salud de red actualizado.", "success");
        } catch (e) {
            toast(`Error al sondear salud: ${e}`, "error");
        } finally {
            setHealthProbing(false);
        }
    }

    // Helper de censura para IPs
    function displayIp(ip: string | null | undefined): string {
        if (!ip) return "-";
        return censorshipEnabled ? maskIp(ip) : ip;
    }

    // Traduce la fase activa del test a texto descriptivo
    function phaseText(phase: string | undefined): string {
        switch (phase) {
            case "init":
                return "Conectando con nodo Anycast de Cloudflare...";
            case "latency":
                return "Midiendo latencia RTT y Jitter RFC 3550...";
            case "download":
                return "Midiendo velocidad de descarga (Chunks progresivos 1MB - 25MB)...";
            case "upload":
                return "Midiendo velocidad de subida (Payloads POST 1MB - 10MB)...";
            case "finished":
                return "Medición completada";
            case "error":
                return "Error durante la medición";
            default:
                return "Listo para iniciar medición";
        }
    }

    // Helper para identificar direcciones loopback
    function isLoopbackIp(ip: string | null | undefined): boolean {
        if (!ip) return false;
        return ip.startsWith("127.") || ip === "::1" || ip === "localhost";
    }

    // Separa la muestra más reciente para gateway y dns (priorizando no-loopback)
    const latestGateway = healthSamples.find((s) => s.target_type === "gateway");
    const latestDns =
        healthSamples.find(
            (s) =>
                (s.target_type === "dns" || s.target_type === "internet") &&
                !isLoopbackIp(s.target_ip),
        ) ??
        healthSamples.find(
            (s) => s.target_type === "dns" || s.target_type === "internet",
        );

    return (
        <div className="flex flex-col gap-6">
            {/* SECCIÓN 1: TEST DE VELOCIDAD */}
            <Card className="border-border">
                <CardHeader className="p-5 pb-3">
                    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                        <div>
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Zap className="h-4 w-4 text-amber-500" aria-hidden />
                                Test de Velocidad (speed.cloudflare.com)
                            </CardTitle>
                            <CardDescription className="text-xs">
                                Medición Anycast de ancho de banda real de descarga y subida, latencia RTT y jitter RFC 3550 contra la red global de Cloudflare.
                            </CardDescription>
                        </div>

                        <div className="flex items-center gap-2">
                            {speedTesting ? (
                                <Button
                                    type="button"
                                    variant="destructive"
                                    size="sm"
                                    onClick={handleCancelSpeedTest}
                                    className="gap-1.5 h-8 text-xs"
                                >
                                    <Square className="h-3.5 w-3.5" aria-hidden />
                                    Cancelar
                                </Button>
                            ) : (
                                <Button
                                    type="button"
                                    variant="default"
                                    size="sm"
                                    onClick={handleStartSpeedTest}
                                    className="gap-1.5 h-8 text-xs"
                                >
                                    <Play className="h-3.5 w-3.5 fill-current" aria-hidden />
                                    Iniciar Test
                                </Button>
                            )}
                        </div>
                    </div>
                </CardHeader>

                <CardContent className="p-5 pt-2 flex flex-col gap-5">
                    {/* Barra de progreso de la medición activa */}
                    {speedTesting && (
                        <div className="flex flex-col gap-1.5 p-3 rounded-lg bg-muted/40 border border-border">
                            <div className="flex items-center justify-between text-xs">
                                <span className="font-medium text-foreground flex items-center gap-2">
                                    <Loader2 className="h-3.5 w-3.5 animate-spin text-amber-500" />
                                    {phaseText(speedProgress?.phase)}
                                </span>
                                <span className="font-mono text-muted-foreground">
                                    {Math.round(speedProgress?.progress_pct || 0)}%
                                </span>
                            </div>
                            <div className="w-full bg-secondary h-2 rounded-full overflow-hidden">
                                <div
                                    className="bg-amber-500 h-full rounded-full transition-all duration-300"
                                    style={{ width: `${Math.max(2, speedProgress?.progress_pct || 0)}%` }}
                                />
                            </div>
                        </div>
                    )}

                    {/* Banner del servidor detectado */}
                    {(speedProgress?.server_info || speedResult?.server_name) && (
                        <div className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 rounded-md bg-muted/20 border border-border text-xs">
                            <div className="flex items-center gap-2 text-muted-foreground">
                                <Server className="h-3.5 w-3.5 text-primary" />
                                <span>Servidor:</span>
                                <span className="font-medium text-foreground">
                                    {speedProgress?.server_info || speedResult?.server_name}
                                </span>
                            </div>
                            {speedResult?.server_country && (
                                <span className="px-2 py-0.5 rounded text-[11px] font-mono bg-secondary text-secondary-foreground">
                                    País: {speedResult.server_country}
                                </span>
                            )}
                        </div>
                    )}

                    {/* Panel de Métricas Principales */}
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                        {/* Descarga */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm relative overflow-hidden">
                            <div className="flex items-center justify-between text-muted-foreground text-xs font-medium mb-1">
                                <span className="flex items-center gap-1.5">
                                    <ArrowDown className="h-4 w-4 text-emerald-500" />
                                    Descarga
                                </span>
                                <span className="text-[11px] font-mono uppercase">Mbps</span>
                            </div>
                            <span className="text-3xl font-extrabold tracking-tight text-foreground font-mono">
                                {speedProgress?.phase === "download"
                                    ? speedProgress.current_mbps.toFixed(1)
                                    : speedResult
                                      ? speedResult.download_mbps.toFixed(1)
                                      : "0.0"}
                            </span>
                            <div className="mt-1 text-[11px] text-muted-foreground">
                                {speedProgress?.phase === "download" ? "Midiendo en vivo..." : "Velocidad de bajada"}
                            </div>
                        </div>

                        {/* Subida */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm relative overflow-hidden">
                            <div className="flex items-center justify-between text-muted-foreground text-xs font-medium mb-1">
                                <span className="flex items-center gap-1.5">
                                    <ArrowUp className="h-4 w-4 text-cyan-500" />
                                    Subida
                                </span>
                                <span className="text-[11px] font-mono uppercase">Mbps</span>
                            </div>
                            <span className="text-3xl font-extrabold tracking-tight text-foreground font-mono">
                                {speedProgress?.phase === "upload"
                                    ? speedProgress.current_mbps.toFixed(1)
                                    : speedResult
                                      ? speedResult.upload_mbps.toFixed(1)
                                      : "0.0"}
                            </span>
                            <div className="mt-1 text-[11px] text-muted-foreground">
                                {speedProgress?.phase === "upload" ? "Midiendo en vivo..." : "Velocidad de carga"}
                            </div>
                        </div>

                        {/* Latencia RTT */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm relative overflow-hidden">
                            <div className="flex items-center justify-between text-muted-foreground text-xs font-medium mb-1">
                                <span className="flex items-center gap-1.5">
                                    <Activity className="h-4 w-4 text-amber-500" />
                                    Latencia (Ping)
                                </span>
                                <span className="text-[11px] font-mono uppercase">ms</span>
                            </div>
                            <span className="text-3xl font-extrabold tracking-tight text-foreground font-mono">
                                {speedProgress?.latency_ms ?? speedResult?.latency_ms ?? "-"}
                            </span>
                            <div className="mt-1 text-[11px] text-muted-foreground">
                                RTT Anycast hacia Cloudflare
                            </div>
                        </div>

                        {/* Jitter RFC 3550 */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm relative overflow-hidden">
                            <div className="flex items-center justify-between text-muted-foreground text-xs font-medium mb-1">
                                <span className="flex items-center gap-1.5">
                                    <Radio className="h-4 w-4 text-purple-500" />
                                    Jitter
                                </span>
                                <span className="text-[11px] font-mono uppercase">ms</span>
                            </div>
                            <span className="text-3xl font-extrabold tracking-tight text-foreground font-mono">
                                {speedProgress?.jitter_ms != null
                                    ? speedProgress.jitter_ms.toFixed(1)
                                    : speedResult?.jitter_ms != null
                                      ? speedResult.jitter_ms.toFixed(1)
                                      : "-"}
                            </span>
                            <div className="mt-1 text-[11px] text-muted-foreground">
                                Variación RFC 3550 (VoIP/Gaming)
                            </div>
                        </div>
                    </div>

                    {/* Historial de Tests de Velocidad */}
                    {speedHistory.length > 0 && (
                        <div className="flex flex-col gap-2 pt-2 border-t border-border">
                            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                                Historial de mediciones recientes
                            </span>
                            <div className="rounded-md border border-border overflow-hidden">
                                <Table className="text-xs text-left">
                                    <TableHeader className="bg-muted/50 text-muted-foreground font-medium border-b border-border">
                                        <TableRow>
                                            <TableHead className="py-2 px-3">Fecha y Hora</TableHead>
                                            <TableHead className="py-2 px-3">Servidor</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Bajada</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Subida</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Latencia</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Jitter</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody className="divide-y divide-border font-mono">
                                        {speedHistory.map((item) => (
                                            <TableRow key={item.id} className="hover:bg-muted/20">
                                                <TableCell className="py-2 px-3 text-muted-foreground font-sans">
                                                    {new Date(item.created_at).toLocaleString()}
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-foreground font-sans">
                                                    {item.server_city
                                                        ? `${item.server_name} (${item.server_city})`
                                                        : item.server_name}
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-emerald-600 dark:text-emerald-400 font-semibold">
                                                    {item.download_mbps.toFixed(1)} Mbps
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-cyan-600 dark:text-cyan-400 font-semibold">
                                                    {item.upload_mbps.toFixed(1)} Mbps
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-muted-foreground">
                                                    {item.latency_ms} ms
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-muted-foreground">
                                                    {item.jitter_ms.toFixed(1)} ms
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* SECCIÓN 2: SALUD DE RED LOCAL Y ENLACE */}
            <Card className="border-border">
                <CardHeader className="p-5 pb-3">
                    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                        <div>
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Activity className="h-4 w-4 text-emerald-500" aria-hidden />
                                Salud y Calidad del Enlace Local (Gateway & DNS)
                            </CardTitle>
                            <CardDescription className="text-xs">
                                Diagnóstico pasivo no root de latencia RTT, variación de paquetes y jitter RFC 3550 hacia el router local y los resolvedores DNS.
                            </CardDescription>
                        </div>

                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            disabled={healthProbing}
                            onClick={handleRunHealthProbe}
                            className="gap-1.5 h-8 text-xs"
                        >
                            {healthProbing ? (
                                <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                            ) : (
                                <RefreshCw className="h-3.5 w-3.5" aria-hidden />
                            )}
                            Sondear Ahora
                        </Button>
                    </div>
                </CardHeader>

                <CardContent className="p-5 pt-2 flex flex-col gap-5">
                    {/* Tarjetas de estado Gateway y DNS */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {/* Gateway */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm">
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                    <Wifi className="h-4 w-4 text-primary" />
                                    <span className="font-semibold text-sm">Router / Gateway Local</span>
                                </div>
                                {latestGateway && (
                                    <span
                                        className={`px-2 py-0.5 rounded text-[11px] font-medium ${
                                            (latestGateway.rtt_ms ?? 999) < 10 && latestGateway.loss_pct === 0
                                                ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                                                : (latestGateway.rtt_ms ?? 999) < 30
                                                  ? "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20"
                                                  : "bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20"
                                        }`}
                                    >
                                        {(latestGateway.rtt_ms ?? 999) < 10 && latestGateway.loss_pct === 0
                                            ? "Excelente"
                                            : (latestGateway.rtt_ms ?? 999) < 30
                                              ? "Aceptable"
                                              : "Degradado"}
                                    </span>
                                )}
                            </div>

                            {latestGateway ? (
                                <div className="grid grid-cols-2 gap-2 mt-2 pt-2 border-t border-border text-xs">
                                    <div>
                                        <span className="text-muted-foreground block text-[11px]">IP Destino</span>
                                        <span className="font-mono font-medium">{displayIp(latestGateway.target_ip)}</span>
                                    </div>
                                    <div>
                                        <span className="text-muted-foreground block text-[11px]">Latencia RTT</span>
                                        <span className="font-mono font-medium">
                                            {latestGateway.rtt_ms != null ? `${latestGateway.rtt_ms} ms` : "Inalcanzable"}
                                        </span>
                                    </div>
                                    <div>
                                        <span className="text-muted-foreground block text-[11px]">Jitter RFC 3550</span>
                                        <span className="font-mono font-medium">{latestGateway.jitter_ms.toFixed(1)} ms</span>
                                    </div>
                                    <div>
                                        <span className="text-muted-foreground block text-[11px]">Pérdida / Método</span>
                                        <span className="font-mono font-medium">
                                            {latestGateway.loss_pct.toFixed(0)}% ({latestGateway.method})
                                        </span>
                                    </div>
                                </div>
                            ) : (
                                <p className="text-xs text-muted-foreground italic mt-2">
                                    Aún no hay muestras registradas. Pulsa &quot;Sondear Ahora&quot; para medir.
                                </p>
                            )}
                        </div>

                        {/* DNS Principal */}
                        <div className="flex flex-col p-4 rounded-xl bg-card border border-border shadow-sm">
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                    <Globe className="h-4 w-4 text-cyan-500" />
                                    <span className="font-semibold text-sm">Servidor DNS / Enlace Salida</span>
                                </div>
                                {latestDns && (
                                    <span
                                        className={`px-2 py-0.5 rounded text-[11px] font-medium ${
                                            isLoopbackIp(latestDns.target_ip)
                                                ? "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20"
                                                : (latestDns.rtt_ms ?? 999) < 30 && latestDns.loss_pct === 0
                                                  ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                                                  : (latestDns.rtt_ms ?? 999) < 60
                                                    ? "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20"
                                                    : "bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20"
                                        }`}
                                    >
                                        {isLoopbackIp(latestDns.target_ip)
                                            ? "Stub Local (No representativo)"
                                            : (latestDns.rtt_ms ?? 999) < 30 && latestDns.loss_pct === 0
                                              ? "Excelente"
                                              : (latestDns.rtt_ms ?? 999) < 60
                                                ? "Aceptable"
                                                : "Degradado"}
                                    </span>
                                )}
                            </div>

                            {latestDns ? (
                                <>
                                    <div className="grid grid-cols-2 gap-2 mt-2 pt-2 border-t border-border text-xs">
                                        <div>
                                            <span className="text-muted-foreground block text-[11px]">IP Resolvedor</span>
                                            <span className="font-mono font-medium">{displayIp(latestDns.target_ip)}</span>
                                        </div>
                                        <div>
                                            <span className="text-muted-foreground block text-[11px]">Latencia RTT</span>
                                            <span className="font-mono font-medium">
                                                {latestDns.rtt_ms != null ? `${latestDns.rtt_ms} ms` : "Inalcanzable"}
                                            </span>
                                        </div>
                                        <div>
                                            <span className="text-muted-foreground block text-[11px]">Jitter RFC 3550</span>
                                            <span className="font-mono font-medium">{latestDns.jitter_ms.toFixed(1)} ms</span>
                                        </div>
                                        <div>
                                            <span className="text-muted-foreground block text-[11px]">Pérdida / Método</span>
                                            <span className="font-mono font-medium">
                                                {latestDns.loss_pct.toFixed(0)}% ({latestDns.method})
                                            </span>
                                        </div>
                                    </div>
                                    {isLoopbackIp(latestDns.target_ip) && (
                                        <p className="mt-2 text-[11px] text-amber-600 dark:text-amber-400 bg-amber-500/10 border border-amber-500/20 rounded p-2">
                                            Muestra previa registrada contra el stub DNS loopback local (127.0.0.X). Pulsa &quot;Sondear Ahora&quot; para medir contra el servidor upstream real.
                                        </p>
                                    )}
                                </>
                            ) : (
                                <p className="text-xs text-muted-foreground italic mt-2">
                                    Aún no hay muestras registradas. Pulsa &quot;Sondear Ahora&quot; para medir.
                                </p>
                            )}
                        </div>
                    </div>

                    {/* Tabla de Muestras de Salud */}
                    {healthSamples.length > 0 && (
                        <div className="flex flex-col gap-2 pt-2 border-t border-border">
                            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                                Registros recientes de salud de enlace
                            </span>
                            <div className="rounded-md border border-border overflow-hidden">
                                <Table className="text-xs text-left">
                                    <TableHeader className="bg-muted/50 text-muted-foreground font-medium border-b border-border">
                                        <TableRow>
                                            <TableHead className="py-2 px-3">Fecha y Hora</TableHead>
                                            <TableHead className="py-2 px-3">Objetivo</TableHead>
                                            <TableHead className="py-2 px-3">IP</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Latencia RTT</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Jitter</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Pérdida</TableHead>
                                            <TableHead className="py-2 px-3 text-right">Método</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody className="divide-y divide-border font-mono">
                                        {healthSamples.slice(0, 8).map((sample) => (
                                            <TableRow key={sample.id} className="hover:bg-muted/20">
                                                <TableCell className="py-2 px-3 text-muted-foreground font-sans">
                                                    {new Date(sample.created_at).toLocaleString()}
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-foreground font-sans capitalize">
                                                    {sample.target_type}
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-foreground">
                                                    <div className="flex items-center gap-1.5">
                                                        <span>{displayIp(sample.target_ip)}</span>
                                                        {isLoopbackIp(sample.target_ip) && (
                                                            <span className="text-[10px] text-amber-600 dark:text-amber-400 font-sans px-1 py-0.5 rounded bg-amber-500/10 border border-amber-500/20">
                                                                stub local
                                                            </span>
                                                        )}
                                                    </div>
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right font-medium">
                                                    {isLoopbackIp(sample.target_ip) ? (
                                                        <span
                                                            className="text-muted-foreground text-[11px] italic"
                                                            title="Medición sobre interfaz loopback (no representativa)"
                                                        >
                                                            0 ms (loopback)
                                                        </span>
                                                    ) : sample.rtt_ms != null ? (
                                                        `${sample.rtt_ms} ms`
                                                    ) : (
                                                        "-"
                                                    )}
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-muted-foreground">
                                                    {sample.jitter_ms.toFixed(1)} ms
                                                </TableCell>
                                                <TableCell
                                                    className={`py-2 px-3 text-right font-medium ${
                                                        sample.loss_pct > 0
                                                            ? "text-red-500"
                                                            : "text-emerald-500"
                                                    }`}
                                                >
                                                    {sample.loss_pct.toFixed(0)}%
                                                </TableCell>
                                                <TableCell className="py-2 px-3 text-right text-muted-foreground uppercase text-[10px]">
                                                    {sample.method}
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}
