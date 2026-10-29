import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
    Activity,
    AlertCircle,
    ArrowRight,
    Check,
    CheckCircle2,
    Compass,
    Copy,
    Globe,
    Loader2,
    Play,
    Power,
    RotateCcw,
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
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
import { SpeedTestPanel } from "@/components/speed-test-panel";

type ToolTab = "ping" | "traceroute" | "dns" | "wol" | "ports" | "speed";

function parseToolTab(val: string | null): ToolTab {
    if (!val) return "ping";
    const v = val.toLowerCase();
    if (v === "speed" || v === "speedtest" || v === "health") return "speed";
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
    { port: 53, name: "DNS" },
    { port: 5432, name: "Postgres" },
];

const DNS_PROVIDERS = [
    { id: "system", label: "Sistema (Local)" },
    { id: "cloudflare", label: "Cloudflare (1.1.1.1)" },
    { id: "google", label: "Google (8.8.8.8)" },
    { id: "quad9", label: "Quad9 (9.9.9.9)" },
];

const DNS_TYPES = ["ANY", "A", "AAAA", "CNAME", "MX", "TXT", "NS", "PTR", "SOA"];

const COMMON_PING_TARGETS = [
    { label: "Router (192.168.1.1)", target: "192.168.1.1" },
    { label: "Cloudflare (1.1.1.1)", target: "1.1.1.1" },
    { label: "Google DNS (8.8.8.8)", target: "8.8.8.8" },
    { label: "Localhost (127.0.0.1)", target: "127.0.0.1" },
];

const COMMON_TRACE_TARGETS = [
    { label: "Cloudflare (1.1.1.1)", target: "1.1.1.1" },
    { label: "Google (8.8.8.8)", target: "8.8.8.8" },
    { label: "debian.org", target: "debian.org" },
    { label: "github.com", target: "github.com" },
];

const COMMON_DNS_TARGETS = [
    "cloudflare.com",
    "google.com",
    "github.com",
    "wikipedia.org",
];

// Indicador Heartbeat de estado activo con espacio reservado fijo (evita layout shift en tabs)
function ToolTabHeartbeat({
    active,
    color = "bg-primary",
}: {
    active: boolean;
    color?: string;
}) {
    return (
        <span
            className={cn(
                "relative flex h-2 w-2 items-center justify-center shrink-0 transition-opacity duration-150",
                active ? "opacity-100" : "opacity-0 pointer-events-none",
            )}
            aria-hidden="true"
        >
            <span
                className={cn(
                    "absolute inline-flex h-full w-full animate-ping rounded-full opacity-75 motion-reduce:animate-none",
                    color,
                )}
            />
            <span className={cn("relative inline-flex h-2 w-2 rounded-full", color)} />
        </span>
    );
}

// Componente de osciloscopio visual de latencia RTT
function LatencyOscilloscope({ packets }: { packets: PingPacket[] }) {
    if (packets.length === 0) return null;
    const recent = packets.slice(-40);
    const validLatencies = recent
        .map((p) => p.latency_ms)
        .filter((l): l is number => l != null && l > 0);
    const maxLat = Math.max(30, ...validLatencies, 10);
    const avgLat =
        validLatencies.length > 0
            ? validLatencies.reduce((a, b) => a + b, 0) / validLatencies.length
            : 0;

    return (
        <div className="rounded-lg border bg-background/60 p-3.5 flex flex-col gap-2.5">
            <div className="flex items-center justify-between text-xs">
                <span className="font-semibold text-foreground flex items-center gap-1.5">
                    <Activity className="h-3.5 w-3.5 text-primary" aria-hidden />
                    Osciloscopio de Latencia RTT ({recent.length} muestras)
                </span>
                <div className="flex items-center gap-3 text-[11px] font-mono text-muted-foreground">
                    <span>
                        Prom: <span className="font-semibold text-foreground">{avgLat.toFixed(1)}</span>ms
                    </span>
                    <span>
                        Pico: <span className="font-semibold text-foreground">{maxLat.toFixed(1)}</span>ms
                    </span>
                </div>
            </div>
            <div className="h-20 w-full relative flex items-end gap-1 pt-4 pb-1 px-1 bg-muted/20 rounded border border-border overflow-hidden">
                {/* Línea horizontal de promedio */}
                {avgLat > 0 && (
                    <div
                        className="absolute left-0 right-0 border-b border-dashed border-primary/40 pointer-events-none z-10"
                        style={{
                            bottom: `${Math.min(95, Math.max(5, (avgLat / maxLat) * 100))}%`,
                        }}
                    />
                )}
                {recent.map((pkt, idx) => {
                    const lat = pkt.latency_ms ?? 0;
                    const pct = pkt.reachable
                        ? Math.min(100, Math.max(8, (lat / maxLat) * 100))
                        : 100;
                    const barColor = !pkt.reachable
                        ? "bg-rose-500/80"
                        : lat < 25
                        ? "bg-emerald-500 hover:bg-emerald-400"
                        : lat < 75
                        ? "bg-primary hover:bg-cyan-300"
                        : lat < 150
                        ? "bg-amber-500 hover:bg-amber-400"
                        : "bg-rose-500 hover:bg-rose-400";

                    return (
                        <div
                            key={pkt.seq || idx}
                            className="flex-1 flex flex-col items-center justify-end h-full group relative"
                        >
                            <div
                                style={{ height: `${pct}%` }}
                                className={cn(
                                    "w-full rounded-t-xs transition-all duration-150 min-h-[4px]",
                                    barColor,
                                )}
                            />
                            {/* Tooltip on hover */}
                            <div className="absolute -top-7 hidden group-hover:flex items-center gap-1 px-1.5 py-0.5 rounded bg-popover text-popover-foreground text-[11px] font-mono shadow-md border border-border z-20 whitespace-nowrap pointer-events-none">
                                #{pkt.seq}: {pkt.reachable ? `${lat}ms` : "Pérdida"}
                            </div>
                        </div>
                    );
                })}
            </div>
            <div className="flex items-center justify-between text-[11px] text-muted-foreground font-mono px-0.5">
                <span className="flex items-center gap-1">
                    <span className="h-2 w-2 rounded-full bg-emerald-500" /> &lt;25ms
                </span>
                <span className="flex items-center gap-1">
                    <span className="h-2 w-2 rounded-full bg-primary" /> 25-75ms
                </span>
                <span className="flex items-center gap-1">
                    <span className="h-2 w-2 rounded-full bg-amber-500" /> 75-150ms
                </span>
                <span className="flex items-center gap-1">
                    <span className="h-2 w-2 rounded-full bg-rose-500" /> &gt;150ms / Pérdida
                </span>
            </div>
        </div>
    );
}

