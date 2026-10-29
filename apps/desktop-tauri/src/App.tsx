import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import {
    createHashRouter,
    NavLink,
    Outlet,
    RouterProvider,
    useLocation,
} from "react-router-dom";
import {
    Network,
    History,
    Settings as SettingsIcon,
    Info,
    Pencil,
    Wrench,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ToastProviderApp } from "@/components/ui/toast";
import { ThemeProvider, useTheme } from "@/components/theme-provider";
import { CensorshipProvider } from "@/components/censorship-provider";
import { WorkspaceProvider } from "@/components/workspace-provider";
import { WorkspaceSelector } from "@/components/workspace-selector";
import { SyncStatusIndicator } from "@/components/sync-status-indicator";
import { CensuraUpgradeDialog } from "@/components/censura-upgrade-dialog";
import { OnboardingDialog } from "@/components/onboarding-dialog";
import { UpdaterProvider, useUpdater } from "@/components/updater-provider";
import { Button } from "@/components/ui/button";
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
import { Tools } from "@/screens/Tools";
import { Settings } from "@/screens/Settings";
import { NotFound } from "@/screens/NotFound";
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

/** Identidad de merge: el `id` estable de la DB (MAC > IP se resuelve en core). */
export function deviceKey(d: Device): string {
    return d.id;
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

// ADR-0007: Dashboard y Dispositivos se unen en la vista Red. La página
// dedicada /devices (y el detalle /devices/:ip) cuelga de Red: se llega desde
// su enlace "Ver todos" y mantiene Red resaltado en la navegación.
const navItems = [
    {
        to: "/",
        label: "Red",
        icon: Network,
        end: true,
        alsoActive: "/devices",
    },
    { to: "/tools", label: "Herramientas", icon: Wrench, end: false },
    { to: "/scans", label: "Historial", icon: History, end: false },
    { to: "/settings", label: "Ajustes", icon: SettingsIcon, end: false },
];

/** Activo propio de NavLink o por prefijo `alsoActive` (subpáginas de Red). */
function useNavActive() {
    const { pathname } = useLocation();
    return (n: (typeof navItems)[number], isActive: boolean) =>
        isActive || (!!n.alsoActive && pathname.startsWith(n.alsoActive));
}

// Marca de la app: Discovery Scope (Opción C oficial), retícula táctica de escaneo
// con 4 cuadrantes simétricos, ondas de radar concéntricas en Q1 y blip de host detectado en Q3.
// - Modo oscuro (final-symbol-color.svg): Deep Void Navy (#090D16) + Electric Cyan (#06B6D4) + Sky Signal (#38BDF8).
// - Modo claro (final-symbol-light.svg): Slate Tile (#F1F5F9, borde #B5BFCE) + Deep Cyan (#0891B2) + Blip (#0284C7).
export function AppMark({ className }: { className?: string }) {
    const { theme } = useTheme();
    const isLight = theme === "light";
    const primaryColor = isLight ? "#0891B2" : "#06B6D4";
    const blipColor = isLight ? "#0284C7" : "#38BDF8";

    return (
        <svg
            viewBox="0 0 256 256"
            className={className}
            role="img"
            aria-hidden
            data-theme-variant={isLight ? "light" : "dark"}
        >
            {isLight ? (
                <rect
                    x="4"
                    y="4"
                    width="248"
                    height="248"
                    rx="54"
                    fill="#F1F5F9"
                    stroke="#B5BFCE"
                    strokeWidth="8"
                />
            ) : (
                <rect width="256" height="256" rx="56" fill="#090D16" />
            )}
            {/* Baliza de origen local central */}
            <circle cx="128" cy="128" r="22" fill={primaryColor} />

            {/* 4 cuadrantes simétricos de retícula (arcos de 72° con aperturas cardinales de 18°) */}
            <path
                fill={primaryColor}
                opacity={isLight ? "0.95" : "0.9"}
                d="M 144.58 23.31 A 106 106 0 0 1 232.69 111.42 L 216.89 113.92 A 90 90 0 0 0 142.08 39.11 Z"
            />
            <path
                fill={primaryColor}
                opacity={isLight ? "0.95" : "0.9"}
                d="M 23.31 111.42 A 106 106 0 0 1 111.42 23.31 L 113.92 39.11 A 90 90 0 0 0 39.11 113.92 Z"
            />
            <path
                fill={primaryColor}
                opacity={isLight ? "0.95" : "0.9"}
                d="M 111.42 232.69 A 106 106 0 0 1 23.31 144.58 L 39.11 142.08 A 90 90 0 0 0 113.92 216.89 Z"
            />
            <path
                fill={primaryColor}
                opacity={isLight ? "0.95" : "0.9"}
                d="M 232.69 144.58 A 106 106 0 0 1 144.58 232.69 L 142.08 216.89 A 90 90 0 0 0 216.89 142.08 Z"
            />

            {/* Ondas concéntricas de barrido de radar */}
            <path
                fill={primaryColor}
                opacity={isLight ? "0.8" : "0.75"}
                d="M 137.70 66.76 A 62 62 0 0 1 189.24 118.30 L 173.43 120.80 A 46 46 0 0 0 135.20 82.57 Z"
            />
            <path
                fill={primaryColor}
                opacity={isLight ? "0.6" : "0.5"}
                d="M 141.14 45.03 A 84 84 0 0 1 210.97 114.86 L 195.16 117.36 A 68 68 0 0 0 138.64 60.84 Z"
            />

            {/* Host detectado (blip de señal en Q3) */}
            <circle cx="89.82" cy="166.18" r="10" fill={blipColor} />
        </svg>
    );
}

function Sidebar() {
    const navActive = useNavActive();
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
                                "relative flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-[background-color,color,transform] duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring active:scale-[0.98]",
                                navActive(n, isActive)
                                    ? "bg-primary/10 text-foreground"
                                    : "text-muted-foreground hover:bg-muted hover:text-foreground",
                            )
                        }
                    >
                        {({ isActive }) => (
                            <>
                                {navActive(n, isActive) && (
                                    <span
                                        className="absolute left-0 top-1/2 h-5 w-[3px] -translate-y-1/2 rounded-full bg-primary"
                                        aria-hidden
                                    />
                                )}
                                <Icon
                                    className={cn(
                                        "h-4 w-4 shrink-0",
                                        navActive(n, isActive) &&
                                            "text-primary",
                                    )}
                                    aria-hidden
                                />
                                <span>{n.label}</span>
                            </>
                        )}
                    </NavLink>
                );
            })}
        </nav>
    );
}

