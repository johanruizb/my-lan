import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import { HashRouter, NavLink, Route, Routes } from "react-router-dom";
import {
    LayoutDashboard,
    Network,
    Radar,
    Settings as SettingsIcon,
    Sun,
    Moon,
    Activity,
    Info,
    Pencil,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ToastProviderApp } from "@/components/ui/toast";
import { ThemeProvider, useTheme } from "@/components/theme-provider";
import { CensorshipProvider } from "@/components/censorship-provider";
import { CensuraUpgradeDialog } from "@/components/censura-upgrade-dialog";
import { OnboardingDialog } from "@/components/onboarding-dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import type { Device, ScanOutcomeDto, UnlistenFn } from "@/lib/tauri";
import {
    cancelScan,
    getAppVersion,
    getSettings,
    onDiscoveryProgress,
    onScanCancelled,
    onScanDevice,
    onScanFinished,
    onScanStarted,
    runDiscovery,
} from "@/lib/tauri";
import { NetworkNameProvider, useNetworkName } from "@/lib/use-network-name";
import { newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import { Dashboard } from "@/screens/Dashboard";
import { Devices } from "@/screens/Devices";
import { DeviceDetail } from "@/screens/DeviceDetail";
import { Scans } from "@/screens/Scans";
import { Settings } from "@/screens/Settings";
import { AboutDialog } from "@/screens/About";

// Contexto ligero para compartir el resumen del último scan entre el Dashboard
// (que lanza run_discovery) y el header (que lo muestra, AC-1).
interface LastScanContextValue {
    lastScan: ScanOutcomeDto | null;
    setLastScan: (o: ScanOutcomeDto | null) => void;
}
const LastScanContext = createContext<LastScanContextValue | null>(null);

export function useLastScan(): LastScanContextValue {
    const ctx = useContext(LastScanContext);
    if (!ctx)
        throw new Error("useLastScan debe usarse dentro de LastScanProvider");
    return ctx;
}

// Estado GLOBAL de un escaneo en curso (un único scan a la vez) compartido por
// Dashboard y /devices: ambos ven el mismo progreso/hallazgos y pueden cancelar,
// y la navegación preserva la vista en progreso (decisión bloqueada / AC-11).
export interface ScanProgressState {
    swept: number;
    total: number;
    /** % = swept/total; `null` cuando el total es desconocido (indeterminado). */
    percent: number | null;
}

interface ScanContextValue {
    scanning: boolean;
    scanId: string | null;
    progress: ScanProgressState | null;
    devicesFound: Device[];
    startScan: (profile?: string) => Promise<void>;
    cancel: () => void;
}
export const ScanContext = createContext<ScanContextValue | null>(null);

export function useScan(): ScanContextValue {
    const ctx = useContext(ScanContext);
    if (!ctx) throw new Error("useScan debe usarse dentro de ScanProvider");
    return ctx;
}

/** Identidad de merge: primary_ip y, como fallback, primary_mac (AC-6). */
export function deviceKey(d: Device): string {
    return d.primary_ip ?? d.primary_mac ?? d.id;
}

// Merge en sitio por identidad: reemplaza la entrada existente o la añade, sin
// duplicados (AC-6). Inmutable: devuelve una nueva lista. Exportado para test
// unit directo del dedup (code-review MAJOR #2).
export function mergeDevice(list: Device[], device: Device): Device[] {
    const k = deviceKey(device);
    const idx = list.findIndex((d) => deviceKey(d) === k);
    if (idx === -1) return [...list, device];
    const next = list.slice();
    next[idx] = device;
    return next;
}

const navItems = [
    {
        to: "/",
        label: "Dashboard",
        icon: LayoutDashboard,
        end: true,
        desc: "Resumen de red",
    },
    {
        to: "/devices",
        label: "Dispositivos",
        icon: Network,
        end: false,
        desc: "Inventario de dispositivos",
    },
    {
        to: "/scans",
        label: "Escaneo de puertos",
        icon: Radar,
        end: false,
        desc: "Historial y escaneo de puertos",
    },
    {
        to: "/settings",
        label: "Ajustes",
        icon: SettingsIcon,
        end: false,
        desc: "Configuración",
    },
];

function ThemeToggle() {
    const { theme, toggleTheme } = useTheme();
    const isDark = theme === "dark";
    return (
        <Button
            variant="ghost"
            size="icon"
            onClick={toggleTheme}
            aria-label={isDark ? "Activar modo claro" : "Activar modo oscuro"}
        >
            {isDark ? (
                <Sun className="h-4 w-4" aria-hidden />
            ) : (
                <Moon className="h-4 w-4" aria-hidden />
            )}
        </Button>
    );
}

function LastScanBadge() {
    const { lastScan } = useLastScan();
    if (!lastScan) return null;
    return (
        <div
            className="flex items-center gap-2 rounded-full border border-border bg-muted/50 px-3 py-1 text-xs"
            aria-label={`Último escaneo: ${lastScan.hosts_alive} dispositivos activos, ${lastScan.hosts_new} nuevos`}
        >
            <Activity
                className="h-3.5 w-3.5 text-muted-foreground"
                aria-hidden
            />
            <span className="text-muted-foreground">Último scan:</span>
            <Badge variant="success" className="px-1.5 py-0">
                {lastScan.hosts_alive} activos
            </Badge>
            <Badge variant="secondary" className="px-1.5 py-0">
                {lastScan.hosts_new} nuevos
            </Badge>
        </div>
    );
}

function Sidebar() {
    return (
        <nav aria-label="Navegación principal" className="flex flex-col gap-1">
            {navItems.map((n) => {
                const Icon = n.icon;
                return (
                    <NavLink
                        key={n.to}
                        to={n.to}
                        end={n.end}
                        className={({ isActive }) =>
                            cn(
                                "flex items-center gap-3 rounded-md border-l-2 px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                isActive
                                    ? "border-primary bg-accent text-foreground"
                                    : "border-transparent text-muted-foreground hover:bg-muted hover:text-foreground",
                            )
                        }
                    >
                        <Icon className="h-4 w-4 shrink-0" aria-hidden />
                        <span className="flex flex-col">
                            <span>{n.label}</span>
                            {n.desc && (
                                <span className="text-xs font-normal">
                                    {n.desc}
                                </span>
                            )}
                        </span>
                    </NavLink>
                );
            })}
        </nav>
    );
}

// Pie de la sidebar (AC-8): nombre de la red activa (SSID / etiqueta / CIDR,
// renderizado en claro — NUNCA enmascarado, AC-11) con un control "Editar"
// inline que persiste una etiqueta de usuario, más la versión corta de la app.
// El tag "auto"/"editado" indica el origen del nombre para que el usuario
// entienda que un re-escaneo no pisa su etiqueta (AC-3).
function SidebarFooter({ onOpenAbout }: { onOpenAbout: () => void }) {
    const { name, source, cidr, editName } = useNetworkName();
    const [version, setVersion] = useState("");
    const [editing, setEditing] = useState(false);
    const [draft, setDraft] = useState("");

    useEffect(() => {
        getAppVersion()
            .then(setVersion)
            .catch(() => {});
    }, []);

    const displayName = name || cidr || "Sin red";

    async function save() {
        const label = draft.trim();
        if (label) await editName(label);
        setEditing(false);
    }

    return (
        <div className="mt-auto flex flex-col gap-2 px-2 text-xs text-muted-foreground">
            <div className="flex flex-col gap-1">
                <span className="flex items-center gap-1.5 text-[11px] uppercase tracking-wide">
                    <Network className="h-3.5 w-3.5 shrink-0" aria-hidden />
                    Red activa
                </span>
                {editing ? (
                    <div className="flex flex-col gap-1.5">
                        <Input
                            value={draft}
                            onChange={(e) => setDraft(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === "Enter") void save();
                                if (e.key === "Escape") setEditing(false);
                            }}
                            placeholder="Nombre de la red"
                            aria-label="Nombre de la red"
                            autoFocus
                            className="h-7 text-xs"
                        />
                        <div className="flex gap-1.5">
                            <Button
                                size="sm"
                                className="h-6 px-2 text-[11px]"
                                onClick={() => void save()}
                            >
                                Guardar
                            </Button>
                            <Button
                                size="sm"
                                variant="ghost"
                                className="h-6 px-2 text-[11px]"
                                onClick={() => setEditing(false)}
                            >
                                Cancelar
                            </Button>
                        </div>
                    </div>
                ) : (
                    <>
                        <div className="flex items-center justify-between gap-2">
                            <span
                                className="truncate font-medium text-foreground"
                                title={displayName}
                            >
                                {displayName}
                            </span>
                            <Badge
                                variant="secondary"
                                className="shrink-0 px-1 py-0 text-[10px]"
                            >
                                {source === "user" ? "editado" : "auto"}
                            </Badge>
                        </div>
                        <button
                            type="button"
                            onClick={() => {
                                setDraft(name);
                                setEditing(true);
                            }}
                            disabled={!cidr}
                            className="inline-flex w-fit items-center gap-1 rounded text-[11px] hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                        >
                            <Pencil className="h-3 w-3" aria-hidden />
                            Editar
                        </button>
                    </>
                )}
            </div>
            <p className="text-[10px] text-muted-foreground">
                MyLAN v{version || "…"}
            </p>
            <button
                type="button"
                onClick={onOpenAbout}
                className="inline-flex w-fit items-center gap-1.5 rounded text-[11px] hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
                <Info className="h-3 w-3" aria-hidden />
                Acerca de
            </button>
        </div>
    );
}

