import { createContext, useCallback, useContext, useState } from "react";
import { HashRouter, NavLink, Route, Routes } from "react-router-dom";
import {
    LayoutDashboard,
    Network,
    Radar,
    Settings as SettingsIcon,
    Sun,
    Moon,
    Activity,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ToastProviderApp } from "@/components/ui/toast";
import { ThemeProvider, useTheme } from "@/components/theme-provider";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { ScanOutcomeDto } from "@/lib/tauri";
import { Dashboard } from "@/screens/Dashboard";
import { Devices } from "@/screens/Devices";
import { DeviceDetail } from "@/screens/DeviceDetail";
import { Scans } from "@/screens/Scans";
import { Settings } from "@/screens/Settings";

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
        desc: "Inventario de hosts",
    },
    {
        to: "/scans",
        label: "Scans",
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
            aria-label={`Último escaneo: ${lastScan.hosts_alive} hosts vivos, ${lastScan.hosts_new} nuevos`}
        >
            <Activity
                className="h-3.5 w-3.5 text-muted-foreground"
                aria-hidden
            />
            <span className="text-muted-foreground">Último scan:</span>
            <Badge variant="success" className="px-1.5 py-0">
                {lastScan.hosts_alive} vivos
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
                                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                isActive
                                    ? "bg-primary text-primary-foreground"
                                    : "text-muted-foreground hover:bg-muted hover:text-foreground",
                            )
                        }
                    >
                        <Icon className="h-4 w-4 shrink-0" aria-hidden />
                        <span>{n.label}</span>
                    </NavLink>
                );
            })}
        </nav>
    );
}

function AppShell() {
    return (
        <div className="flex min-h-screen bg-background text-foreground">
            {/* Sidebar fija en desktop; en móvil se colapsa arriba (AC-1). */}
            <aside className="flex w-60 shrink-0 flex-col gap-6 border-r border-border bg-card p-4 md:flex">
                <div className="flex items-center gap-2 px-2 pt-2">
                    <Network className="h-6 w-6 text-primary" aria-hidden />
                    <span className="text-lg font-bold tracking-tight">
                        MyLAN
                    </span>
                </div>
                <Sidebar />
                <div className="mt-auto px-2 text-xs text-muted-foreground">
                    <p>Desktop Alpha</p>
                </div>
            </aside>

            <div className="flex flex-1 flex-col">
                <header className="sticky top-0 z-30 flex items-center justify-between gap-4 border-b border-border bg-background/80 px-6 py-3 backdrop-blur">
                    <div className="flex items-center gap-3">
                        <h1 className="text-lg font-semibold tracking-tight">
                            MyLAN
                        </h1>
                        <LastScanBadge />
                    </div>
                    <ThemeToggle />
                </header>
                {/* Navegación móvil (sidebar arriba en pantallas pequeñas, AC-1). */}
                <nav
                    aria-label="Navegación móvil"
                    className="flex gap-1 px-4 py-2 md:hidden"
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
                                        "flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                        isActive
                                            ? "bg-primary text-primary-foreground"
                                            : "text-muted-foreground hover:bg-muted hover:text-foreground",
                                    )
                                }
                            >
                                <Icon className="h-3.5 w-3.5" aria-hidden />
                                <span>{n.label}</span>
                            </NavLink>
                        );
                    })}
                </nav>
                <main className="flex-1 px-4 py-6 md:px-6 md:py-8">
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
        </div>
    );
}

function AppInner() {
    const [lastScan, setLastScan] = useState<ScanOutcomeDto | null>(null);
    const setLastScanCb = useCallback(
        (o: ScanOutcomeDto | null) => setLastScan(o),
        [],
    );
    return (
        <LastScanContext.Provider
            value={{ lastScan, setLastScan: setLastScanCb }}
        >
            <HashRouter>
                <AppShell />
            </HashRouter>
        </LastScanContext.Provider>
    );
}

function App() {
    return (
        <ThemeProvider>
            <TooltipProvider delayDuration={200}>
                <ToastProviderApp>
                    <AppInner />
                </ToastProviderApp>
            </TooltipProvider>
        </ThemeProvider>
    );
}

export default App;