// Pie de la sidebar (AC-8): panel compacto con la red activa (SSID / etiqueta
// / CIDR, renderizado en claro, NUNCA enmascarado, AC-11), edición inline que
// persiste una etiqueta de usuario, y la fila de meta (versión + Acerca de).
function SidebarFooter({ onOpenAbout }: { onOpenAbout: () => void }) {
    const { name, cidr, editName } = useNetworkName();
    const { status, updateInfo } = useUpdater();
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
        <div className="mt-auto flex flex-col gap-3">
            {/* Red activa: panel compacto. El punto verde es estado semántico
                (red detectada), no decoración. */}
            <div className="rounded-lg border bg-muted/30 p-3">
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
                    <div className="flex items-center justify-between gap-2">
                        <div className="flex min-w-0 items-center gap-2">
                            <span
                                className={cn(
                                    "h-1.5 w-1.5 shrink-0 rounded-full",
                                    cidr
                                        ? "bg-emerald-500"
                                        : "bg-muted-foreground/40",
                                )}
                                aria-hidden
                            />
                            <span
                                className="truncate text-sm font-medium"
                                title={displayName}
                            >
                                {displayName}
                            </span>
                        </div>
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                                setDraft(name);
                                setEditing(true);
                            }}
                            disabled={!cidr}
                            aria-label="Editar nombre de la red"
                            title="Editar nombre de la red"
                            className="h-6 w-6 shrink-0 text-muted-foreground hover:bg-muted hover:text-foreground"
                        >
                            <Pencil className="h-3.5 w-3.5" aria-hidden />
                        </Button>
                    </div>
                )}
            </div>
            <div className="flex items-center justify-between px-1">
                <div className="flex items-center gap-1.5">
                    <p
                        className="text-[11px] text-muted-foreground"
                        translate="no"
                    >
                        MyLAN v{version || "…"}
                    </p>
                    {status === "available" && (
                        <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={onOpenAbout}
                            className="h-auto rounded-full bg-primary/20 px-1.5 py-0.5 text-[11px] font-semibold text-primary hover:bg-primary/30"
                            title={`Nueva versión v${updateInfo?.version} disponible`}
                        >
                            Update
                        </Button>
                    )}
                    {status === "ready" && (
                        <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={onOpenAbout}
                            className="h-auto rounded-full bg-success/20 px-1.5 py-0.5 text-[11px] font-semibold text-success-strong hover:bg-success/30"
                            title="Actualización lista para reiniciar"
                        >
                            Reiniciar
                        </Button>
                    )}
                </div>
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={onOpenAbout}
                    className="h-auto p-1 gap-1 text-[11px] text-muted-foreground hover:text-foreground"
                >
                    <Info className="h-3 w-3" aria-hidden />
                    Acerca de
                </Button>
            </div>
        </div>
    );
}