function AppShell() {
    // T18/#11: estado local del dialog "Acerca de" (abierto desde SidebarFooter
    // en desktop y desde el 5º item del top-nav móvil).
    const [aboutOpen, setAboutOpen] = useState(false);
    const openAbout = () => setAboutOpen(true);

    return (
        <div className="flex h-screen overflow-hidden bg-background text-foreground">
            {/* Sidebar fija en desktop; en móvil se colapsa arriba (AC-1). */}
            <aside className="hidden h-full w-60 shrink-0 flex-col gap-6 border-r border-border bg-card p-4 md:flex">
                <div className="flex items-center gap-2 px-2 pt-2">
                    <Network className="h-6 w-6 text-primary" aria-hidden />
                    <span className="text-lg font-bold tracking-tight">
                        MyLAN
                    </span>
                </div>
                <Sidebar />
                <SidebarFooter onOpenAbout={openAbout} />
            </aside>

            <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
                <header className="z-30 flex shrink-0 items-center justify-between gap-4 border-b border-border bg-background/80 px-6 py-3 backdrop-blur">
                    <div className="flex items-center gap-3">
                        {/* #37: "MyLAN" solo en móvil (desktop tiene el brand en la sidebar). */}
                        <h1 className="text-lg font-semibold tracking-tight md:hidden">
                            MyLAN
                        </h1>
                        <LastScanBadge />
                    </div>
                    <ThemeToggle />
                </header>
                {/* Navegación móvil (sidebar arriba en pantallas pequeñas, AC-1). */}
                <nav
                    aria-label="Navegación móvil"
                    className="flex shrink-0 gap-1 overflow-x-auto px-4 py-2 md:hidden"
                >
                    {navItems.map((n) => {
                        const Icon = n.icon;
                        return (
                            <NavLink
                                key={n.to}
                                to={n.to}
                                end={n.end}
                                className={({ isActive }) =>
                                    cn(
                                        "flex items-center gap-1.5 rounded-md border-l-2 px-2.5 py-1.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                        isActive
                                            ? "border-primary bg-accent text-foreground"
                                            : "border-transparent text-muted-foreground hover:bg-muted hover:text-foreground",
                                    )
                                }
                            >
                                <Icon className="h-3.5 w-3.5" aria-hidden />
                                <span className="flex flex-col">
                                    <span>{n.label}</span>
                                    {n.desc && (
                                        <span className="text-[10px] font-normal">
                                            {n.desc}
                                        </span>
                                    )}
                                </span>
                            </NavLink>
                        );
                    })}
                    {/* #21: 5º item "Acerca de" abre dialog (no es ruta). */}
                    <button
                        type="button"
                        onClick={openAbout}
                        className="flex items-center gap-1.5 rounded-md border-l-2 border-transparent px-2.5 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                        <Info className="h-3.5 w-3.5" aria-hidden />
                        <span className="flex flex-col">
                            <span>Acerca de</span>
                            <span className="text-[10px] font-normal">
                                Información de la app
                            </span>
                        </span>
                    </button>
                </nav>
                <main className="min-h-0 flex-1 overflow-y-auto px-4 py-6 md:px-6 md:py-8">
                    <div className="mx-auto flex max-w-5xl flex-col gap-6">
                        <Routes>
                            <Route path="/" element={<Dashboard />} />
                            <Route path="/devices" element={<Devices />} />
                            <Route
                                path="/devices/:ip"
                                element={<DeviceDetail />}
                            />
                            <Route path="/scans" element={<Scans />} />
                            <Route path="/settings" element={<Settings />} />
                        </Routes>
                    </div>
                </main>
            </div>
            <AboutDialog open={aboutOpen} onOpenChange={setAboutOpen} />
        </div>
    );
}

