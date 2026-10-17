import { useCallback, useEffect, useState } from "react";
import { useBlocker, useBeforeUnload } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useToast } from "@/components/ui/toast";
import { useTheme } from "@/components/theme-provider";
import { useCensorship } from "@/components/censorship-provider";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { reopenOnboarding } from "@/components/onboarding-dialog";
import {
    Dialog,
    DialogContent,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import {
    Sun,
    Moon,
    Database,
    Gauge,
    Palette,
    EyeOff,
    Loader2,
    RotateCcw,
    RotateCw,
    RefreshCw,
    Download,
    CheckCircle2,
    AlertCircle,
    Activity,
    Clock,
    Minimize2,
    Power,
    Play,
    Pause,
    Bell,
    Cloud,
    ShieldCheck,
} from "lucide-react";
import { useUpdater } from "@/components/updater-provider";
import { PROFILES } from "@/lib/profiles";
import { GdriveSyncDialog } from "@/components/gdrive-sync-dialog";
import { useWorkspace } from "@/components/workspace-provider";
import {
    dbPath,
    getSettings,
    setSettings,
    getMonitoringStatus,
    toggleMonitoringPause,
    triggerBackgroundScan,
    onMonitoringStatus,
    getGdriveSyncStatus,
    syncWorkspace,
    type GdriveSyncStatus,
    type MonitoringStatusDto,
    type Settings as SettingsDto,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";

const CENSORED_FIELDS = [
    { label: "IP", glossaryKey: "ip" },
    { label: "MAC", glossaryKey: "mac" },
    { label: "Hostname", glossaryKey: "hostname" },
    { label: "Gateway", glossaryKey: "gateway" },
    { label: "DNS", glossaryKey: "dns" },
] as const;

const MONITORING_INTERVALS = [
    { label: "1 min", value: 60 },
    { label: "2 min", value: 120 },
    { label: "5 min", value: 300, recommended: true },
    { label: "10 min", value: 600 },
    { label: "15 min", value: 900 },
    { label: "30 min", value: 1800 },
] as const;

export function Settings() {
    const { toast } = useToast();
    const { theme, setTheme } = useTheme();
    const { censorshipEnabled, setCensorshipEnabled } = useCensorship();
    const [settings, setSettingsState] = useState<SettingsDto | null>(null);
    const [dbPathValue, setDbPathValue] = useState("");
    const [saving, setSaving] = useState(false);
    const [monitoringStatus, setMonitoringStatus] = useState<MonitoringStatusDto | null>(null);

    const { activeWorkspace } = useWorkspace();
    const [gdriveStatus, setGdriveStatus] = useState<GdriveSyncStatus | null>(null);
    const [syncDialogOpen, setSyncDialogOpen] = useState(false);
    const [isSyncingNow, setIsSyncingNow] = useState(false);

    const {
        status,
        updateInfo,
        progress,
        error: updateError,
        checkForUpdates,
        downloadAndInstall,
        restartApp,
    } = useUpdater();

    // Aviso de cambios sin guardar: db_path modificado pero no guardado.
    // El resto de preferencias se aplican y persisten al instante.
    const dbPathDirty = settings != null && dbPathValue !== settings.db_path;
    const blocker = useBlocker(dbPathDirty);
    useBeforeUnload(
        useCallback(
            (e: BeforeUnloadEvent) => {
                if (dbPathDirty) {
                    e.preventDefault();
                    e.returnValue = "";
                }
            },
            [dbPathDirty],
        ),
    );

    useEffect(() => {
        Promise.all([getSettings(), dbPath(), getGdriveSyncStatus().catch(() => null)])
            .then(([s, p, g]) => {
                setSettingsState(s);
                setDbPathValue(p || s.db_path);
                if (g) setGdriveStatus(g);
            })
            .catch((e) => toast(`Error: ${e}`, "error"));

        getMonitoringStatus()
            .then(setMonitoringStatus)
            .catch(() => {});

        let unlisten: (() => void) | undefined;
        onMonitoringStatus((s) => {
            setMonitoringStatus(s);
        }).then((u) => {
            unlisten = u;
        });

        return () => {
            if (unlisten) unlisten();
        };
    }, [toast]);

    async function persistPatch(patch: Partial<SettingsDto>) {
        if (!settings) return;
        const next: SettingsDto = {
            ...settings,
            theme,
            censorship_enabled: censorshipEnabled,
            ...patch,
        };
        setSettingsState(next);
        try {
            await setSettings(next);
        } catch (e) {
            toast(`Error guardando: ${e}`, "error");
        }
    }

    async function handleSave() {
        if (!settings) return;
        setSaving(true);
        try {
            const next: SettingsDto = {
                ...settings,
                db_path: dbPathValue,
                theme,
                censorship_enabled: censorshipEnabled,
            };
            await setSettings(next);
            setSettingsState(next);
            toast("Ajustes guardados.", "success");
        } catch (e) {
            toast(`Error guardando: ${e}`, "error");
        } finally {
            setSaving(false);
        }
    }

    async function handleManualSync() {
        if (!activeWorkspace) return;
        setIsSyncingNow(true);
        try {
            const outcome = await syncWorkspace(activeWorkspace.id);
            if (outcome.status === "success") {
                toast(`Sincronización exitosa: ${outcome.action} (${outcome.devices_synced} dispositivos).`, "success");
                const s = await getGdriveSyncStatus();
                setGdriveStatus(s);
            } else if (outcome.status === "conflict") {
                toast("Conflicto de sincronización detectado. Abre el asistente para resolverlo.", "warning");
                setSyncDialogOpen(true);
            } else {
                toast(outcome.message, "info");
            }
        } catch (e) {
            toast(`Error sincronizando: ${e}`, "error");
        } finally {
            setIsSyncingNow(false);
        }
    }

    async function handleTogglePause() {
        try {
            const paused = await toggleMonitoringPause();
            toast(paused ? "Monitoreo en pausa." : "Monitoreo reanudado.", "info");
            const s = await getMonitoringStatus();
            setMonitoringStatus(s);
        } catch (e) {
            toast(`Error: ${e}`, "error");
        }
    }

    async function handleTriggerScan() {
        try {
            await triggerBackgroundScan();
            toast("Escaneo en segundo plano iniciado.", "info");
        } catch (e) {
            toast(`Error: ${e}`, "error");
        }
    }

    if (!settings)
        return (
            <div className="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                Cargando ajustes…
            </div>
        );

    return (
        <div className={cn("flex flex-col", SECTION_GAP)}>
            {/* Cabecera de página compacta con acceso rápido al tour */}
            <div className="flex flex-wrap items-center justify-between gap-4">
                <div className="flex flex-col gap-0.5">
                    <h2 className="text-xl font-bold tracking-tight">
                        Ajustes
                    </h2>
                    <p className="text-sm text-muted-foreground">
                        Configuración de descubrimiento, privacidad, interfaz y
                        almacenamiento local de{" "}
                        <span translate="no">MyLAN</span>.
                    </p>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => reopenOnboarding()}
                    className="gap-1.5 text-xs"
                >
                    <RotateCcw className="h-3.5 w-3.5" aria-hidden />
                    Ver de nuevo el tour de bienvenida
                </Button>
            </div>

            {/* Rejilla de banco de trabajo en 2 columnas para escritorio */}
            <div className="grid gap-4 lg:grid-cols-12 lg:items-start">
                {/* Columna izquierda (7/12): Descubrimiento + Almacenamiento SQLite */}
                <div className="flex flex-col gap-4 lg:col-span-7">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Gauge
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Descubrimiento por defecto
                            </CardTitle>
                            <CardDescription>
                                Perfil inicial preseleccionado en la vista Red.
                                Se guarda automáticamente al elegirlo.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="p-5 pt-2">
                            <div
                                role="radiogroup"
                                aria-label="Perfil de descubrimiento por defecto"
                                className="grid gap-2.5 sm:grid-cols-2"
                            >
                                {PROFILES.map((p) => {
                                    const active =
                                        settings.default_profile === p.value;
                                    return (
                                        <Button
                                            key={p.value}
                                            type="button"
                                            variant="ghost"
                                            role="radio"
                                            aria-checked={active}
                                            onClick={() =>
                                                void persistPatch({
                                                    default_profile: p.value,
                                                })
                                            }
                                            className={cn(
                                                "group h-full w-full flex flex-col items-stretch justify-between gap-2.5 rounded-lg border p-3.5 text-left transition-colors whitespace-normal font-normal",
                                                active
                                                    ? "border-primary/60 bg-primary/10 hover:bg-primary/15"
                                                    : "border-border bg-background/40 hover:border-border hover:bg-muted/40",
                                            )}
                                        >
                                            <div className="flex w-full flex-col gap-1.5 text-left">
                                                <div className="flex w-full items-center justify-between gap-2">
                                                    <span className="text-sm font-semibold text-foreground">
                                                        {p.label}
                                                    </span>
                                                    <span
                                                        aria-hidden
                                                        className={cn(
                                                            "flex h-4 w-4 shrink-0 items-center justify-center rounded-full border transition-colors",
                                                            active
                                                                ? "border-primary bg-primary/20"
                                                                : "border-muted-foreground/40",
                                                        )}
                                                    >
                                                        {active && (
                                                            <span className="h-2 w-2 rounded-full bg-primary" />
                                                        )}
                                                    </span>
                                                </div>
                                                <p className="min-h-[2.25rem] text-xs leading-relaxed text-muted-foreground text-left">
                                                    {p.description}
                                                </p>
                                            </div>
                                            <div
                                                className={cn(
                                                    "flex min-h-[2rem] w-full items-start border-t pt-2 text-left text-[11px] text-muted-foreground/90 transition-colors",
                                                    active
                                                        ? "border-primary/25 text-foreground/80"
                                                        : "border-border",
                                                )}
                                            >
                                                <span className="leading-snug">{p.tradeoff}</span>
                                            </div>
                                        </Button>
                                    );
                                })}
                            </div>
                        </CardContent>
                    </Card>

                    {/* Monitoreo continuo 24/7 y bandeja (Fase 5: Desktop Watch) */}
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Activity
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Monitoreo continuo 24/7 y bandeja
                            </CardTitle>
                            <CardDescription>
                                Supervisa la red en segundo plano para alertarte de cambios y nuevos
                                dispositivos sin necesidad de mantener la ventana abierta.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-4 p-5 pt-2">
                            {/* Switch Maestro */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium">
                                        Monitoreo en segundo plano
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Ejecuta barridos periódicos rápidos (ARP/ping) para vigilar la presencia de hosts.
                                    </span>
                                </div>
                                <Switch
                                    checked={settings.background_monitoring_enabled ?? true}
                                    onCheckedChange={(checked) =>
                                        void persistPatch({
                                            background_monitoring_enabled: checked,
                                        })
                                    }
                                    aria-label="Activar monitoreo en segundo plano"
                                />
                            </div>

                            {/* Frecuencia de escaneo */}
                            {(settings.background_monitoring_enabled ?? true) && (
                                <div className="flex flex-col gap-2 rounded-lg border bg-background/40 p-3.5">
                                    <div className="flex items-center justify-between gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <Clock className="h-3.5 w-3.5 text-muted-foreground" aria-hidden />
                                            <span className="text-xs font-medium text-foreground">
                                                Frecuencia de escaneo
                                            </span>
                                        </div>
                                        <span className="text-[11px] text-muted-foreground">
                                            Perfil rápido (Quick)
                                        </span>
                                    </div>
                                    <div
                                        role="radiogroup"
                                        aria-label="Intervalo de monitoreo"
                                        className="grid grid-cols-3 gap-2 sm:grid-cols-6"
                                    >
                                        {MONITORING_INTERVALS.map((item) => {
                                            const active =
                                                (settings.monitoring_interval_secs ?? 300) === item.value;
                                            return (
                                                <Button
                                                    key={item.value}
                                                    type="button"
                                                    variant="ghost"
                                                    role="radio"
                                                    aria-checked={active}
                                                    onClick={() =>
                                                        void persistPatch({
                                                            monitoring_interval_secs: item.value,
                                                        })
                                                    }
                                                    className={cn(
                                                        "h-auto flex flex-col items-center justify-center rounded-md border py-2 px-1 text-center text-xs transition-colors whitespace-normal font-normal",
                                                        active
                                                            ? "border-primary bg-primary/15 font-semibold text-primary hover:bg-primary/20 hover:text-primary"
                                                            : "border-border bg-background/60 hover:border-border hover:bg-muted/40 text-muted-foreground",
                                                    )}
                                                >
                                                    <span>{item.label}</span>
                                                    {"recommended" in item && (
                                                        <span className="text-[9px] text-primary/80">Rec.</span>
                                                    )}
                                                </Button>
                                            );
                                        })}
                                    </div>
                                </div>
                            )}

                            {/* Minimizar a bandeja */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="flex items-center gap-1.5 text-sm font-medium">
                                        <Minimize2 className="h-3.5 w-3.5 text-primary" aria-hidden />
                                        Minimizar a la bandeja al cerrar
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Al pulsar el botón cerrar (X), MyLAN continúa activo en el System Tray.
                                    </span>
                                </div>
                                <Switch
                                    checked={settings.minimize_to_tray ?? true}
                                    onCheckedChange={(checked) =>
                                        void persistPatch({
                                            minimize_to_tray: checked,
                                        })
                                    }
                                    aria-label="Minimizar a la bandeja al cerrar"
                                />
                            </div>

                            {/* Autoarranque con el SO */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="flex items-center gap-1.5 text-sm font-medium">
                                        <Power className="h-3.5 w-3.5 text-primary" aria-hidden />
                                        Iniciar con el sistema operativo
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Arranca MyLAN en segundo plano minimizado en la bandeja al encender el equipo.
                                    </span>
                                </div>
                                <Switch
                                    checked={settings.autostart_enabled ?? false}
                                    onCheckedChange={(checked) =>
                                        void persistPatch({
                                            autostart_enabled: checked,
                                        })
                                    }
                                    aria-label="Iniciar automáticamente con el sistema operativo"
                                />
                            </div>

                            {/* Panel de estado en vivo */}
                            {monitoringStatus && (
                                <div className="flex flex-col gap-2.5 rounded-lg border border-border bg-muted/20 p-3.5">
                                    <div className="flex items-center justify-between gap-2">
                                        <div className="flex items-center gap-2">
                                            <span
                                                className={cn(
                                                    "h-2 w-2 rounded-full",
                                                    !monitoringStatus.enabled
                                                        ? "bg-muted-foreground"
                                                        : monitoringStatus.paused
                                                        ? "bg-amber-500 animate-pulse"
                                                        : monitoringStatus.is_scanning
                                                        ? "bg-blue-500 animate-ping"
                                                        : "bg-emerald-500",
                                                )}
                                                aria-hidden
                                            />
                                            <span className="text-xs font-semibold">
                                                {!monitoringStatus.enabled
                                                    ? "Monitoreo desactivado"
                                                    : monitoringStatus.paused
                                                    ? "Monitoreo en pausa"
                                                    : monitoringStatus.is_scanning
                                                    ? "Escaneo en curso…"
                                                    : "Monitoreando activamente"}
                                            </span>
                                        </div>
                                        {monitoringStatus.current_network && (
                                            <span className="rounded bg-background px-2 py-0.5 font-mono text-[11px] text-muted-foreground border">
                                                {monitoringStatus.current_network}
                                            </span>
                                        )}
                                    </div>

                                    <div className="flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground border-t border-border pt-2">
                                        <span>
                                            {monitoringStatus.last_scan_at
                                                ? `Último análisis: ${new Date(monitoringStatus.last_scan_at).toLocaleTimeString()}`
                                                : "Aún no ha escaneado"}
                                        </span>
                                        {monitoringStatus.next_scan_in_secs != null && (
                                            <span>
                                                Próximo en: ~{Math.max(1, Math.ceil(monitoringStatus.next_scan_in_secs / 60))} min
                                            </span>
                                        )}
                                    </div>

                                    <div className="flex items-center justify-end gap-2 pt-1">
                                        <Button
                                            type="button"
                                            variant="outline"
                                            size="sm"
                                            onClick={handleTogglePause}
                                            className="h-7 text-xs gap-1"
                                        >
                                            {monitoringStatus.paused ? (
                                                <>
                                                    <Play className="h-3 w-3" aria-hidden />
                                                    Reanudar
                                                </>
                                            ) : (
                                                <>
                                                    <Pause className="h-3 w-3" aria-hidden />
                                                    Pausar
                                                </>
                                            )}
                                        </Button>
                                        <Button
                                            type="button"
                                            variant="secondary"
                                            size="sm"
                                            disabled={monitoringStatus.is_scanning || !monitoringStatus.enabled}
                                            onClick={handleTriggerScan}
                                            className="h-7 text-xs gap-1"
                                        >
                                            {monitoringStatus.is_scanning ? (
                                                <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
                                            ) : (
                                                <RotateCw className="h-3 w-3" aria-hidden />
                                            )}
                                            Escanear ahora
                                        </Button>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Bell
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Notificaciones de escritorio
                            </CardTitle>
                            <CardDescription>
                                Alertas nativas del sistema operativo ante eventos críticos o dispositivos inesperados en la red.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-4 p-5 pt-2">
                            {/* Toggle Maestro */}
                            <div className="flex items-start justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span id="notifications-master-label" className="text-sm font-medium">
                                        Habilitar notificaciones del SO
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Envía avisos al centro de notificaciones nativo del sistema.
                                    </span>
                                </div>
                                <Switch
                                    id="notifications-master-switch"
                                    checked={settings.notifications_enabled ?? true}
                                    onCheckedChange={(checked) => persistPatch({ notifications_enabled: checked })}
                                    aria-labelledby="notifications-master-label"
                                />
                            </div>

                            {/* Sub-toggles (activos solo si notifications_enabled es true) */}
                            {(settings.notifications_enabled ?? true) && (
                                <div className="flex flex-col gap-3 rounded-lg border bg-muted/20 p-3.5">
                                    <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                        Disparadores de alerta
                                    </span>

                                    {/* Dispositivos no confiables */}
                                    <div className="flex items-center justify-between gap-4">
                                        <div className="flex flex-col gap-0.5">
                                            <span id="notify-untrusted-label" className="text-xs font-medium">
                                                Dispositivos no reconocidos
                                            </span>
                                            <span className="text-[11px] text-muted-foreground">
                                                Avisa cuando se detecte un host nuevo sin etiqueta de Confiable.
                                            </span>
                                        </div>
                                        <Switch
                                            id="notify-untrusted-switch"
                                            checked={settings.notify_untrusted_devices ?? true}
                                            onCheckedChange={(checked) => persistPatch({ notify_untrusted_devices: checked })}
                                            aria-labelledby="notify-untrusted-label"
                                        />
                                    </div>

                                    {/* Cambio de IP/identidad en confiables */}
                                    <div className="flex items-center justify-between gap-4 border-t border-border pt-2.5">
                                        <div className="flex flex-col gap-0.5">
                                            <span id="notify-tampered-label" className="text-xs font-medium">
                                                Cambio de IP en equipo confiable
                                            </span>
                                            <span className="text-[11px] text-muted-foreground">
                                                Alerta ante posibles suplantaciones ARP o cambios de dirección IP.
                                            </span>
                                        </div>
                                        <Switch
                                            id="notify-tampered-switch"
                                            checked={settings.notify_trusted_device_tampered ?? true}
                                            onCheckedChange={(checked) => persistPatch({ notify_trusted_device_tampered: checked })}
                                            aria-labelledby="notify-tampered-label"
                                        />
                                    </div>

                                    {/* Caída o reconexión de Gateway */}
                                    <div className="flex items-center justify-between gap-4 border-t border-border pt-2.5">
                                        <div className="flex flex-col gap-0.5">
                                            <span id="notify-gateway-label" className="text-xs font-medium">
                                                Estado del gateway / conexión
                                            </span>
                                            <span className="text-[11px] text-muted-foreground">
                                                Avisa si el router o la puerta de enlace dejan de responder.
                                            </span>
                                        </div>
                                        <Switch
                                            id="notify-gateway-switch"
                                            checked={settings.notify_gateway_down ?? true}
                                            onCheckedChange={(checked) => persistPatch({ notify_gateway_down: checked })}
                                            aria-labelledby="notify-gateway-label"
                                        />
                                    </div>

                                    {/* Puertos críticos */}
                                    <div className="flex items-center justify-between gap-4 border-t border-border pt-2.5">
                                        <div className="flex flex-col gap-0.5">
                                            <span id="notify-ports-label" className="text-xs font-medium">
                                                Puertos críticos recién abiertos
                                            </span>
                                            <span className="text-[11px] text-muted-foreground">
                                                Detecta servicios sensibles expuestos (FTP 21, Telnet 23, SMB 445, RDP 3389).
                                            </span>
                                        </div>
                                        <Switch
                                            id="notify-ports-switch"
                                            checked={settings.notify_critical_ports ?? true}
                                            onCheckedChange={(checked) => persistPatch({ notify_critical_ports: checked })}
                                            aria-labelledby="notify-ports-label"
                                        />
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Database
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Almacenamiento local
                            </CardTitle>
                            <CardDescription>
                                Ubicación del archivo SQLite donde se guarda el
                                inventario e historial de tu red (100% local).
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-3 p-5 pt-2">
                            <div className="flex flex-col gap-1.5">
                                <div className="flex items-center justify-between gap-2">
                                    <label
                                        htmlFor="db-path"
                                        className="text-xs font-medium text-muted-foreground"
                                    >
                                        Ruta de la base de datos
                                    </label>
                                    {dbPathDirty && (
                                        <span className="text-[11px] font-medium text-amber-600 dark:text-amber-400">
                                            Cambios sin guardar
                                        </span>
                                    )}
                                </div>
                                <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                                    <Input
                                        id="db-path"
                                        value={dbPathValue}
                                        onChange={(e) =>
                                            setDbPathValue(e.target.value)
                                        }
                                        className="flex-1 font-mono text-xs"
                                        name="db-path"
                                        autoComplete="off"
                                        spellCheck={false}
                                    />
                                    <div className="flex shrink-0 items-center gap-2">
                                        {dbPathDirty && (
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                size="sm"
                                                onClick={() =>
                                                    setDbPathValue(
                                                        settings.db_path,
                                                    )
                                                }
                                            >
                                                Restaurar
                                            </Button>
                                        )}
                                        <Button
                                            onClick={handleSave}
                                            disabled={saving}
                                            size="sm"
                                            className="gap-1.5"
                                        >
                                            {saving ? (
                                                <>
                                                    <Loader2
                                                        className="h-3.5 w-3.5 animate-spin"
                                                        aria-hidden
                                                    />
                                                    Guardando…
                                                </>
                                            ) : (
                                                "Guardar"
                                            )}
                                        </Button>
                                    </div>
                                </div>
                                <p className="text-xs text-muted-foreground">
                                    Modifícala solo si deseas mover o compartir
                                    el archivo de base de datos en otra carpeta.
                                </p>
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* Columna derecha (5/12): Apariencia, Privacidad y Actualizaciones */}
                <div className="flex flex-col gap-4 lg:col-span-5">
                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Palette
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Apariencia y privacidad
                            </CardTitle>
                            <CardDescription>
                                Ajustes visuales inmediatos para sesiones de
                                diagnóstico y capturas seguras.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-4 p-5 pt-2">
                            {/* Tema */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium">
                                        Tema de la interfaz
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Se aplica y guarda al instante.
                                    </span>
                                </div>
                                <div
                                    className="flex shrink-0 rounded-md border border-border p-0.5"
                                    role="group"
                                    aria-label="Selector de tema"
                                >
                                    <Button
                                        variant={
                                            theme === "light"
                                                ? "secondary"
                                                : "ghost"
                                        }
                                        size="sm"
                                        onClick={() => setTheme("light")}
                                        aria-pressed={theme === "light"}
                                        className="h-7 gap-1.5 px-2.5 text-xs"
                                    >
                                        <Sun
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Claro
                                    </Button>
                                    <Button
                                        variant={
                                            theme === "dark"
                                                ? "secondary"
                                                : "ghost"
                                        }
                                        size="sm"
                                        onClick={() => setTheme("dark")}
                                        aria-pressed={theme === "dark"}
                                        className="h-7 gap-1.5 px-2.5 text-xs"
                                    >
                                        <Moon
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Oscuro
                                    </Button>
                                </div>
                            </div>

                            {/* Modo censura */}
                            <div className="flex flex-col gap-3 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex items-start justify-between gap-4">
                                    <div className="flex flex-col gap-0.5">
                                        <span
                                            id="censura-label"
                                            className="flex items-center gap-1.5 text-sm font-medium"
                                        >
                                            <EyeOff
                                                className="h-3.5 w-3.5 text-primary"
                                                aria-hidden
                                            />
                                            Censura
                                        </span>
                                        <span className="text-xs text-muted-foreground">
                                            Enmascara datos sensibles en la
                                            interfaz y archivos exportados para
                                            compartir capturas sin exponer tu
                                            red.
                                        </span>
                                    </div>
                                    <Switch
                                        checked={censorshipEnabled}
                                        onCheckedChange={setCensorshipEnabled}
                                        aria-labelledby="censura-label"
                                    />
                                </div>

                                <div className="flex flex-wrap items-center gap-1.5 border-t border-border pt-2.5">
                                    <span className="mr-1 text-[11px] text-muted-foreground">
                                        Campos protegidos:
                                    </span>
                                    {CENSORED_FIELDS.map((f) => (
                                        <span
                                            key={f.glossaryKey}
                                            className="inline-flex items-center gap-1 rounded border border-border bg-muted/40 px-2 py-0.5 font-mono text-[11px] text-foreground"
                                        >
                                            {f.label}
                                            <InfoTooltip
                                                term={f.label}
                                                glossaryKey={f.glossaryKey}
                                            />
                                        </span>
                                    ))}
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <Cloud className="h-4 w-4 text-primary" aria-hidden />
                                Sincronización en la Nube (Google Drive)
                            </CardTitle>
                            <CardDescription>
                                Cifrado Zero-Knowledge (Argon2id + XChaCha20-Poly1305). Tus datos nunca son legibles para Google ni terceros.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-3 p-5 pt-2">
                            {/* Estado de Cuenta */}
                            <div className="flex items-center justify-between gap-3 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium">
                                        Cuenta de Google
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        {gdriveStatus?.is_authenticated
                                            ? (gdriveStatus.email || "Conectado")
                                            : "No conectado"}
                                    </span>
                                </div>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => setSyncDialogOpen(true)}
                                    className="h-7 text-xs gap-1.5"
                                >
                                    {gdriveStatus?.is_authenticated ? "Administrar" : "Conectar"}
                                </Button>
                            </div>

                            {/* Clave Maestra Zero Knowledge */}
                            <div className="flex items-center justify-between gap-3 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium flex items-center gap-1.5">
                                        <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />
                                        Clave Maestra Zero-Knowledge
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        {gdriveStatus?.has_master_key
                                            ? "Clave activa protegida en el llavero del sistema"
                                            : "Pendiente de configuración"}
                                    </span>
                                </div>
                                {!gdriveStatus?.has_master_key && gdriveStatus?.is_authenticated && (
                                    <Button
                                        variant="secondary"
                                        size="sm"
                                        onClick={() => setSyncDialogOpen(true)}
                                        className="h-7 text-xs gap-1"
                                    >
                                        Configurar
                                    </Button>
                                )}
                            </div>

                            {/* Sincronización periódica automática */}
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium">
                                        Sincronización en segundo plano
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Sincronizar cambios de forma automática periódicamente.
                                    </span>
                                </div>
                                <Switch
                                    checked={settings.gdrive_sync_enabled ?? false}
                                    disabled={!gdriveStatus?.is_authenticated || !gdriveStatus?.has_master_key}
                                    onCheckedChange={(checked) =>
                                        void persistPatch({ gdrive_sync_enabled: checked })
                                    }
                                    aria-label="Activar sincronización periódica en segundo plano"
                                />
                            </div>

                            {/* Intervalo de sincronización */}
                            {settings.gdrive_sync_enabled && (
                                <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                    <div className="flex flex-col gap-0.5">
                                        <span className="text-sm font-medium">
                                            Intervalo de sincronización
                                        </span>
                                        <span className="text-xs text-muted-foreground">
                                            Frecuencia de sincronización de snapshots cifrados.
                                        </span>
                                    </div>
                                    <div className="flex items-center gap-1">
                                        {[15, 30, 60].map((mins) => {
                                            const active = (settings.gdrive_sync_interval_mins ?? 30) === mins;
                                            return (
                                                <Button
                                                    key={mins}
                                                    variant={active ? "secondary" : "ghost"}
                                                    size="sm"
                                                    onClick={() => void persistPatch({ gdrive_sync_interval_mins: mins })}
                                                    className="h-7 px-2 text-xs"
                                                >
                                                    {mins}m
                                                </Button>
                                            );
                                        })}
                                    </div>
                                </div>
                            )}

                            {/* Última sincronización y botón manual */}
                            {gdriveStatus?.is_authenticated && gdriveStatus?.has_master_key && (
                                <div className="flex items-center justify-between gap-3 pt-1">
                                    <span className="text-[11px] text-muted-foreground truncate">
                                        Última sync: {gdriveStatus.last_synced_at
                                            ? new Date(gdriveStatus.last_synced_at).toLocaleString()
                                            : "Nunca"}
                                    </span>
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        disabled={isSyncingNow}
                                        onClick={handleManualSync}
                                        className="h-7 text-xs gap-1.5 shrink-0"
                                    >
                                        <RefreshCw
                                            className={cn("h-3 w-3", isSyncingNow && "animate-spin")}
                                        />
                                        Sincronizar ahora
                                    </Button>
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="p-5 pb-3">
                            <CardTitle className="flex items-center gap-2 text-base">
                                <RefreshCw
                                    className="h-4 w-4 text-primary"
                                    aria-hidden
                                />
                                Actualizaciones de la aplicación
                            </CardTitle>
                            <CardDescription>
                                Comprueba e instala nuevas versiones desde
                                GitHub Releases sin descargas manuales.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-3 p-5 pt-2">
                            <div className="flex items-center justify-between gap-4 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-sm font-medium">
                                        Buscar al iniciar
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        Comprobar en segundo plano al abrir{" "}
                                        <span translate="no">MyLAN</span>.
                                    </span>
                                </div>
                                <Switch
                                    checked={
                                        settings.check_updates_on_startup ??
                                        true
                                    }
                                    onCheckedChange={(checked) =>
                                        void persistPatch({
                                            check_updates_on_startup: checked,
                                        })
                                    }
                                    aria-label="Buscar actualizaciones automáticamente al iniciar"
                                />
                            </div>

                            <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-background/40 p-3.5">
                                <div className="flex min-w-0 items-center gap-2">
                                    <span className="text-xs text-muted-foreground">
                                        Estado:
                                    </span>
                                    {status === "checking" && (
                                        <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                            <Loader2
                                                className="h-3 w-3 animate-spin text-primary"
                                                aria-hidden
                                            />
                                            Buscando...
                                        </span>
                                    )}
                                    {status === "available" && (
                                        <span className="inline-flex items-center gap-1 rounded-full bg-primary/15 px-2 py-0.5 font-mono text-[11px] font-semibold text-primary">
                                            v{updateInfo?.version} disponible
                                        </span>
                                    )}
                                    {status === "downloading" && (
                                        <span className="flex items-center gap-1 font-mono text-xs text-primary tabular-nums">
                                            Descargando...{" "}
                                            {progress?.percent != null
                                                ? `${progress.percent}%`
                                                : ""}
                                        </span>
                                    )}
                                    {status === "ready" && (
                                        <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-semibold text-emerald-600 dark:text-emerald-400">
                                            Lista para reiniciar
                                        </span>
                                    )}
                                    {status === "up-to-date" && (
                                        <span className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
                                            <CheckCircle2
                                                className="h-3.5 w-3.5"
                                                aria-hidden
                                            />
                                            Al día
                                        </span>
                                    )}
                                    {status === "idle" && (
                                        <span className="text-xs text-muted-foreground">
                                            Sin comprobar
                                        </span>
                                    )}
                                    {status === "error" && (
                                        <Alert
                                            variant="destructive"
                                            className="my-1 py-2 px-3"
                                        >
                                            <AlertCircle className="h-4 w-4" />
                                            <AlertTitle className="text-xs">
                                                Error al comprobar actualizaciones
                                            </AlertTitle>
                                            <AlertDescription className="text-[11px] text-destructive/90">
                                                {updateError
                                                    ? updateError
                                                    : "Verifica tu conexión a Internet o inténtalo nuevamente."}
                                            </AlertDescription>
                                        </Alert>
                                    )}
                                </div>

                                {status === "available" ? (
                                    <Button
                                        size="sm"
                                        onClick={() =>
                                            void downloadAndInstall()
                                        }
                                        className="h-7 gap-1.5 text-xs"
                                    >
                                        <Download
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Descargar e instalar
                                    </Button>
                                ) : status === "ready" ? (
                                    <Button
                                        size="sm"
                                        onClick={() => void restartApp()}
                                        className="h-7 gap-1.5 text-xs"
                                    >
                                        <RotateCw
                                            className="h-3.5 w-3.5"
                                            aria-hidden
                                        />
                                        Reiniciar MyLAN
                                    </Button>
                                ) : (
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() =>
                                            void checkForUpdates({
                                                silent: false,
                                            })
                                        }
                                        disabled={
                                            status === "checking" ||
                                            status === "downloading"
                                        }
                                        className="h-7 shrink-0 gap-1.5 text-xs"
                                    >
                                        <RefreshCw
                                            className={cn(
                                                "h-3 w-3",
                                                status === "checking" &&
                                                    "animate-spin",
                                            )}
                                            aria-hidden
                                        />
                                        {status === "error"
                                            ? "Reintentar"
                                            : "Comprobar ahora"}
                                    </Button>
                                )}
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>

            {/* Dialog de confirmación: db_path sin guardar antes de navegar */}
            {blocker.state === "blocked" && (
                <Dialog
                    open
                    onOpenChange={(o) => {
                        if (!o) blocker.reset();
                    }}
                >
                    <DialogContent className="max-w-sm">
                        <DialogTitle>¿Descartar cambios?</DialogTitle>
                        <DialogDescription>
                            Has modificado la ruta de la base de datos sin
                            guardar. Si navegas ahora, se perderá el cambio.
                        </DialogDescription>
                        <div className="flex justify-end gap-2">
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => blocker.reset()}
                            >
                                Quedarse
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => blocker.proceed()}
                            >
                                Descartar y salir
                            </Button>
                        </div>
                    </DialogContent>
                </Dialog>
            )}

            <GdriveSyncDialog
                open={syncDialogOpen}
                onOpenChange={setSyncDialogOpen}
                status={gdriveStatus}
                onStatusChange={setGdriveStatus}
                onSyncStart={() => setIsSyncingNow(true)}
                onSyncEnd={() => {
                    setIsSyncingNow(false);
                    getGdriveSyncStatus().then(setGdriveStatus).catch(() => {});
                }}
            />
        </div>
    );
}