function AppShell() {
    // T18/#11: estado local del dialog "Acerca de" (abierto desde SidebarFooter
    // en desktop y desde el 5º item del top-nav móvil).
    const [aboutOpen, setAboutOpen] = useState(false);
    const openAbout = () => setAboutOpen(true);
    const navActive = useNavActive();

    return (
        <div className="flex h-screen overflow-hidden bg-background text-foreground">
            {/* Sidebar fija en desktop; en móvil se colapsa arriba (AC-1). */}
            <aside className="hidden h-full w-60 shrink-0 flex-col gap-4 border-r border-border bg-card p-4 md:flex">
                <div className="flex items-center gap-2.5 px-1 pt-1">
                    <AppMark className="h-8 w-8 shrink-0" />
                    <span
                        className="text-lg font-bold tracking-tight"
                        translate="no"
                    >
                        MyLAN
                    </span>
                </div>
                <div className="flex flex-col gap-2">
                    <WorkspaceSelector />
                    <SyncStatusIndicator className="w-full justify-start" showLabel />
                </div>
                <Sidebar />
                <SidebarFooter onOpenAbout={openAbout} />
            </aside>

            <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
                {/* #37: barra superior solo en móvil (desktop tiene el brand en
                    la sidebar). El resumen del último descubrimiento vive en la
                    cabecera de la vista Red (ADR-0007). */}
                <header className="z-30 flex shrink-0 items-center justify-between gap-3 border-b border-border bg-background/80 px-4 py-2.5 backdrop-blur md:hidden">
                    <div className="flex items-center gap-2">
                        <AppMark className="h-6 w-6 shrink-0" />
                        <span
                            className="text-base font-semibold tracking-tight"
                            translate="no"
                            aria-hidden
                        >
                            MyLAN
                        </span>
                    </div>
                    <div className="flex items-center gap-2">
                        <SyncStatusIndicator />
                        <div className="w-36 sm:w-44">
                            <WorkspaceSelector />
                        </div>
                    </div>
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
                                        "flex min-h-[44px] items-center gap-1.5 rounded-md px-2.5 py-2.5 text-xs font-medium touch-manipulation transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                        navActive(n, isActive)
                                            ? "bg-primary/10 text-foreground"
                                            : "text-muted-foreground hover:bg-muted hover:text-foreground",
                                    )
                                }
                            >
                                <Icon className="h-3.5 w-3.5" aria-hidden />
                                <span>{n.label}</span>
                            </NavLink>
                        );
                    })}
                    {/* #21: 5º item "Acerca de" abre dialog (no es ruta). */}
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={openAbout}
                        className="h-auto min-h-[44px] gap-1.5 px-2.5 py-2.5 text-xs font-medium touch-manipulation text-muted-foreground hover:bg-muted hover:text-foreground"
                    >
                        <Info className="h-3.5 w-3.5" aria-hidden />
                        <span>Acerca de</span>
                    </Button>
                </nav>
                <a
                    href="#main-content"
                    className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-[100] focus:rounded-md focus:bg-background focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:shadow-lg focus:border focus:border-border"
                >
                    Saltar al contenido principal
                </a>
                <main
                    id="main-content"
                    className="min-h-0 flex-1 overflow-y-auto px-6 py-6 md:px-8 md:py-6"
                >
                    <div className="mx-auto flex w-full max-w-7xl 2xl:max-w-screen-2xl flex-col gap-6">
                        <Outlet />
                    </div>
                </main>
            </div>
            <AboutDialog open={aboutOpen} onOpenChange={setAboutOpen} />
        </div>
    );
}

// Router de datos (react-router v7): useBlocker en Settings y DeviceDetail
// (aviso de cambios sin guardar) requiere un data router; AppShell es el
// layout raíz y las pantallas se renderizan en su <Outlet />.
const router = createHashRouter([
    {
        element: <AppShell />,
        children: [
            { path: "/", element: <Dashboard /> },
            { path: "/devices", element: <Devices /> },
            { path: "/devices/:ip", element: <DeviceDetail /> },
            { path: "/tools", element: <Tools /> },
            { path: "/scans", element: <Scans /> },
            { path: "/settings", element: <Settings /> },
            { path: "*", element: <NotFound /> },
        ],
    },
]);

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
            onScanStarted((s) => {
                if (s.scan_id === scanIdRef.current) setScanId(s.scan_id);
            }),
            onDiscoveryProgress((p) => {
                if (p.scan_id !== scanIdRef.current) return;
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
                });
            }),
            onScanDevice((d) => {
                if (d.scan_id !== scanIdRef.current) return;
                setDevicesFound((prev) => mergeDevice(prev, d.device));
            }),
            // Éxito o salida limpia: barra a estado final (100% si total conocido).
            // Filtrar por scan_id evita que un port-scan cancele la UI del descubrimiento.
            onScanFinished((f) => {
                if (f.scan_id !== scanIdRef.current) return;
                setScanning(false);
                setProgress((prev) =>
                    prev && prev.total > 0
                        ? { ...prev, swept: prev.total, percent: 100 }
                        : prev,
                );
            }),
            // Cancelación: conserva parciales, congela la barra y avisa (AC-8/AC-9).
            onScanCancelled((c) => {
                if (c.scan_id !== scanIdRef.current) return;
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
                    <RouterProvider router={router} />
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
                <WorkspaceProvider>
                    <TooltipProvider delayDuration={200}>
                        <ToastProviderApp>
                            <UpdaterProvider>
                                <AppInner />
                            </UpdaterProvider>
                        </ToastProviderApp>
                    </TooltipProvider>
                </WorkspaceProvider>
            </CensorshipProvider>
        </ThemeProvider>
    );
}

export default App;