function AppInner() {
    const { toast } = useToast();
    const [lastScan, setLastScan] = useState<ScanOutcomeDto | null>(null);
    const setLastScanCb = useCallback(
        (o: ScanOutcomeDto | null) => setLastScan(o),
        [],
    );

    const [scanning, setScanning] = useState(false);
    const [scanId, setScanId] = useState<string | null>(null);
    const [progress, setProgress] = useState<ScanProgressState | null>(null);
    const [devicesFound, setDevicesFound] = useState<Device[]>([]);
    const scanningRef = useRef(false);
    const scanIdRef = useRef<string | null>(null);
    const cancelledRef = useRef(false);

    // Suscripción ÚNICA al ciclo de vida del scan (AC-11): ambas pantallas
    // reflejan el mismo progreso/hallazgos y la navegación no lo interrumpe.
    useEffect(() => {
        const unlistens: UnlistenFn[] = [];
        let active = true;
        Promise.all([
            onScanStarted((s) => setScanId(s.scan_id)),
            onDiscoveryProgress((p) =>
                setProgress({
                    swept: Math.min(p.swept, p.total),
                    total: p.total,
                    percent:
                        p.total > 0
                            ? Math.min(
                                  100,
                                  Math.round((p.swept / p.total) * 100),
                              )
                            : null,
                }),
            ),
            onScanDevice((d) =>
                setDevicesFound((prev) => mergeDevice(prev, d.device)),
            ),
            // Éxito o salida limpia: barra a estado final (100% si total conocido).
            onScanFinished(() => {
                setScanning(false);
                setProgress((prev) =>
                    prev && prev.total > 0
                        ? { ...prev, swept: prev.total, percent: 100 }
                        : prev,
                );
            }),
            // Cancelación: conserva parciales, congela la barra y avisa (AC-8/AC-9).
            onScanCancelled(() => {
                cancelledRef.current = true;
                setScanning(false);
                toast(
                    "Escaneo cancelado. Se conservan los dispositivos ya encontrados.",
                );
            }),
        ]).then((fns) => {
            if (!active) {
                fns.forEach((f) => f());
                return;
            }
            unlistens.push(...fns);
        });
        return () => {
            active = false;
            unlistens.forEach((f) => f());
        };
    }, [toast]);

    const startScan = useCallback(
        async (profile?: string) => {
            if (scanningRef.current) return; // un único scan a la vez
            scanningRef.current = true;
            cancelledRef.current = false;
            const id = newScanId();
            scanIdRef.current = id;
            setScanId(id);
            setScanning(true);
            setProgress(null);
            setDevicesFound([]);
            try {
                const resolved =
                    profile ?? (await getSettings()).default_profile;
                const outcome = await runDiscovery(resolved, id);
                setLastScan(outcome);
                // Resumen tipo Dashboard salvo que el usuario haya cancelado.
                if (!cancelledRef.current) {
                    toast(
                        `Escaneo completado: ${outcome.hosts_alive} dispositivos activos, ${outcome.hosts_new} nuevos.`,
                        "success",
                    );
                }
            } catch (e) {
                toast(`Error: ${e}`, "error");
            } finally {
                scanningRef.current = false;
                setScanning(false);
            }
        },
        [toast],
    );

    const cancel = useCallback(() => {
        const id = scanIdRef.current;
        if (id) cancelScan(id).catch(() => {});
    }, []);

    const scanValue = useMemo<ScanContextValue>(
        () => ({ scanning, scanId, progress, devicesFound, startScan, cancel }),
        [scanning, scanId, progress, devicesFound, startScan, cancel],
    );

    return (
        <LastScanContext.Provider
            value={{ lastScan, setLastScan: setLastScanCb }}
        >
            <ScanContext.Provider value={scanValue}>
                {/* NetworkNameProvider va dentro de ScanContext para refrescar
                    el nombre tras cada scan (useNetworkName consume useScan). */}
                <NetworkNameProvider>
                    <HashRouter>
                        <AppShell />
                    </HashRouter>
                </NetworkNameProvider>
                {/* Dialog one-shot de upgrade censura (AC-4). Va dentro de
                    AppInner para que useCensorship resuelva (CensorshipProvider
                    envuelve AppInner). */}
                <CensuraUpgradeDialog />
                {/* Onboarding primera ejecución (AC-4): tour discovery vs
                    puertos. One-shot, persistido en localStorage. */}
                <OnboardingDialog />
            </ScanContext.Provider>
        </LastScanContext.Provider>
    );
}

function App() {
    return (
        <ThemeProvider>
            <CensorshipProvider>
                <TooltipProvider delayDuration={200}>
                    <ToastProviderApp>
                        <AppInner />
                    </ToastProviderApp>
                </TooltipProvider>
            </CensorshipProvider>
        </ThemeProvider>
    );
}

export default App;
