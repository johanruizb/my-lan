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
} from "lucide-react";
import { useUpdater } from "@/components/updater-provider";
import { PROFILES } from "@/lib/profiles";
import {
    dbPath,
    getSettings,
    setSettings,
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

export function Settings() {
    const { toast } = useToast();
    const { theme, setTheme } = useTheme();
    const { censorshipEnabled, setCensorshipEnabled } = useCensorship();
    const [settings, setSettingsState] = useState<SettingsDto | null>(null);
    const [dbPathValue, setDbPathValue] = useState("");
    const [saving, setSaving] = useState(false);

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
        Promise.all([getSettings(), dbPath()])
            .then(([s, p]) => {
                setSettingsState(s);
                setDbPathValue(p || s.db_path);
            })
            .catch((e) => toast(`Error: ${e}`, "error"));
    }, [toast]);

    async function persistPatch(patch: Partial<SettingsDto>) {
        if (!settings) return;
        const next: SettingsDto = {
            db_path: settings.db_path,
            default_profile: settings.default_profile,
            theme,
            censorship_enabled: censorshipEnabled,
            check_updates_on_startup:
                settings.check_updates_on_startup ?? true,
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
                db_path: dbPathValue,
                default_profile: settings.default_profile,
                theme,
                censorship_enabled: censorshipEnabled,
                check_updates_on_startup:
                    settings.check_updates_on_startup ?? true,
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
                                        <button
                                            key={p.value}
                                            type="button"
                                            role="radio"
                                            aria-checked={active}
                                            onClick={() =>
                                                void persistPatch({
                                                    default_profile: p.value,
                                                })
                                            }
                                            className={cn(
                                                "group flex flex-col justify-between gap-2 rounded-lg border p-3.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                                active
                                                    ? "border-primary/60 bg-primary/10"
                                                    : "border-border bg-background/40 hover:border-border/80 hover:bg-muted/40",
                                            )}
                                        >
                                            <div className="flex flex-col gap-1">
                                                <div className="flex items-center justify-between gap-2">
                                                    <span className="text-sm font-semibold text-foreground">
                                                        {p.label}
                                                    </span>
                                                    <span
                                                        aria-hidden
                                                        className={cn(
                                                            "flex h-4 w-4 shrink-0 items-center justify-center rounded-full border",
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
                                                <p className="text-xs leading-relaxed text-muted-foreground">
                                                    {p.description}
                                                </p>
                                            </div>
                                            <span className="border-t border-border/60 pt-2 text-[11px] text-muted-foreground/90">
                                                {p.tradeoff}
                                            </span>
                                        </button>
                                    );
                                })}
                            </div>
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

                                <div className="flex flex-wrap items-center gap-1.5 border-t border-border/60 pt-2.5">
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
        </div>
    );
}