export function Tools() {
    const { toast } = useToast();
    const { censorshipEnabled } = useCensorship();
    const [searchParams, setSearchParams] = useSearchParams();

    const initialTool = parseToolTab(searchParams.get("tool"));
    const initialIp = searchParams.get("ip") || "";
    const initialMac = searchParams.get("mac") || "";

    const [activeTab, setActiveTab] = useState<ToolTab>(initialTool);
    const [lanDevices] = useState<
        { ip: string; mac?: string; hostname?: string; name?: string }[]
    >([]);

    // --- Ping State ---
    const [pingTarget, setPingTarget] = useState(initialIp);
    const [pingCount, setPingCount] = useState<number | null>(10);
    const [pinging, setPinging] = useState(false);
    const [pingPackets, setPingPackets] = useState<PingPacket[]>([]);
    const [pingSummary, setPingSummary] = useState<PingStreamSummary | null>(null);
    const [copiedPing, setCopiedPing] = useState(false);
    const [pingError, setPingError] = useState<string | null>(null);
    const pingSessionRef = useRef<string | null>(null);

    // --- Traceroute State ---
    const [traceTarget, setTraceTarget] = useState(initialIp);
    const [maxHops, setMaxHops] = useState(30);
    const [tracing, setTracing] = useState(false);
    const [traceHops, setTraceHops] = useState<TraceHop[]>([]);
    const [traceError, setTraceError] = useState<string | null>(null);
    const traceSessionRef = useRef<string | null>(null);

    // --- DNS State ---
    const [dnsTarget, setDnsTarget] = useState(initialIp || "google.com");
    const [dnsType, setDnsType] = useState("A");
    const [dnsProvider, setDnsProvider] = useState("system");
    const [dnsLoading, setDnsLoading] = useState(false);
    const [dnsRecords, setDnsRecords] = useState<DnsRecord[]>([]);
    const [copiedDns, setCopiedDns] = useState(false);

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
        // Patrón disposed: la Promise de listen() resuelve tras el round-trip
        // IPC, DESPUÉS del primer cleanup de StrictMode. Sin este flag, el
        // unlisten llega tarde y queda registrado (listener zombie que
        // duplica cada paquete en el stream). Si la Promise resuelve tras el
        // cleanup, desregistramos inmediatamente.
        let disposed = false;
        let unlistens: (() => void)[] = [];

        Promise.all([
            onToolPingPacket((payload) => {
                if (pingSessionRef.current && payload.session_id === pingSessionRef.current) {
                    setPingPackets((prev) => [...prev.slice(-49), payload.packet]);
                }
            }),
            onToolPingSummary((payload) => {
                if (pingSessionRef.current && payload.session_id === pingSessionRef.current) {
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
            if (disposed) {
                fns.forEach((u) => u());
                return;
            }
            unlistens = fns;
        });

        return () => {
            disposed = true;
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
            setPingError("Ingresa una IP o dominio para hacer ping.");
            toast("Ingresa una IP o dominio para hacer ping.", "info");
            return;
        }
        setPingError(null);
        const sessionId = `ping-${Date.now()}`;
        pingSessionRef.current = sessionId;
        setPingPackets([]);
        setPingSummary(null);
        setPinging(true);

        try {
            await startToolPing({
                target: pingTarget.trim(),
                sessionId,
                count: pingCount,
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

    function handleClearPing() {
        if (pingSessionRef.current) {
            try {
                cancelTool(pingSessionRef.current)?.catch?.(() => {});
            } catch {
                // Ignore
            }
            pingSessionRef.current = null;
        }
        setPinging(false);
        setPingPackets([]);
        setPingSummary(null);
    }

    function handleCopyPingStream() {
        if (pingPackets.length === 0) return;
        const text = pingPackets
            .map((p) =>
                p.reachable
                    ? `seq=${p.seq} target=${p.target} latency=${p.latency_ms}ms jitter=${p.jitter_ms.toFixed(1)}ms (${p.method})`
                    : `seq=${p.seq} target=${p.target} TIMEOUT (${p.method})`,
            )
            .join("\n");
        navigator.clipboard.writeText(text);
        setCopiedPing(true);
        setTimeout(() => setCopiedPing(false), 2000);
        toast("Historial de ping copiado al portapapeles.", "success");
    }

    // Handlers para Traceroute
    async function handleStartTraceroute() {
        if (!traceTarget.trim()) {
            setTraceError("Ingresa una IP o dominio para traceroute.");
            toast("Ingresa una IP o dominio para traceroute.", "info");
            return;
        }
        setTraceError(null);
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

    function handleCopyDnsRecords() {
        if (dnsRecords.length === 0) return;
        const text = dnsRecords
            .map((r) => `${r.name}\t${r.ttl}s\tIN\t${r.record_type}\t${r.value}`)
            .join("\n");
        navigator.clipboard.writeText(text);
        setCopiedDns(true);
        setTimeout(() => setCopiedDns(false), 2000);
        toast("Registros DNS copiados al portapapeles.", "success");
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
        if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(val)) {
            const parts = val.split(".");
            return `${parts[0]}.${parts[1]}.*.*`;
        }
        if (/^([0-9a-fA-F]{2}[:-]){5}[0-9a-fA-F]{2}$/.test(val)) {
            return `${val.substring(0, 8)}:**:**:**`;
        }
        return val;
    }

    return (
        <div className={cn("flex flex-col", SECTION_GAP)}>
            {/* Cabecera Técnica de Consola */}
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 pb-1 border-b border-border">
                <div className="flex flex-col gap-0.5">
                    <h1 className="text-xl font-bold tracking-tight flex items-center gap-2">
                        <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20">
                            <Wrench className="h-4 w-4" aria-hidden />
                        </span>
                        Herramientas de red
                    </h1>
                    <p className="text-xs text-muted-foreground">
                        Utilidades interactivas de diagnóstico rápido, sondeo y control para tu red local e Internet.
                    </p>
                </div>

                {/* Status chip del motor */}
                <div className="flex items-center gap-2">
                    {pinging ? (
                        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-primary/10 border border-primary/20 text-xs font-mono text-primary animate-pulse">
                            <span className="h-2 w-2 rounded-full bg-primary" />
                            Sondeo Ping activo ({maskDisplay(pingTarget)})
                        </div>
                    ) : tracing ? (
                        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-xs font-mono text-indigo-400 animate-pulse">
                            <span className="h-2 w-2 rounded-full bg-indigo-500" />
                            Trazando ruta ({maskDisplay(traceTarget)})
                        </div>
                    ) : (
                        <div className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-muted/40 border border-border text-[11px] text-muted-foreground font-mono">
                            <Terminal className="h-3 w-3" aria-hidden />
                            Motor de Diagnóstico Listo
                        </div>
                    )}
                </div>
            </div>

            {/* Selector de Herramientas / Workbench Tab Strip */}
            <nav
                aria-label="Pestañas de herramientas"
                className="flex items-center gap-1.5 p-1 rounded-xl bg-muted/30 border border-border overflow-x-auto"
            >
                <Button
                    type="button"
                    variant={activeTab === "ping" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("ping");
                        setSearchParams({ tool: "ping" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "ping"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Activity className={cn("h-3.5 w-3.5", activeTab === "ping" ? "text-primary" : "")} aria-hidden />
                    <span>Ping Interactivo</span>
                    <ToolTabHeartbeat active={pinging} color="bg-primary" />
                </Button>

                <Button
                    type="button"
                    variant={activeTab === "traceroute" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("traceroute");
                        setSearchParams({ tool: "traceroute" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "traceroute"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Compass className={cn("h-3.5 w-3.5", activeTab === "traceroute" ? "text-indigo-400" : "")} aria-hidden />
                    <span>Traceroute Visual</span>
                    <ToolTabHeartbeat active={tracing} color="bg-indigo-500" />
                </Button>

                <Button
                    type="button"
                    variant={activeTab === "dns" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("dns");
                        setSearchParams({ tool: "dns" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "dns"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Globe className={cn("h-3.5 w-3.5", activeTab === "dns" ? "text-sky-400" : "")} aria-hidden />
                    <span>DNS Lookup</span>
                    <ToolTabHeartbeat active={dnsLoading} color="bg-sky-400" />
                </Button>

                <Button
                    type="button"
                    variant={activeTab === "ports" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("ports");
                        setSearchParams({ tool: "ports" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "ports"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Server className={cn("h-3.5 w-3.5", activeTab === "ports" ? "text-emerald-400" : "")} aria-hidden />
                    <span>Port Checker</span>
                    <ToolTabHeartbeat active={portChecking} color="bg-emerald-400" />
                </Button>

                <Button
                    type="button"
                    variant={activeTab === "wol" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("wol");
                        setSearchParams({ tool: "wol" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "wol"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Power className={cn("h-3.5 w-3.5", activeTab === "wol" ? "text-amber-400" : "")} aria-hidden />
                    <span>Wake-on-LAN</span>
                    <ToolTabHeartbeat active={wolSending} color="bg-amber-400" />
                </Button>

                <Button
                    type="button"
                    variant={activeTab === "speed" ? "secondary" : "ghost"}
                    size="sm"
                    onClick={() => {
                        setActiveTab("speed");
                        setSearchParams({ tool: "speed" });
                    }}
                    className={cn(
                        "gap-1.5 text-xs h-8.5 px-3 rounded-lg transition-all",
                        activeTab === "speed"
                            ? "bg-card text-foreground font-semibold border border-border"
                            : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <Zap className="h-3.5 w-3.5 text-amber-500" aria-hidden />
                    <span>Velocidad y Salud</span>
                    <ToolTabHeartbeat active={false} color="bg-amber-500" />
                </Button>
            </nav>

            {/* =========================================================================
                PESTAÑA: PING INTERACTIVO (Workbench Studio)
            ========================================================================== */}
            {activeTab === "ping" && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
                    {/* Columna Izquierda: Panel de Control & Objetivos */}
                    <div className="lg:col-span-5 flex flex-col gap-4">
                        <Card className="border-border">
                            <CardHeader className="p-4 sm:p-5 pb-3">
                                <CardTitle as="h2" className="flex items-center gap-2">
                                    <Activity className="h-4 w-4 text-primary" aria-hidden />
                                    Sondeo Ping en tiempo real
                                </CardTitle>
                                <CardDescription className="text-xs">
                                    Envía paquetes continuos para evaluar la latencia, variación de jitter (RFC 3550) y estabilidad del enlace.
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="p-4 sm:p-5 pt-1 flex flex-col gap-4">
                                <div className="flex flex-col gap-2.5">
                                    <label htmlFor="ping-target-input" className="text-xs font-medium text-foreground">
                                        Destino de sondeo
                                    </label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="ping-target-input"
                                            placeholder="IP o dominio (ej. 192.168.1.1 o cloudflare.com)"
                                            value={pingTarget}
                                            aria-invalid={Boolean(pingError)}
                                            aria-describedby={pingError ? "ping-target-error" : undefined}
                                            onChange={(e) => {
                                                setPingTarget(e.target.value);
                                                if (pingError && e.target.value.trim()) setPingError(null);
                                            }}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    if (pinging) handleStopPing();
                                                    else handleStartPing();
                                                }
                                            }}
                                            disabled={pinging}
                                            className="font-mono text-xs flex-1"
                                        />
                                        {pinging ? (
                                            <Button
                                                type="button"
                                                variant="destructive"
                                                size="sm"
                                                onClick={handleStopPing}
                                                className="gap-1.5 shrink-0"
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
                                                className="gap-1.5 shrink-0"
                                            >
                                                <Play className="h-3.5 w-3.5" aria-hidden />
                                                Iniciar Ping
                                            </Button>
                                        )}
                                    </div>
                                    {pingError && (
                                        <p id="ping-target-error" className="text-xs text-destructive" role="alert">
                                            {pingError}
                                        </p>
                                    )}

                                    {/* Selector de límite de pings enviados */}
                                    <div className="flex items-center justify-between gap-2 pt-1">
                                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                            <span className="font-medium text-foreground">Límite de pings:</span>
                                            <Input
                                                id="ping-count-input"
                                                type="number"
                                                min={1}
                                                max={1000}
                                                placeholder="∞"
                                                value={pingCount ?? ""}
                                                onChange={(e) => {
                                                    const val = e.target.value.trim();
                                                    if (val === "" || val === "0") {
                                                        setPingCount(null);
                                                    } else {
                                                        const n = parseInt(val, 10);
                                                        setPingCount(isNaN(n) ? null : Math.max(1, n));
                                                    }
                                                }}
                                                disabled={pinging}
                                                className="w-16 h-7.5 font-mono text-xs px-2"
                                                title="Cantidad de paquetes a enviar (dejar vacío o pulsar ∞ para sondeo continuo)"
                                            />
                                        </div>
                                        <div className="flex items-center gap-1">
                                            {[5, 10, 20].map((num) => (
                                                <Button
                                                    key={num}
                                                    type="button"
                                                    variant="ghost"
                                                    size="sm"
                                                    disabled={pinging}
                                                    onClick={() => setPingCount(num)}
                                                    className={cn(
                                                        "h-6.5 px-2 text-[11px] font-mono",
                                                        pingCount === num
                                                            ? "bg-primary/10 text-primary font-bold border border-primary/20"
                                                            : "text-muted-foreground hover:text-foreground",
                                                    )}
                                                >
                                                    {num}
                                                </Button>
                                            ))}
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                size="sm"
                                                disabled={pinging}
                                                onClick={() => setPingCount(null)}
                                                className={cn(
                                                    "h-6.5 px-2 text-[11px] font-mono",
                                                    pingCount === null
                                                        ? "bg-primary/10 text-primary font-bold border border-primary/20"
                                                        : "text-muted-foreground hover:text-foreground",
                                                )}
                                                title="Sondeo continuo sin límite"
                                            >
                                                ∞ Continuo
                                            </Button>
                                        </div>
                                    </div>
                                </div>

                                {/* Objetivos rápidos predefinidos */}
                                <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                    <span className="text-[11px] font-semibold text-muted-foreground">
                                        Atajos y Objetivos Rápidos
                                    </span>
                                    <div className="flex flex-wrap gap-1.5">
                                        {COMMON_PING_TARGETS.map((tgt) => (
                                            <Button
                                                key={tgt.target}
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                disabled={pinging}
                                                onClick={() => {
                                                    setPingTarget(tgt.target);
                                                    if (pingError) setPingError(null);
                                                }}
                                                className={cn(
                                                    "h-7 text-[11px] font-mono px-2.5",
                                                    pingTarget === tgt.target ? "border-primary/50 bg-primary/5" : "",
                                                )}
                                            >
                                                {tgt.label}
                                            </Button>
                                        ))}
                                    </div>
                                </div>

                                {/* Dispositivos en la LAN activa */}
                                {lanDevices.length > 0 && (
                                    <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                        <span className="text-[11px] font-semibold text-muted-foreground">
                                            Dispositivos detectados en LAN
                                        </span>
                                        <div className="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto">
                                            {lanDevices.slice(0, 6).map((dev) => (
                                                <Button
                                                    key={dev.ip}
                                                    type="button"
                                                    variant="outline"
                                                    size="sm"
                                                    disabled={pinging}
                                                    onClick={() => {
                                                        setPingTarget(dev.ip);
                                                        if (pingError) setPingError(null);
                                                    }}
                                                    className="h-6 text-[11px] font-mono px-2 text-muted-foreground hover:text-foreground"
                                                >
                                                    {dev.name ? `${dev.name} (${maskDisplay(dev.ip)})` : maskDisplay(dev.ip)}
                                                </Button>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {/* Especificaciones del motor de sondeo */}
                                <div className="rounded-lg bg-muted/20 border border-border p-3 text-[11px] flex flex-col gap-1.5 text-muted-foreground">
                                    <div className="flex items-center justify-between">
                                        <span className="font-semibold text-foreground">Protocolo:</span>
                                        <span className="font-mono">ICMP Echo + TCP SYN fallback</span>
                                    </div>
                                    <div className="flex items-center justify-between">
                                        <span className="font-semibold text-foreground">Intervalo / Timeout:</span>
                                        <span className="font-mono">1000 ms / 1000 ms</span>
                                    </div>
                                    <div className="flex items-center justify-between">
                                        <span className="font-semibold text-foreground">Límite / Modo:</span>
                                        <span className="font-mono">
                                            {pingCount ? `${pingCount} pings (${pingCount}s)` : "Continuo (Sin límite)"}
                                        </span>
                                    </div>
                                    <div className="flex items-center justify-between">
                                        <span className="font-semibold text-foreground">Cálculo Jitter:</span>
                                        <span className="font-mono">RFC 3550 (Media exponencial)</span>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Columna Derecha: Telemetría, Osciloscopio y Stream de Paquetes */}
                    <div className="lg:col-span-7 flex flex-col gap-4">
                        {pingPackets.length === 0 ? (
                            /* Estado de Espera / Readiness HUD */
                            <div
                                key="ping-readiness-hud"
                                className="rounded-xl border border-dashed border-border bg-muted/10 p-6 flex flex-col gap-4 text-center items-center justify-center min-h-[360px]"
                            >
                                <div className="h-12 w-12 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center text-primary">
                                    <Activity className="h-6 w-6" aria-hidden />
                                </div>
                                <div className="max-w-md flex flex-col gap-1">
                                    <h3 className="text-sm font-semibold text-foreground">
                                        Sondeo de latencia en espera
                                    </h3>
                                    <p className="text-xs text-muted-foreground leading-relaxed">
                                        Selecciona un destino a la izquierda y pulsa Iniciar Ping para comenzar la captura en tiempo real de latencia, varianza de jitter y pérdida de paquetes.
                                    </p>
                                </div>
                                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 w-full max-w-lg pt-2 text-left">
                                    <div className="rounded-lg border bg-background/50 p-2.5 overflow-hidden">
                                        <span className="text-[11px] uppercase font-semibold text-muted-foreground block">
                                            LAN Local
                                        </span>
                                        <span className="text-xs font-mono font-bold text-emerald-500 block">
                                            &lt; 2 ms
                                        </span>
                                        <span className="text-[11px] text-muted-foreground block">
                                            Cable / Wi-Fi 6
                                        </span>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5 overflow-hidden">
                                        <span className="text-[11px] uppercase font-semibold text-muted-foreground block">
                                            Fibra FTTH
                                        </span>
                                        <span className="text-xs font-mono font-bold text-primary block">
                                            5 - 20 ms
                                        </span>
                                        <span className="text-[11px] text-muted-foreground block">
                                            CDN & Nodos
                                        </span>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5 overflow-hidden">
                                        <span className="text-[11px] uppercase font-semibold text-muted-foreground block">
                                            4G / DSL
                                        </span>
                                        <span className="text-xs font-mono font-bold text-amber-500 block">
                                            25 - 60 ms
                                        </span>
                                        <span className="text-[11px] text-muted-foreground block">
                                            Móvil común
                                        </span>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5 overflow-hidden">
                                        <span className="text-[11px] uppercase font-semibold text-muted-foreground block">
                                            Jitter RFC 3550
                                        </span>
                                        <span className="text-xs font-mono font-bold text-emerald-500 block">
                                            &lt; 5 ms
                                        </span>
                                        <span className="text-[11px] text-muted-foreground block">
                                            Óptimo VoIP
                                        </span>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            /* Telemetría activa: KPIs + Osciloscopio + Consola Terminal */
                            <div key="ping-active-telemetry" className="flex flex-col gap-4">
                                {/* Grid de Estadísticas / KPI HUD */}
                                <div className="grid grid-cols-2 sm:grid-cols-5 gap-2.5">
                                    <Card className="p-3 text-center">
                                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground block">
                                            Transmitidos
                                        </span>
                                        <div className="text-xl font-bold font-mono text-foreground mt-0.5">
                                            {pingSummary ? pingSummary.transmitted : pingPackets.length}
                                        </div>
                                    </Card>

                                    <Card className="p-3 text-center">
                                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground block">
                                            Recibidos
                                        </span>
                                        <div className="text-xl font-bold font-mono text-emerald-600 dark:text-emerald-400 mt-0.5">
                                            {pingSummary
                                                ? pingSummary.received
                                                : pingPackets.filter((p) => p.reachable).length}
                                        </div>
                                    </Card>

                                    <Card className="p-3 text-center">
                                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground block">
                                            Pérdida
                                        </span>
                                        <div
                                            className={cn(
                                                "text-xl font-bold font-mono mt-0.5",
                                                (pingSummary?.packet_loss_percent || 0) > 0
                                                    ? "text-rose-500"
                                                    : "text-emerald-600 dark:text-emerald-400",
                                            )}
                                        >
                                            {pingSummary
                                                ? `${pingSummary.packet_loss_percent.toFixed(1)}%`
                                                : `${(
                                                      ((pingPackets.filter((p) => !p.reachable).length) /
                                                          pingPackets.length) *
                                                      100
                                                  ).toFixed(0)}%`}
                                        </div>
                                    </Card>

                                    <Card className="p-3 text-center">
                                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground block">
                                            Último RTT
                                        </span>
                                        <div className="text-xl font-bold font-mono text-primary mt-0.5">
                                            {pingPackets[pingPackets.length - 1]?.latency_ms != null
                                                ? `${pingPackets[pingPackets.length - 1].latency_ms} ms`
                                                : "-"}
                                        </div>
                                    </Card>

                                    <Card className="p-3 text-center">
                                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground block">
                                            Jitter RFC 3550
                                        </span>
                                        <div className="text-xl font-bold font-mono text-amber-600 dark:text-amber-400 mt-0.5">
                                            {pingPackets[pingPackets.length - 1]?.jitter_ms != null
                                                ? `${pingPackets[pingPackets.length - 1].jitter_ms.toFixed(1)} ms`
                                                : "-"}
                                        </div>
                                    </Card>
                                </div>

                                {/* Osciloscopio Gráfico SVG */}
                                <LatencyOscilloscope packets={pingPackets} />

                                {/* Stream de Paquetes tipo Consola Terminal */}
                                <Card className="overflow-hidden flex flex-col">
                                    <div className="flex items-center justify-between px-3.5 py-2 border-b border-border bg-muted/30">
                                        <div className="flex items-center gap-2">
                                            <Terminal className="h-3.5 w-3.5 text-primary" aria-hidden />
                                            <span className="text-xs font-semibold text-foreground font-mono">
                                                Stream de paquetes ({pingPackets.length}{pingCount ? `/${pingCount}` : ""})
                                            </span>
                                            {pinging && (
                                                <Badge variant="success" className="gap-1 font-mono font-medium">
                                                    <span className="h-2 w-2 rounded-full bg-success animate-pulse" />
                                                    Capturando
                                                </Badge>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                size="sm"
                                                onClick={handleCopyPingStream}
                                                className="h-7 px-2 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
                                                title="Copiar historial"
                                            >
                                                {copiedPing ? (
                                                    <Check className="h-3 w-3 text-emerald-500" />
                                                ) : (
                                                    <Copy className="h-3 w-3" />
                                                )}
                                                <span>{copiedPing ? "Copiado" : "Copiar"}</span>
                                            </Button>
                                            {!pinging && (
                                                <Button
                                                    type="button"
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={handleClearPing}
                                                    className="h-7 px-2 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
                                                    title="Limpiar paquetes"
                                                >
                                                    <RotateCcw className="h-3 w-3" />
                                                    <span>Limpiar</span>
                                                </Button>
                                            )}
                                        </div>
                                    </div>

                                    <div className="p-3 font-mono text-xs flex flex-col gap-1.5 max-h-64 overflow-y-auto bg-muted/10">
                                        {pingPackets
                                            .slice()
                                            .reverse()
                                            .map((pkt) => (
                                                <div
                                                    key={pkt.seq}
                                                    className="flex items-center justify-between border-b border-border pb-1 last:border-0 hover:bg-muted/30 px-1.5 py-0.5 rounded transition-colors"
                                                >
                                                    <div className="flex items-center gap-2 truncate">
                                                        <span className="text-muted-foreground text-[11px] w-8 shrink-0">
                                                            #{pkt.seq}
                                                        </span>
                                                        {pkt.reachable ? (
                                                            <span className="text-emerald-600 dark:text-emerald-400 font-semibold truncate">
                                                                Respuesta de {maskDisplay(pkt.target)}
                                                            </span>
                                                        ) : (
                                                            <span className="text-destructive font-semibold">
                                                                Tiempo de espera agotado
                                                            </span>
                                                        )}
                                                        <span className="text-[11px] text-muted-foreground/80 shrink-0">
                                                            ({pkt.method})
                                                        </span>
                                                    </div>
                                                    <div className="flex items-center gap-3 shrink-0 ml-2">
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
                                </Card>
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* =========================================================================
                PESTAÑA: TRACEROUTE VISUAL (Workbench Studio)
            ========================================================================== */}
            {activeTab === "traceroute" && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
                    {/* Panel de Configuración y Lanzamiento */}
                    <div className="lg:col-span-5 flex flex-col gap-4">
                        <Card className="border-border">
                            <CardHeader className="p-4 sm:p-5 pb-3">
                                <CardTitle as="h2" className="flex items-center gap-2">
                                    <Compass className="h-4 w-4 text-indigo-400" aria-hidden />
                                    Trazado de ruta (Traceroute)
                                </CardTitle>
                                <CardDescription className="text-xs">
                                    Identifica cada router y salto intermedio en el camino hacia el destino con análisis de latencia acumulada.
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="p-4 sm:p-5 pt-1 flex flex-col gap-4">
                                <div className="flex flex-col gap-3">
                                    <div className="flex flex-col gap-1.5">
                                        <label htmlFor="trace-target-input" className="text-xs font-medium text-foreground">
                                            Host o IP de destino
                                        </label>
                                        <Input
                                            id="trace-target-input"
                                            placeholder="IP o dominio (ej. 1.1.1.1 o debian.org)"
                                            value={traceTarget}
                                            aria-invalid={Boolean(traceError)}
                                            aria-describedby={traceError ? "trace-target-error" : undefined}
                                            onChange={(e) => {
                                                setTraceTarget(e.target.value);
                                                if (traceError && e.target.value.trim()) setTraceError(null);
                                            }}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    if (tracing) handleStopTraceroute();
                                                    else handleStartTraceroute();
                                                }
                                            }}
                                            disabled={tracing}
                                            className="font-mono text-xs"
                                        />
                                        {traceError && (
                                            <p id="trace-target-error" className="text-xs text-destructive" role="alert">
                                                {traceError}
                                            </p>
                                        )}
                                    </div>

                                    <div className="flex items-center justify-between gap-3 pt-1">
                                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
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

                                {/* Atajos rápidos de trazado */}
                                <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                    <span className="text-[11px] font-semibold text-muted-foreground">
                                        Objetivos de Red
                                    </span>
                                    <div className="flex flex-wrap gap-1.5">
                                        {COMMON_TRACE_TARGETS.map((tgt) => (
                                            <Button
                                                key={tgt.target}
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                disabled={tracing}
                                                onClick={() => {
                                                    setTraceTarget(tgt.target);
                                                    if (traceError) setTraceError(null);
                                                }}
                                                className="h-7 text-[11px] font-mono px-2.5"
                                            >
                                                {tgt.label}
                                            </Button>
                                        ))}
                                    </div>
                                </div>

                                <div className="rounded-lg bg-muted/20 border border-border p-3 text-[11px] flex flex-col gap-1 text-muted-foreground">
                                    <span className="font-semibold text-foreground">Topología de Salto (TTL):</span>
                                    <span>
                                        Incrementa progresivamente el tiempo de vida (TTL) del paquete IP para forzar el retorno de ICMP Time Exceeded (RFC 1393) desde cada enrutador intermedio.
                                    </span>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Mapa Visual de Saltos y Tabla de Rutas */}
                    <div className="lg:col-span-7 flex flex-col gap-4">
                        {traceHops.length === 0 ? (
                            <div key="trace-readiness-hud" className="rounded-xl border border-dashed border-border bg-muted/10 p-6 flex flex-col gap-4 text-center items-center justify-center min-h-[340px]">
                                <div className="h-12 w-12 rounded-full bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                                    <Compass className="h-6 w-6" aria-hidden />
                                </div>
                                <div className="max-w-md flex flex-col gap-1">
                                    <h3 className="text-sm font-semibold text-foreground">
                                        Mapeo de ruta en espera
                                    </h3>
                                    <p className="text-xs text-muted-foreground leading-relaxed">
                                        Inicia un trazado de ruta para visualizar el tránsito de paquetes a través de los nodos de tu ISP, backbones globales y puntos de intercambio IXP.
                                    </p>
                                </div>
                                <div className="flex flex-wrap items-center justify-center gap-1.5 text-xs font-mono text-muted-foreground pt-2">
                                    <span className="px-2 py-1 rounded bg-background border border-border text-[11px]">
                                        Tu Equipo
                                    </span>
                                    <ArrowRight className="h-3 w-3 text-muted-foreground" />
                                    <span className="px-2 py-1 rounded bg-background border border-border text-[11px]">
                                        Router LAN
                                    </span>
                                    <ArrowRight className="h-3 w-3 text-muted-foreground" />
                                    <span className="px-2 py-1 rounded bg-background border border-border text-[11px]">
                                        Gateway ISP
                                    </span>
                                    <ArrowRight className="h-3 w-3 text-muted-foreground" />
                                    <span className="px-2 py-1 rounded bg-background border border-border text-[11px]">
                                        Tier-1 Transit
                                    </span>
                                    <ArrowRight className="h-3 w-3 text-muted-foreground" />
                                    <span className="px-2 py-1 rounded bg-background border border-border text-[11px]">
                                        Destino
                                    </span>
                                </div>
                            </div>
                        ) : (
                            <div key="trace-active-map" className="flex flex-col gap-3">
                                {/* Carrusel/Cadena visual interactiva de nodos */}
                                <Card className="p-3 overflow-x-auto">
                                    <div className="flex items-center gap-2 min-w-max pb-1">
                                        {traceHops.map((hop, idx) => {
                                            const lat = hop.latency_ms;
                                            const latColor =
                                                lat == null
                                                    ? "text-muted-foreground bg-muted/60 border-border"
                                                    : lat < 30
                                                    ? "text-success-strong bg-success/10 border-success/30"
                                                    : lat < 100
                                                    ? "text-warning-strong bg-warning/10 border-warning/30"
                                                    : "text-destructive-strong bg-destructive/10 border-destructive/30";

                                            return (
                                                <div key={hop.hop_number} className="flex items-center gap-2">
                                                    <div className="flex flex-col items-center gap-1 p-2 rounded-lg border bg-background/80 min-w-[96px] text-center">
                                                        <span className="text-[11px] font-bold text-muted-foreground">
                                                            Salto #{hop.hop_number}
                                                        </span>
                                                        <span className="text-xs font-mono font-semibold text-foreground truncate max-w-[110px]">
                                                            {hop.ip ? maskDisplay(hop.ip) : "*"}
                                                        </span>
                                                        <span
                                                            className={cn(
                                                                "text-[11px] font-mono px-1.5 py-0.5 rounded border font-semibold",
                                                                latColor,
                                                            )}
                                                        >
                                                            {lat != null ? `${lat} ms` : "Timeout"}
                                                        </span>
                                                    </div>
                                                    {idx < traceHops.length - 1 && (
                                                        <ArrowRight className="h-3.5 w-3.5 text-muted-foreground/60 shrink-0" />
                                                    )}
                                                </div>
                                            );
                                        })}
                                    </div>
                                </Card>

                                {/* Tabla completa de saltos */}
                                <Card className="overflow-hidden">
                                    <div className="flex items-center justify-between px-4 py-2.5 border-b border-border bg-muted/20">
                                        <div className="flex items-center gap-2 text-xs font-semibold">
                                            <Compass className="h-3.5 w-3.5 text-indigo-400" />
                                            <span>Detalle de saltos ({traceHops.length})</span>
                                        </div>
                                        {tracing && (
                                            <div className="flex items-center gap-1.5 text-xs text-muted-foreground font-mono">
                                                <Loader2 className="h-3.5 w-3.5 animate-spin text-primary" aria-hidden />
                                                <span>Descubriendo saltos de red…</span>
                                            </div>
                                        )}
                                    </div>

                                    <Table className="text-xs font-mono">
                                        <TableHeader className="bg-muted/30 text-[11px] uppercase">
                                            <TableRow>
                                                <TableHead className="p-2.5 w-12">#</TableHead>
                                                <TableHead className="p-2.5">Dirección IP</TableHead>
                                                <TableHead className="p-2.5">Hostname resuelto</TableHead>
                                                <TableHead className="p-2.5">Estado</TableHead>
                                                <TableHead className="p-2.5 text-right">Latencia RTT</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody className="divide-y divide-border">
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
                                                    <TableRow key={hop.hop_number} className="hover:bg-muted/30">
                                                        <TableCell className="p-2.5 text-muted-foreground font-semibold">
                                                            {hop.hop_number}
                                                        </TableCell>
                                                        <TableCell className="p-2.5">
                                                            {hop.ip ? (
                                                                <span className="font-semibold text-foreground">
                                                                    {maskDisplay(hop.ip)}
                                                                </span>
                                                            ) : (
                                                                <span className="text-muted-foreground">*</span>
                                                            )}
                                                        </TableCell>
                                                        <TableCell className="p-2.5 text-muted-foreground truncate max-w-[200px]">
                                                            {hop.hostname ? maskDisplay(hop.hostname) : "-"}
                                                        </TableCell>
                                                        <TableCell className="p-2.5">
                                                            <Badge
                                                                variant={
                                                                    hop.state === "reached"
                                                                        ? "success"
                                                                        : hop.state === "*"
                                                                        ? "secondary"
                                                                        : "info"
                                                                }
                                                                className="px-1.5 py-0.5 font-medium"
                                                            >
                                                                {hop.state}
                                                            </Badge>
                                                        </TableCell>
                                                        <TableCell className={cn("p-2.5 text-right", latBadgeClass)}>
                                                            {lat != null ? `${lat} ms` : "*"}
                                                        </TableCell>
                                                    </TableRow>
                                                );
                                            })}
                                        </TableBody>
                                    </Table>
                                </Card>
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* =========================================================================
                PESTAÑA: DNS LOOKUP (Workbench Studio)
            ========================================================================== */}
            {activeTab === "dns" && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
                    {/* Consola de Parámetros DNS */}
                    <div className="lg:col-span-5 flex flex-col gap-4">
                        <Card className="border-border">
                            <CardHeader className="p-4 sm:p-5 pb-3">
                                <CardTitle as="h2" className="flex items-center gap-2">
                                    <Globe className="h-4 w-4 text-sky-400" aria-hidden />
                                    Consulta DNS Multidominio
                                </CardTitle>
                                <CardDescription className="text-xs">
                                    Resuelve registros A, AAAA, CNAME, MX, TXT, NS, PTR y SOA alternando entre tu DNS local y proveedores públicos.
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="p-4 sm:p-5 pt-1 flex flex-col gap-4">
                                <div className="flex flex-col gap-3">
                                    <div>
                                        <label className="text-xs font-medium text-foreground mb-1 block">
                                            Dominio o IP
                                        </label>
                                        <Input
                                            placeholder="ej. github.com o 1.1.1.1"
                                            value={dnsTarget}
                                            onChange={(e) => setDnsTarget(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    handleDnsLookup();
                                                }
                                            }}
                                            className="font-mono text-xs"
                                        />
                                    </div>

                                    <div className="grid grid-cols-2 gap-2.5">
                                        <div>
                                            <label
                                                htmlFor="dns-type-select"
                                                className="text-xs font-medium text-foreground mb-1 block"
                                            >
                                                Tipo de registro
                                            </label>
                                            <Select value={dnsType} onValueChange={setDnsType}>
                                                <SelectTrigger
                                                    id="dns-type-select"
                                                    className="w-full font-mono text-xs"
                                                    aria-label="Tipo de registro"
                                                >
                                                    <SelectValue placeholder="Tipo" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {DNS_TYPES.map((t) => (
                                                        <SelectItem key={t} value={t} className="font-mono text-xs">
                                                            {t}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>

                                        <div>
                                            <label
                                                htmlFor="dns-provider-select"
                                                className="text-xs font-medium text-foreground mb-1 block"
                                            >
                                                Servidor DNS
                                            </label>
                                            <Select value={dnsProvider} onValueChange={setDnsProvider}>
                                                <SelectTrigger
                                                    id="dns-provider-select"
                                                    className="w-full text-xs"
                                                    aria-label="Servidor DNS"
                                                >
                                                    <SelectValue placeholder="Servidor DNS" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {DNS_PROVIDERS.map((p) => (
                                                        <SelectItem key={p.id} value={p.id} className="text-xs">
                                                            {p.label}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>
                                    </div>

                                    <div className="flex justify-end pt-1">
                                        <Button
                                            type="button"
                                            variant="default"
                                            size="sm"
                                            disabled={dnsLoading}
                                            onClick={handleDnsLookup}
                                            className="gap-1.5 w-full sm:w-auto"
                                        >
                                            {dnsLoading ? (
                                                <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                            ) : (
                                                <Search className="h-3.5 w-3.5" aria-hidden />
                                            )}
                                            Consultar DNS
                                        </Button>
                                    </div>
                                </div>

                                {/* Selector rápido de tipos de registro */}
                                <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                    <span className="text-[11px] font-semibold text-muted-foreground">
                                        Tipos de Registro Rápidos
                                    </span>
                                    <div className="flex flex-wrap gap-1">
                                        {DNS_TYPES.map((t) => (
                                            <Button
                                                key={t}
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                onClick={() => setDnsType(t)}
                                                className={cn(
                                                    "h-6 text-[11px] font-mono px-2",
                                                    dnsType === t
                                                        ? "bg-primary/10 text-primary border-primary/30 font-bold"
                                                        : "text-muted-foreground",
                                                )}
                                            >
                                                {t}
                                            </Button>
                                        ))}
                                    </div>
                                </div>

                                {/* Dominios sugeridos */}
                                <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                    <span className="text-[11px] font-semibold text-muted-foreground">
                                        Dominios sugeridos
                                    </span>
                                    <div className="flex flex-wrap gap-1.5">
                                        {COMMON_DNS_TARGETS.map((domain) => (
                                            <Button
                                                key={domain}
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                onClick={() => setDnsTarget(domain)}
                                                className="h-6 text-[11px] font-mono px-2 text-muted-foreground hover:text-foreground"
                                            >
                                                {domain}
                                            </Button>
                                        ))}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Inspector de Registros DNS */}
                    <div className="lg:col-span-7 flex flex-col gap-4">
                        {dnsRecords.length === 0 ? (
                            <div className="rounded-xl border border-dashed border-border bg-muted/10 p-6 flex flex-col gap-4 text-center items-center justify-center min-h-[340px]">
                                <div className="h-12 w-12 rounded-full bg-primary/10 border border-primary/30 flex items-center justify-center text-primary">
                                    <Globe className="h-6 w-6" aria-hidden />
                                </div>
                                <div className="max-w-md flex flex-col gap-1">
                                    <h3 className="text-sm font-semibold text-foreground">
                                        Inspector de registros DNS listo
                                    </h3>
                                    <p className="text-xs text-muted-foreground leading-relaxed">
                                        Consulta registros de resolución directa o inversa para diagnosticar propagación DNS, configuraciones SPF/DKIM y asignaciones de red.
                                    </p>
                                </div>
                                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 w-full max-w-lg pt-2 text-left">
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <Badge
                                            variant="outline"
                                            className="mb-1"
                                        >
                                            A / AAAA
                                        </Badge>
                                        <p className="text-[11px] text-muted-foreground">
                                            Mapeo IPv4 (32-bit) e IPv6 (128-bit)
                                        </p>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <Badge
                                            variant="outline"
                                            className="mb-1"
                                        >
                                            MX
                                        </Badge>
                                        <p className="text-[11px] text-muted-foreground">
                                            Rutas de servidores de correo SMTP
                                        </p>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <Badge
                                            variant="outline"
                                            className="mb-1 text-[11px] bg-purple-500/10 text-purple-400 border-purple-500/20"
                                        >
                                            TXT
                                        </Badge>
                                        <p className="text-[11px] text-muted-foreground">
                                            Políticas SPF, DKIM y verificación
                                        </p>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <Badge
                                            variant="outline"
                                            className="mb-1"
                                        >
                                            CNAME / PTR
                                        </Badge>
                                        <p className="text-[11px] text-muted-foreground">
                                            Alias canónico y búsqueda inversa
                                        </p>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <Card className="overflow-hidden flex flex-col">
                                <div className="flex items-center justify-between px-4 py-2.5 border-b border-border bg-muted/20">
                                    <div className="flex items-center gap-2">
                                        <Globe className="h-3.5 w-3.5 text-sky-400" />
                                        <span className="text-xs font-semibold text-foreground font-mono">
                                            Registros resueltos ({dnsRecords.length})
                                        </span>
                                    </div>
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={handleCopyDnsRecords}
                                        className="h-7 px-2 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
                                    >
                                        {copiedDns ? (
                                            <Check className="h-3 w-3 text-emerald-500" />
                                        ) : (
                                            <Copy className="h-3 w-3" />
                                        )}
                                        <span>{copiedDns ? "Copiado" : "Copiar todo"}</span>
                                    </Button>
                                </div>

                                <Table className="text-xs font-mono">
                                    <TableHeader className="bg-muted/30 text-[11px] uppercase">
                                        <TableRow>
                                            <TableHead className="p-2.5">Nombre</TableHead>
                                            <TableHead className="p-2.5 w-20">Tipo</TableHead>
                                            <TableHead className="p-2.5">Valor resuelto</TableHead>
                                            <TableHead className="p-2.5 text-right w-20">TTL</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody className="divide-y divide-border">
                                        {dnsRecords.map((r, idx) => (
                                            <TableRow
                                                key={`${r.record_type}-${r.value}-${idx}`}
                                                className="hover:bg-muted/30"
                                            >
                                                <TableCell className="p-2.5 font-medium">{r.name}</TableCell>
                                                <TableCell className="p-2.5">
                                                    <Badge
                                                        variant="outline"
                                                        className="px-1.5 py-0.5 text-[11px] font-semibold bg-primary/10 text-primary border-primary/20"
                                                    >
                                                        {r.record_type}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell className="p-2.5 font-semibold text-foreground break-all">
                                                    {maskDisplay(r.value)}
                                                </TableCell>
                                                <TableCell className="p-2.5 text-right text-muted-foreground">
                                                    {r.ttl}s
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </Card>
                        )}
                    </div>
                </div>
            )}

            {/* =========================================================================
                PESTAÑA: PORT CHECKER (Workbench Studio)
            ========================================================================== */}
            {activeTab === "ports" && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
                    {/* Consola de Parámetros de Puerto */}
                    <div className="lg:col-span-5 flex flex-col gap-4">
                        <Card className="border-border">
                            <CardHeader className="p-4 sm:p-5 pb-3">
                                <CardTitle as="h2" className="flex items-center gap-2">
                                    <Server className="h-4 w-4 text-emerald-400" aria-hidden />
                                    Comprobación rápida de puerto (Port Checker)
                                </CardTitle>
                                <CardDescription className="text-xs">
                                    Verifica si un servicio o puerto TCP específico está abierto, cerrado o bloqueado por firewall en un host puntual.
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="p-4 sm:p-5 pt-1 flex flex-col gap-4">
                                <div className="flex flex-col gap-3">
                                    <div>
                                        <label className="text-xs font-medium text-foreground mb-1 block">
                                            Host o Dirección IP
                                        </label>
                                        <Input
                                            placeholder="ej. 192.168.1.1 o mi-servidor.lan"
                                            value={portTarget}
                                            onChange={(e) => setPortTarget(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    handleCheckPort();
                                                }
                                            }}
                                            className="font-mono text-xs"
                                        />
                                    </div>

                                    <div>
                                        <label className="text-xs font-medium text-foreground mb-1 block">
                                            Puerto TCP (1 - 65535)
                                        </label>
                                        <Input
                                            type="number"
                                            min={1}
                                            max={65535}
                                            value={portNumber}
                                            onChange={(e) => setPortNumber(Number(e.target.value))}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    handleCheckPort();
                                                }
                                            }}
                                            className="font-mono text-xs"
                                        />
                                    </div>

                                    <div className="flex justify-end pt-1">
                                        <Button
                                            type="button"
                                            variant="default"
                                            size="sm"
                                            disabled={portChecking}
                                            onClick={handleCheckPort}
                                            className="gap-1.5 w-full sm:w-auto"
                                        >
                                            {portChecking ? (
                                                <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                            ) : (
                                                <Zap className="h-3.5 w-3.5" aria-hidden />
                                            )}
                                            Comprobar puerto
                                        </Button>
                                    </div>
                                </div>

                                {/* Accesos rápidos a puertos comunes */}
                                <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                    <span className="text-[11px] font-semibold text-muted-foreground">
                                        Puertos y Servicios Frecuentes
                                    </span>
                                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5">
                                        {QUICK_PORTS.map((qp) => (
                                            <Button
                                                key={qp.port}
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                onClick={() => setPortNumber(qp.port)}
                                                className={cn(
                                                    "h-7 text-[11px] font-mono px-2 justify-start truncate",
                                                    portNumber === qp.port
                                                        ? "border-primary/50 bg-primary/10 text-primary font-bold"
                                                        : "",
                                                )}
                                            >
                                                {qp.port} ({qp.name})
                                            </Button>
                                        ))}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* HUD de Resultado del Puerto */}
                    <div className="lg:col-span-7 flex flex-col gap-4">
                        {!portResult ? (
                            <div className="rounded-xl border border-dashed border-border bg-muted/10 p-6 flex flex-col gap-4 text-center items-center justify-center min-h-[320px]">
                                <div className="h-12 w-12 rounded-full bg-success/10 border border-success/30 flex items-center justify-center text-success">
                                    <Server className="h-6 w-6" aria-hidden />
                                </div>
                                <div className="max-w-md flex flex-col gap-1">
                                    <h3 className="text-sm font-semibold text-foreground">
                                        Sondeo puntual de socket TCP
                                    </h3>
                                    <p className="text-xs text-muted-foreground leading-relaxed">
                                        Evalúa el ciclo de sincronización TCP SYN/ACK para comprobar la disponibilidad inmediata de servicios como SSH, servidores HTTP o bases de datos.
                                    </p>
                                </div>
                                <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 w-full max-w-lg pt-2 text-left">
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <span className="text-xs font-semibold text-emerald-500 flex items-center gap-1.5">
                                            <CheckCircle2 className="h-3.5 w-3.5" /> Abierto
                                        </span>
                                        <p className="text-[11px] text-muted-foreground mt-1">
                                            El host acepta la conexión y devuelve SYN-ACK.
                                        </p>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <span className="text-xs font-semibold text-amber-500 flex items-center gap-1.5">
                                            <AlertCircle className="h-3.5 w-3.5" /> Cerrado
                                        </span>
                                        <p className="text-[11px] text-muted-foreground mt-1">
                                            El host responde con paquete TCP RST explícito.
                                        </p>
                                    </div>
                                    <div className="rounded-lg border bg-background/50 p-2.5">
                                        <span className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
                                            <Square className="h-3.5 w-3.5" /> Filtrado / Drop
                                        </span>
                                        <p className="text-[11px] text-muted-foreground mt-1">
                                            Un firewall descarta paquetes silenciosamente.
                                        </p>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <Card className="p-5 flex flex-col gap-4">
                                <div className="flex items-center justify-between pb-3 border-b border-border">
                                    <div className="flex flex-col">
                                        <span className="text-[11px] uppercase font-semibold text-muted-foreground">
                                            Host y Puerto Auditado
                                        </span>
                                        <span className="font-mono text-base font-bold text-foreground">
                                            {maskDisplay(portResult.target)}:{portResult.port}
                                        </span>
                                    </div>

                                    {portResult.latency_ms != null && (
                                        <div className="flex flex-col items-end">
                                            <span className="text-[11px] uppercase font-semibold text-muted-foreground">
                                                Tiempo de Conexión
                                            </span>
                                            <span className="font-mono text-sm text-primary font-bold">
                                                {portResult.latency_ms} ms RTT
                                            </span>
                                        </div>
                                    )}
                                </div>

                                <div className="flex items-center gap-3">
                                    {portResult.state === "open" ? (
                                        <div className="flex items-center gap-2 rounded-lg bg-success/10 text-success-strong border border-success/30 px-3 py-2 text-sm font-bold">
                                            <CheckCircle2 className="h-4 w-4" />
                                            <span>Abierto</span>
                                        </div>
                                    ) : portResult.state === "closed" ? (
                                        <div className="flex items-center gap-2 rounded-lg bg-warning/10 text-warning-strong border border-warning/30 px-3 py-2 text-sm font-semibold">
                                            <AlertCircle className="h-4 w-4" />
                                            <span>Cerrado</span>
                                        </div>
                                    ) : (
                                        <div className="flex items-center gap-2 rounded-lg bg-muted text-muted-foreground border border-border px-3 py-2 text-sm font-semibold">
                                            <Square className="h-4 w-4" />
                                            <span>Filtrado / Timeout</span>
                                        </div>
                                    )}

                                    <span className="text-xs text-muted-foreground">
                                        {portResult.state === "open"
                                            ? "El servicio escuchando en este puerto respondió satisfactoriamente al handshake TCP."
                                            : portResult.state === "closed"
                                            ? "El host destino está activo pero rechazó la conexión (puerto no en escucha)."
                                            : "No se recibió respuesta en el tiempo límite; el tráfico puede estar bloqueado por firewall."}
                                    </span>
                                </div>

                                {portResult.banner && (
                                    <div className="mt-2 text-xs bg-muted/40 rounded-lg border border-border p-3 font-mono break-all text-foreground">
                                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground block mb-1">
                                            Banner de Servicio Detectado:
                                        </span>
                                        Banner: {portResult.banner}
                                    </div>
                                )}
                            </Card>
                        )}
                    </div>
                </div>
            )}

            {/* =========================================================================
                PESTAÑA: WAKE-ON-LAN (Workbench Studio)
            ========================================================================== */}
            {activeTab === "wol" && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
                    {/* Consola de Control WoL */}
                    <div className="lg:col-span-5 flex flex-col gap-4">
                        <Card className="border-border">
                            <CardHeader className="p-4 sm:p-5 pb-3">
                                <CardTitle as="h2" className="flex items-center gap-2">
                                    <Power className="h-4 w-4 text-amber-400" aria-hidden />
                                    Encendido remoto Wake-on-LAN (WoL)
                                </CardTitle>
                                <CardDescription className="text-xs">
                                    Envía un paquete mágico estándar a través de broadcast UDP (puerto 9) para encender equipos compatibles en tu red local.
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="p-4 sm:p-5 pt-1 flex flex-col gap-4">
                                <div className="flex flex-col gap-3">
                                    <div>
                                        <label className="text-xs font-medium text-foreground mb-1 block">
                                            Dirección MAC destino
                                        </label>
                                        <Input
                                            placeholder="ej. AA:BB:CC:DD:EE:FF"
                                            value={wolMac}
                                            onChange={(e) => setWolMac(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    handleSendWol();
                                                }
                                            }}
                                            className="font-mono text-xs"
                                        />
                                    </div>

                                    <div>
                                        <label className="text-xs font-medium text-foreground mb-1 block">
                                            IP Broadcast (Opcional)
                                        </label>
                                        <Input
                                            placeholder="255.255.255.255"
                                            value={wolBroadcast}
                                            onChange={(e) => setWolBroadcast(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    handleSendWol();
                                                }
                                            }}
                                            className="font-mono text-xs"
                                        />
                                    </div>

                                    <div className="flex justify-end pt-1">
                                        <Button
                                            type="button"
                                            variant="default"
                                            size="sm"
                                            disabled={wolSending}
                                            onClick={handleSendWol}
                                            className="gap-1.5 w-full sm:w-auto"
                                        >
                                            {wolSending ? (
                                                <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden />
                                            ) : (
                                                <Power className="h-3.5 w-3.5" aria-hidden />
                                            )}
                                            Enviar Magic Packet
                                        </Button>
                                    </div>
                                </div>

                                {/* Dispositivos LAN con MAC conocida */}
                                {lanDevices.filter((d) => d.mac).length > 0 && (
                                    <div className="flex flex-col gap-2 pt-1 border-t border-border">
                                        <span className="text-[11px] font-semibold text-muted-foreground">
                                            Equipos con MAC detectada en LAN
                                        </span>
                                        <div className="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto">
                                            {lanDevices
                                                .filter((d) => d.mac)
                                                .slice(0, 6)
                                                .map((dev) => (
                                                    <Button
                                                        key={dev.mac}
                                                        type="button"
                                                        variant="outline"
                                                        size="sm"
                                                        disabled={wolSending}
                                                        onClick={() => setWolMac(dev.mac || "")}
                                                        className="h-6 text-[11px] font-mono px-2 text-muted-foreground hover:text-foreground"
                                                    >
                                                        {dev.name ? `${dev.name}` : maskDisplay(dev.mac || "")}
                                                    </Button>
                                                ))}
                                        </div>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>

                    {/* HUD Informativo y Resultado WoL */}
                    <div className="lg:col-span-7 flex flex-col gap-4">
                        {wolResult ? (
                            <Alert variant={wolResult.success ? "default" : "destructive"}>
                                {wolResult.success ? (
                                    <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                                ) : (
                                    <AlertCircle className="h-4 w-4" />
                                )}
                                <AlertTitle>
                                    {wolResult.success ? "Paquete enviado" : "Error en el envío"}
                                </AlertTitle>
                                <AlertDescription className="text-xs font-mono mt-1">
                                    {wolResult.msg}
                                </AlertDescription>
                            </Alert>
                        ) : null}

                        {/* Inspector de Trama Magic Packet */}
                        <Card className="p-5 flex flex-col gap-3">
                            <div className="flex items-center gap-2">
                                <Power className="h-4 w-4 text-amber-400" />
                                <h3 className="text-sm font-semibold text-foreground">
                                    Estructura del Magic Packet (IEEE 802.3)
                                </h3>
                            </div>
                            <p className="text-xs text-muted-foreground leading-relaxed">
                                El paquete mágico es una trama de capa de enlace transmitida vía socket de difusión (Broadcast UDP, comúnmente puertos 7 o 9). Contiene una cabecera de sincronización seguida de la dirección MAC del dispositivo repetida exactamente 16 veces.
                            </p>
                            <div className="rounded-lg bg-muted/20 border border-border p-3 font-mono text-[11px] flex flex-col gap-1.5">
                                <div className="flex items-center justify-between">
                                    <span className="text-muted-foreground">Cabecera de Sincronización:</span>
                                    <span className="text-primary font-bold">FF:FF:FF:FF:FF:FF (6 bytes)</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-muted-foreground">Payload Target:</span>
                                    <span className="text-foreground">
                                        16 repeticiones de {wolMac ? maskDisplay(wolMac) : "MAC destino"}
                                    </span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-muted-foreground">Tamaño Total:</span>
                                    <span className="text-foreground">102 bytes en carga útil</span>
                                </div>
                            </div>
                        </Card>
                    </div>
                </div>
            )}

            {/* =========================================================================
                PESTAÑA: VELOCIDAD Y SALUD
            ========================================================================== */}
            {activeTab === "speed" && <SpeedTestPanel />}
        </div>
    );
}
