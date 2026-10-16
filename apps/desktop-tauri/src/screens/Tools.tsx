import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
    Activity,
    AlertCircle,
    ArrowRight,
    CheckCircle2,
    Compass,
    Globe,
    Loader2,
    Play,
    Power,
    Search,
    Server,
    Square,
    Terminal,
    Wrench,
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
import { Input } from "@/components/ui/input";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useToast } from "@/components/ui/toast";
import { useCensorship } from "@/components/censorship-provider";
import {
    cancelTool,
    checkPort,
    dnsLookup,
    onToolPingPacket,
    onToolPingSummary,
    onToolTraceDone,
    onToolTraceHop,
    sendWol,
    startToolPing,
    startToolTraceroute,
    type DnsRecord,
    type PingPacket,
    type PingStreamSummary,
    type PortCheckResult,
    type TraceHop,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";

type ToolTab = "ping" | "traceroute" | "dns" | "wol" | "ports";

function parseToolTab(val: string | null): ToolTab {
    if (!val) return "ping";
    const v = val.toLowerCase();
    if (v === "ports" || v === "port") return "ports";
    if (v === "traceroute" || v === "trace") return "traceroute";
    if (v === "dns") return "dns";
    if (v === "wol" || v === "wakeonlan") return "wol";
    return "ping";
}

const QUICK_PORTS = [
    { port: 22, name: "SSH" },
    { port: 80, name: "HTTP" },
    { port: 443, name: "HTTPS" },
    { port: 445, name: "SMB" },
    { port: 3389, name: "RDP" },
    { port: 8080, name: "Web-Alt" },
];

const DNS_PROVIDERS = [
    { id: "system", label: "Sistema (Local)" },
    { id: "cloudflare", label: "Cloudflare (1.1.1.1)" },
    { id: "google", label: "Google (8.8.8.8)" },
    { id: "quad9", label: "Quad9 (9.9.9.9)" },
];

const DNS_TYPES = ["ANY", "A", "AAAA", "CNAME", "MX", "TXT", "NS", "PTR", "SOA"];

export function Tools() {
    const { toast } = useToast();
    const { censorshipEnabled } = useCensorship();
    const [searchParams, setSearchParams] = useSearchParams();

    const initialTool = parseToolTab(searchParams.get("tool"));
    const initialIp = searchParams.get("ip") || "";
    const initialMac = searchParams.get("mac") || "";

    const [activeTab, setActiveTab] = useState<ToolTab>(initialTool);

    // --- Ping State ---
    const [pingTarget, setPingTarget] = useState(initialIp);
    const [pinging, setPinging] = useState(false);
    const [pingPackets, setPingPackets] = useState<PingPacket[]>([]);
    const [pingSummary, setPingSummary] = useState<PingStreamSummary | null>(null);
    const pingSessionRef = useRef<string | null>(null);

    // --- Traceroute State ---
    const [traceTarget, setTraceTarget] = useState(initialIp);
    const [maxHops, setMaxHops] = useState(30);
    const [tracing, setTracing] = useState(false);
    const [traceHops, setTraceHops] = useState<TraceHop[]>([]);
    const traceSessionRef = useRef<string | null>(null);

    // --- DNS State ---
    const [dnsTarget, setDnsTarget] = useState(initialIp || "google.com");
    const [dnsType, setDnsType] = useState("A");
    const [dnsProvider, setDnsProvider] = useState("system");
    const [dnsLoading, setDnsLoading] = useState(false);
    const [dnsRecords, setDnsRecords] = useState<DnsRecord[]>([]);

    // --- WoL State ---
    const [wolMac, setWolMac] = useState(initialMac);
    const [wolBroadcast, setWolBroadcast] = useState("");
    const [wolSending, setWolSending] = useState(false);
    const [wolResult, setWolResult] = useState<{ success: boolean; msg: string } | null>(null);

    // --- Port Checker State ---
    const [portTarget, setPortTarget] = useState(initialIp);
    const [portNumber, setPortNumber] = useState<number>(80);
    const [portChecking, setPortChecking] = useState(false);
    const [portResult, setPortResult] = useState<PortCheckResult | null>(null);

    // Actualiza tabs si cambian query params
    useEffect(() => {
        const rawTool = searchParams.get("tool");
        if (rawTool) {
            setActiveTab(parseToolTab(rawTool));
        }
        const ip = searchParams.get("ip");
        if (ip) {
            setPingTarget(ip);
            setTraceTarget(ip);
            setPortTarget(ip);
        }
        const mac = searchParams.get("mac");
        if (mac) {
            setWolMac(mac);
        }
    }, [searchParams]);

    // Suscripción a eventos IPC de herramientas de streaming
    useEffect(() => {
        let unlistens: (() => void)[] = [];

        Promise.all([
            onToolPingPacket((payload) => {
                if (payload.session_id === pingSessionRef.current) {
                    setPingPackets((prev) => [...prev.slice(-49), payload.packet]);
                }
            }),
            onToolPingSummary((payload) => {
                if (payload.session_id === pingSessionRef.current) {
                    setPingSummary(payload.summary);
                    setPinging(false);
                }
            }),
            onToolTraceHop((payload) => {
                if (payload.session_id === traceSessionRef.current) {
                    setTraceHops((prev) => {
                        const exists = prev.some((h) => h.hop_number === payload.hop.hop_number);
                        if (exists) {
                            return prev.map((h) =>
                                h.hop_number === payload.hop.hop_number ? payload.hop : h,
                            );
                        }
                        return [...prev, payload.hop].sort((a, b) => a.hop_number - b.hop_number);
                    });
                }
            }),
            onToolTraceDone((payload) => {
                if (payload.session_id === traceSessionRef.current) {
                    setTraceHops(payload.hops);
                    setTracing(false);
                }
            }),
        ]).then((fns) => {
            unlistens = fns;
        });

        return () => {
            unlistens.forEach((u) => u());
            if (pingSessionRef.current) {
                try {
                    cancelTool(pingSessionRef.current)?.catch?.(() => {});
                } catch {
                    // Ignore
                }
            }
            if (traceSessionRef.current) {
                try {
                    cancelTool(traceSessionRef.current)?.catch?.(() => {});
                } catch {
                    // Ignore
                }
            }
        };
    }, []);

    // Handlers para Ping
    async function handleStartPing() {
        if (!pingTarget.trim()) {
            toast("Ingresa una IP o dominio para hacer ping.", "info");
            return;
        }
        const sessionId = `ping-${Date.now()}`;
        pingSessionRef.current = sessionId;
        setPingPackets([]);
        setPingSummary(null);
        setPinging(true);

        try {
            await startToolPing({
                target: pingTarget.trim(),
                sessionId,
                intervalMs: 1000,
                timeoutMs: 1000,
            });
        } catch (e) {
            setPinging(false);
            toast(`Error al iniciar ping: ${e}`, "error");
        }
    }

    async function handleStopPing() {
        if (pingSessionRef.current) {
            try {
                await cancelTool(pingSessionRef.current);
            } catch {
                // Ignore
            }
            setPinging(false);
        }
    }

    // Handlers para Traceroute
    async function handleStartTraceroute() {
        if (!traceTarget.trim()) {
            toast("Ingresa una IP o dominio para traceroute.", "info");
            return;
        }
        const sessionId = `trace-${Date.now()}`;
        traceSessionRef.current = sessionId;
        setTraceHops([]);
        setTracing(true);

        try {
            await startToolTraceroute({
                target: traceTarget.trim(),
                sessionId,
                maxHops,
                timeoutMs: 1000,
            });
        } catch (e) {
            setTracing(false);
            toast(`Error al iniciar traceroute: ${e}`, "error");
        }
    }

    async function handleStopTraceroute() {
        if (traceSessionRef.current) {
            try {
                await cancelTool(traceSessionRef.current);
            } catch {
                // Ignore
            }
            setTracing(false);
        }
    }

    // Handlers para DNS
    async function handleDnsLookup() {
        if (!dnsTarget.trim()) {
            toast("Ingresa un dominio o IP para consultar.", "info");
            return;
        }
        setDnsLoading(true);
        try {
            const records = await dnsLookup(
                dnsTarget.trim(),
                dnsType,
                dnsProvider === "system" ? undefined : dnsProvider,
            );
            setDnsRecords(records);
            if (records.length === 0) {
                toast("No se encontraron registros para la consulta.", "info");
            }
        } catch (e) {
            toast(`Error en consulta DNS: ${e}`, "error");
            setDnsRecords([]);
        } finally {
            setDnsLoading(false);
        }
    }

    // Handlers para WoL
    async function handleSendWol() {
        if (!wolMac.trim()) {
            toast("Ingresa la dirección MAC de destino.", "info");
            return;
        }
        setWolSending(true);
        setWolResult(null);
        try {
            await sendWol(wolMac.trim(), wolBroadcast.trim() || undefined);
            setWolResult({
                success: true,
                msg: `Magic Packet enviado con éxito hacia ${wolMac}.`,
            });
            toast("Magic Packet de Wake-on-LAN enviado.", "success");
        } catch (e) {
            setWolResult({
                success: false,
                msg: `Fallo al enviar paquete: ${e}`,
            });
            toast(`Error WoL: ${e}`, "error");
        } finally {
            setWolSending(false);
        }
    }

    // Handlers para Port Checker
    async function handleCheckPort() {
        if (!portTarget.trim() || !portNumber) {
            toast("Ingresa el host y el puerto a comprobar.", "info");
            return;
        }
        setPortChecking(true);
        setPortResult(null);
        try {
            const res = await checkPort(portTarget.trim(), portNumber, 2000);
            setPortResult(res);
        } catch (e) {
            toast(`Error comprobando puerto: ${e}`, "error");
        } finally {
            setPortChecking(false);
        }
    }

    // Helper de enmascaramiento si modo censura está activo
    function maskDisplay(val: string | null): string {
        if (!val) return "-";
        if (!censorshipEnabled) return val;
        // Si parece IP
        if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(val)) {
            const parts = val.split(".");
            return `${parts[0]}.${parts[1]}.*.*`;
        }
        // Si parece MAC
        if (/^([0-9a-fA-F]{2}[:-]){5}[0-9a-fA-F]{2}$/.test(val)) {
            return `${val.substring(0, 8)}:**:**:**`;
        }
        return val;
    }

    return (
        <div className={cn("flex flex-col", SECTION_GAP)}>
            {/* Cabecera */}
            <div className="flex flex-col gap-0.5">
                <h2 className="text-xl font-bold tracking-tight flex items-center gap-2">
                    <Wrench className="h-5 w-5 text-primary" aria-hidden />
                    Herramientas de red
                </h2>
                <p className="text-sm text-muted-foreground">
                    Utilidades interactivas de diagnóstico rápido, sondeo y control para tu red local e Internet.
                </p>
            </div>

            {/* Pestañas de herramientas */}
            <div className="flex flex-wrap items-center gap-1.5 border-b border-border pb-2">
                <Button
                    type="button"
                    variant={activeTab === "ping" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("ping");
                        setSearchParams({ tool: "ping" });
                    }}
                    className="gap-1.5 text-xs h-8"
                >
                    <Activity className="h-3.5 w-3.5" aria-hidden />
                    Ping Interactivo
                </Button>
                <Button
                    type="button"
                    variant={activeTab === "traceroute" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("traceroute");
                        setSearchParams({ tool: "traceroute" });
                    }}
                    className="gap-1.5 text-xs h-8"
                >
                    <Compass className="h-3.5 w-3.5" aria-hidden />
                    Traceroute Visual
                </Button>
                <Button
                    type="button"
                    variant={activeTab === "dns" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("dns");
                        setSearchParams({ tool: "dns" });
                    }}
                    className="gap-1.5 text-xs h-8"
                >
                    <Globe className="h-3.5 w-3.5" aria-hidden />
                    DNS Lookup
                </Button>
                <Button
                    type="button"
                    variant={activeTab === "ports" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("ports");
                        setSearchParams({ tool: "ports" });
                    }}
                    className="gap-1.5 text-xs h-8"
                >
                    <Server className="h-3.5 w-3.5" aria-hidden />
                    Port Checker
                </Button>
                <Button
                    type="button"
                    variant={activeTab === "wol" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("wol");
                        setSearchParams({ tool: "wol" });
                    }}
                    className="gap-1.5 text-xs h-8"
                >
                    <Power className="h-3.5 w-3.5" aria-hidden />
                    Wake-on-LAN
                </Button>
            </div>

            {/* Contenido según herramienta activa */}
            {activeTab === "ping" && (
                <div className="flex flex-col gap-4">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Activity className="h-4 w-4 text-primary" aria-hidden />
                                Sondeo Ping en tiempo real
                            </CardTitle>
                            <CardDescription>
                                Envía paquetes continuos para evaluar la latencia, variación de jitter (RFC 3550) y estabilidad del enlace.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2 flex flex-col gap-4">
                            <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                                <Input
                                    placeholder="IP o dominio (ej. 192.168.1.1 o cloudflare.com)"
                                    value={pingTarget}
                                    onChange={(e) => setPingTarget(e.target.value)}
                                    disabled={pinging}
                                    className="font-mono text-xs flex-1"
                                />
                                {pinging ? (
                                    <Button
                                        type="button"
                                        variant="destructive"
                                        size="sm"
                                        onClick={handleStopPing}
                                        className="gap-1.5"
                                    >
                                        <Square className="h-3.5 w-3.5" aria-hidden />
                                        Detener
                                    </Button>
                                ) : (
                                    <Button
                                        type="button"
                                        variant="default"
                                        size="sm"
                                        onClick={handleStartPing}
                                        className="gap-1.5"
                                    >
                                        <Play className="h-3.5 w-3.5" aria-hidden />
                                        Iniciar Ping
                                    </Button>
                                )}
                            </div>

                            {/* Tarjetas de estadísticas */}
                            {pingPackets.length > 0 && (
                                <div className="grid grid-cols-2 gap-2 sm:grid-cols-5">
                                    <div className="rounded-lg border bg-background/50 p-3 text-center">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
                                            Transmitidos
                                        </span>
                                        <div className="text-lg font-bold font-mono">
                                            {pingSummary ? pingSummary.transmitted : pingPackets.length}
                                        </div>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-3 text-center">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
                                            Recibidos
                                        </span>
                                        <div className="text-lg font-bold font-mono text-emerald-600 dark:text-emerald-400">
                                            {pingSummary
                                                ? pingSummary.received
                                                : pingPackets.filter((p) => p.reachable).length}
                                        </div>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-3 text-center">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
                                            Pérdida
                                        </span>
                                        <div className="text-lg font-bold font-mono">
                                            {pingSummary
                                                ? `${pingSummary.packet_loss_percent.toFixed(1)}%`
                                                : `${(
                                                      ((pingPackets.filter((p) => !p.reachable).length) /
                                                          pingPackets.length) *
                                                      100
                                                  ).toFixed(0)}%`}
                                        </div>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-3 text-center">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
                                            Último RTT
                                        </span>
                                        <div className="text-lg font-bold font-mono text-primary">
                                            {pingPackets[pingPackets.length - 1]?.latency_ms != null
                                                ? `${pingPackets[pingPackets.length - 1].latency_ms} ms`
                                                : "-"}
                                        </div>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-3 text-center">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
                                            Jitter RFC 3550
                                        </span>
                                        <div className="text-lg font-bold font-mono text-amber-600 dark:text-amber-400">
                                            {pingPackets[pingPackets.length - 1]?.jitter_ms != null
                                                ? `${pingPackets[pingPackets.length - 1].jitter_ms.toFixed(1)} ms`
                                                : "-"}
                                        </div>
                                    </div>
                                </div>
                            )}

                            {/* Stream de paquetes */}
                            {pingPackets.length > 0 && (
                                <div className="rounded-lg border bg-muted/20 p-3 font-mono text-xs flex flex-col gap-1.5 max-h-64 overflow-y-auto">
                                    {pingPackets.slice().reverse().map((pkt) => (
                                        <div
                                            key={pkt.seq}
                                            className="flex items-center justify-between border-b border-border/40 pb-1 last:border-0"
                                        >
                                            <div className="flex items-center gap-2">
                                                <span className="text-muted-foreground">#{pkt.seq}</span>
                                                {pkt.reachable ? (
                                                    <span className="text-emerald-600 dark:text-emerald-400 font-semibold">
                                                        Respuesta de {maskDisplay(pkt.target)}
                                                    </span>
                                                ) : (
                                                    <span className="text-destructive font-semibold">
                                                        Tiempo de espera agotado
                                                    </span>
                                                )}
                                                <span className="text-[10px] text-muted-foreground/80">
                                                    ({pkt.method})
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-3">
                                                {pkt.latency_ms != null && (
                                                    <span className="text-primary font-bold">
                                                        tiempo={pkt.latency_ms} ms
                                                    </span>
                                                )}
                                                <span className="text-muted-foreground text-[11px]">
                                                    jitter={pkt.jitter_ms.toFixed(1)}ms
                                                </span>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            )}

            {activeTab === "traceroute" && (
                <div className="flex flex-col gap-4">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Compass className="h-4 w-4 text-primary" aria-hidden />
                                Trazado de ruta (Traceroute)
                            </CardTitle>
                            <CardDescription>
                                Identifica cada router y salto intermedio en el camino hacia el destino con análisis de latencia acumulada.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2 flex flex-col gap-4">
                            <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                                <Input
                                    placeholder="IP o dominio (ej. 1.1.1.1 o debian.org)"
                                    value={traceTarget}
                                    onChange={(e) => setTraceTarget(e.target.value)}
                                    disabled={tracing}
                                    className="font-mono text-xs flex-1"
                                />
                                <div className="flex items-center gap-2">
                                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                        <span>Saltos máx:</span>
                                        <Input
                                            type="number"
                                            min={1}
                                            max={64}
                                            value={maxHops}
                                            onChange={(e) => setMaxHops(Number(e.target.value) || 30)}
                                            disabled={tracing}
                                            className="w-16 h-8 font-mono text-xs"
                                        />
                                    </div>
                                    {tracing ? (
                                        <Button
                                            type="button"
                                            variant="destructive"
                                            size="sm"
                                            onClick={handleStopTraceroute}
                                            className="gap-1.5"
                                        >
                                            <Square className="h-3.5 w-3.5" aria-hidden />
                                            Detener
                                        </Button>
                                    ) : (
                                        <Button
                                            type="button"
                                            variant="default"
                                            size="sm"
                                            onClick={handleStartTraceroute}
                                            className="gap-1.5"
                                        >
                                            <Play className="h-3.5 w-3.5" aria-hidden />
                                            Trazar ruta
                                        </Button>
                                    )}
                                </div>
                            </div>

                            {/* Escalera de saltos */}
                            {traceHops.length > 0 && (
                                <div className="rounded-lg border bg-background/50 overflow-hidden">
                                    <table className="w-full text-xs font-mono">
                                        <thead className="bg-muted/40 border-b border-border text-[11px] text-muted-foreground uppercase">
                                            <tr>
                                                <th className="p-2.5 text-left w-12">#</th>
                                                <th className="p-2.5 text-left">Dirección IP</th>
                                                <th className="p-2.5 text-left">Hostname resuelto</th>
                                                <th className="p-2.5 text-left">Estado</th>
                                                <th className="p-2.5 text-right">Latencia RTT</th>
                                            </tr>
                                        </thead>
                                        <tbody className="divide-y divide-border/40">
                                            {traceHops.map((hop) => {
                                                const lat = hop.latency_ms;
                                                const latBadgeClass =
                                                    lat == null
                                                        ? "text-muted-foreground"
                                                        : lat < 30
                                                        ? "text-emerald-600 dark:text-emerald-400 font-bold"
                                                        : lat < 100
                                                        ? "text-amber-600 dark:text-amber-400 font-bold"
                                                        : "text-rose-600 dark:text-rose-400 font-bold";

                                                return (
                                                    <tr key={hop.hop_number} className="hover:bg-muted/30">
                                                        <td className="p-2.5 text-muted-foreground font-semibold">
                                                            {hop.hop_number}
                                                        </td>
                                                        <td className="p-2.5">
                                                            {hop.ip ? (
                                                                <span className="font-semibold text-foreground">
                                                                    {maskDisplay(hop.ip)}
                                                                </span>
                                                            ) : (
                                                                <span className="text-muted-foreground">*</span>
                                                            )}
                                                        </td>
                                                        <td className="p-2.5 text-muted-foreground truncate max-w-[200px]">
                                                            {hop.hostname ? maskDisplay(hop.hostname) : "-"}
                                                        </td>
                                                        <td className="p-2.5">
                                                            <span
                                                                className={cn(
                                                                    "inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium",
                                                                    hop.state === "reached"
                                                                        ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                                                                        : hop.state === "*"
                                                                        ? "bg-muted text-muted-foreground"
                                                                        : "bg-primary/10 text-primary border border-primary/20",
                                                                )}
                                                            >
                                                                {hop.state}
                                                            </span>
                                                        </td>
                                                        <td className={cn("p-2.5 text-right", latBadgeClass)}>
                                                            {lat != null ? `${lat} ms` : "*"}
                                                        </td>
                                                    </tr>
                                                );
                                            })}
                                        </tbody>
                                    </table>
                                </div>
                            )}

                            {tracing && (
                                <div className="flex items-center gap-2 text-xs text-muted-foreground pt-1">
                                    <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                    Descubriendo saltos de red…
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            )}

            {activeTab === "dns" && (
                <div className="flex flex-col gap-4">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Globe className="h-4 w-4 text-primary" aria-hidden />
                                Consulta DNS Multidominio
                            </CardTitle>
                            <CardDescription>
                                Resuelve registros A, AAAA, CNAME, MX, TXT, NS, PTR y SOA alternando entre tu DNS local y proveedores públicos.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2 flex flex-col gap-4">
                            <div className="grid gap-3 sm:grid-cols-12">
                                <div className="sm:col-span-5">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Dominio o IP
                                    </label>
                                    <Input
                                        placeholder="ej. github.com o 1.1.1.1"
                                        value={dnsTarget}
                                        onChange={(e) => setDnsTarget(e.target.value)}
                                        className="font-mono text-xs"
                                    />
                                </div>
                                <div className="sm:col-span-3">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Tipo de registro
                                    </label>
                                    <select
                                        value={dnsType}
                                        onChange={(e) => setDnsType(e.target.value)}
                                        className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs font-mono shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                                    >
                                        {DNS_TYPES.map((t) => (
                                            <option key={t} value={t}>
                                                {t}
                                            </option>
                                        ))}
                                    </select>
                                </div>
                                <div className="sm:col-span-4">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Servidor DNS
                                    </label>
                                    <select
                                        value={dnsProvider}
                                        onChange={(e) => setDnsProvider(e.target.value)}
                                        className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                                    >
                                        {DNS_PROVIDERS.map((p) => (
                                            <option key={p.id} value={p.id}>
                                                {p.label}
                                            </option>
                                        ))}
                                    </select>
                                </div>
                            </div>

                            <div className="flex justify-end">
                                <Button
                                    type="button"
                                    variant="default"
                                    size="sm"
                                    disabled={dnsLoading}
                                    onClick={handleDnsLookup}
                                    className="gap-1.5"
                                >
                                    {dnsLoading ? (
                                        <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                    ) : (
                                        <Search className="h-3.5 w-3.5" aria-hidden />
                                    )}
                                    Consultar DNS
                                </Button>
                            </div>

                            {/* Resultados de la consulta */}
                            {dnsRecords.length > 0 && (
                                <div className="rounded-lg border bg-background/50 overflow-hidden">
                                    <table className="w-full text-xs font-mono">
                                        <thead className="bg-muted/40 border-b border-border text-[11px] text-muted-foreground uppercase">
                                            <tr>
                                                <th className="p-2.5 text-left">Nombre</th>
                                                <th className="p-2.5 text-left w-20">Tipo</th>
                                                <th className="p-2.5 text-left">Valor resuelto</th>
                                                <th className="p-2.5 text-right w-20">TTL</th>
                                            </tr>
                                        </thead>
                                        <tbody className="divide-y divide-border/40">
                                            {dnsRecords.map((r, idx) => (
                                                <tr key={`${r.record_type}-${r.value}-${idx}`} className="hover:bg-muted/30">
                                                    <td className="p-2.5 font-medium">{r.name}</td>
                                                    <td className="p-2.5">
                                                        <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-semibold bg-primary/10 text-primary border border-primary/20">
                                                            {r.record_type}
                                                        </span>
                                                    </td>
                                                    <td className="p-2.5 font-semibold text-foreground break-all">
                                                        {maskDisplay(r.value)}
                                                    </td>
                                                    <td className="p-2.5 text-right text-muted-foreground">
                                                        {r.ttl}s
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            )}

            {activeTab === "ports" && (
                <div className="flex flex-col gap-4">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Server className="h-4 w-4 text-primary" aria-hidden />
                                Comprobación rápida de puerto (Port Checker)
                            </CardTitle>
                            <CardDescription>
                                Verifica si un servicio o puerto TCP específico está abierto, cerrado o bloqueado por firewall en un host puntual.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2 flex flex-col gap-4">
                            <div className="grid gap-3 sm:grid-cols-12 items-end">
                                <div className="sm:col-span-8">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Host o Dirección IP
                                    </label>
                                    <Input
                                        placeholder="ej. 192.168.1.1 o mi-servidor.lan"
                                        value={portTarget}
                                        onChange={(e) => setPortTarget(e.target.value)}
                                        className="font-mono text-xs"
                                    />
                                </div>
                                <div className="sm:col-span-4">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Puerto TCP
                                    </label>
                                    <Input
                                        type="number"
                                        min={1}
                                        max={65535}
                                        value={portNumber}
                                        onChange={(e) => setPortNumber(Number(e.target.value))}
                                        className="font-mono text-xs"
                                    />
                                </div>
                            </div>

                            {/* Accesos rápidos a puertos comunes */}
                            <div className="flex flex-wrap items-center gap-1.5 pt-1">
                                <span className="text-[11px] text-muted-foreground mr-1">Comunes:</span>
                                {QUICK_PORTS.map((qp) => (
                                    <Button
                                        key={qp.port}
                                        type="button"
                                        variant="outline"
                                        size="sm"
                                        onClick={() => setPortNumber(qp.port)}
                                        className="h-6 text-[11px] font-mono px-2"
                                    >
                                        {qp.port} ({qp.name})
                                    </Button>
                                ))}
                            </div>

                            <div className="flex justify-end">
                                <Button
                                    type="button"
                                    variant="default"
                                    size="sm"
                                    disabled={portChecking}
                                    onClick={handleCheckPort}
                                    className="gap-1.5"
                                >
                                    {portChecking ? (
                                        <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                    ) : (
                                        <Zap className="h-3.5 w-3.5" aria-hidden />
                                    )}
                                    Comprobar puerto
                                </Button>
                            </div>

                            {/* Resultado del puerto */}
                            {portResult && (
                                <div className="rounded-lg border bg-background/50 p-4 flex flex-col gap-2">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className="font-semibold text-sm">
                                                {maskDisplay(portResult.target)}:{portResult.port}
                                            </span>
                                            {portResult.state === "open" ? (
                                                <span className="inline-flex items-center gap-1 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20 px-2 py-0.5 text-xs font-bold">
                                                    <CheckCircle2 className="h-3 w-3" />
                                                    Abierto
                                                </span>
                                            ) : portResult.state === "closed" ? (
                                                <span className="inline-flex items-center gap-1 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 px-2 py-0.5 text-xs font-semibold">
                                                    Cerrado
                                                </span>
                                            ) : (
                                                <span className="inline-flex items-center gap-1 rounded bg-muted text-muted-foreground border border-border px-2 py-0.5 text-xs font-semibold">
                                                    Filtrado / Timeout
                                                </span>
                                            )}
                                        </div>
                                        {portResult.latency_ms != null && (
                                            <span className="font-mono text-xs text-primary font-bold">
                                                {portResult.latency_ms} ms RTT
                                            </span>
                                        )}
                                    </div>
                                    {portResult.banner && (
                                        <div className="text-xs bg-muted/30 rounded border p-2 font-mono break-all text-muted-foreground">
                                            Banner: {portResult.banner}
                                        </div>
                                    )}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            )}

            {activeTab === "wol" && (
                <div className="flex flex-col gap-4">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Power className="h-4 w-4 text-primary" aria-hidden />
                                Encendido remoto Wake-on-LAN (WoL)
                            </CardTitle>
                            <CardDescription>
                                Envía un paquete mágico estándar a través de broadcast UDP (puerto 9) para encender equipos compatibles en tu red local.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2 flex flex-col gap-4">
                            <div className="grid gap-3 sm:grid-cols-12">
                                <div className="sm:col-span-8">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        Dirección MAC destino
                                    </label>
                                    <Input
                                        placeholder="ej. AA:BB:CC:DD:EE:FF"
                                        value={wolMac}
                                        onChange={(e) => setWolMac(e.target.value)}
                                        className="font-mono text-xs"
                                    />
                                </div>
                                <div className="sm:col-span-4">
                                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                                        IP Broadcast (Opcional)
                                    </label>
                                    <Input
                                        placeholder="255.255.255.255"
                                        value={wolBroadcast}
                                        onChange={(e) => setWolBroadcast(e.target.value)}
                                        className="font-mono text-xs"
                                    />
                                </div>
                            </div>

                            <div className="flex justify-end">
                                <Button
                                    type="button"
                                    variant="default"
                                    size="sm"
                                    disabled={wolSending}
                                    onClick={handleSendWol}
                                    className="gap-1.5"
                                >
                                    {wolSending ? (
                                        <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                    ) : (
                                        <Power className="h-3.5 w-3.5" aria-hidden />
                                    )}
                                    Enviar Magic Packet
                                </Button>
                            </div>

                            {wolResult && (
                                <Alert variant={wolResult.success ? "default" : "destructive"}>
                                    {wolResult.success ? (
                                        <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                                    ) : (
                                        <AlertCircle className="h-4 w-4" />
                                    )}
                                    <AlertTitle>
                                        {wolResult.success ? "Paquete enviado" : "Error en el envío"}
                                    </AlertTitle>
                                    <AlertDescription className="text-xs font-mono">
                                        {wolResult.msg}
                                    </AlertDescription>
                                </Alert>
                            )}
                        </CardContent>
                    </Card>
                </div>
            )}
        </div>
    );
}
