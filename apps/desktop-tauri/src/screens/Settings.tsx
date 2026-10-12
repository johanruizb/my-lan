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
import { ProfileSelect } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import { useTheme } from "@/components/theme-provider";
import { useCensorship } from "@/components/censorship-provider";
import { InfoTooltip } from "@/components/ui/info-tooltip";
import { reopenOnboarding } from "@/components/onboarding-dialog";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
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
    ChevronDown,
    RotateCcw,
    RotateCw,
    RefreshCw,
    Download,
    CheckCircle2,
    Settings as SettingsIcon,
} from "lucide-react";
import { useUpdater } from "@/components/updater-provider";
import {
    dbPath,
    getSettings,
    setSettings,
    type Settings as SettingsDto,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { SECTION_GAP } from "@/lib/design-tokens";

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
        checkForUpdates,
        downloadAndInstall,
        restartApp,
    } = useUpdater();

    // Aviso de cambios sin guardar: db_path modificado pero no guardado.
    // theme y censorship se aplican inmediatamente (no necesitan guardado).
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
    }, []);

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
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Database
                            className="h-5 w-5 text-primary"
                            aria-hidden
                        />
                        Ajustes
                    </CardTitle>
                    <CardDescription>
                        Configuración de <span translate="no">MyLAN</span>{" "}
                        Desktop.
                    </CardDescription>
                </CardHeader>
                <CardContent className="flex flex-col gap-5">
                    {/* #20 */}
                    <section className="flex flex-col gap-3">
                        <h2 className="text-sm font-medium">Escaneo</h2>
                        <div className="flex flex-col gap-1.5">
                            <label
                                htmlFor="default-profile"
                                className="flex items-center gap-1.5 text-xs text-muted-foreground"
                            >
                                <Gauge className="h-3.5 w-3.5" aria-hidden />
                                Perfil de scan por defecto
                            </label>
                            <ProfileSelect
                                value={settings.default_profile}
                                onChange={(v) =>
                                    setSettingsState({
                                        ...settings,
                                        default_profile: v,
                                    })
                                }
                                className="w-40"
                                id="default-profile"
                            />
                        </div>
                    </section>

                    <div className="h-px bg-border" role="separator" />

                    {/* #20 #16 */}
                    <section className="flex flex-col gap-3">
                        <h2 className="text-sm font-medium">Apariencia</h2>
                        <div className="flex flex-col gap-1.5">
                            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                <Palette className="h-3.5 w-3.5" aria-hidden />
                                Tema
                            </span>
                            <div
                                className="flex gap-2"
                                role="group"
                                aria-label="Selector de tema"
                            >
                                <Button
                                    variant={
                                        theme === "light"
                                            ? "secondary"
                                            : "outline"
                                    }
                                    size="sm"
                                    onClick={() => setTheme("light")}
                                    aria-pressed={theme === "light"}
                                    className="gap-1.5"
                                >
                                    <Sun className="h-4 w-4" aria-hidden />
                                    Claro
                                </Button>
                                <Button
                                    variant={
                                        theme === "dark"
                                            ? "secondary"
                                            : "outline"
                                    }
                                    size="sm"
                                    onClick={() => setTheme("dark")}
                                    aria-pressed={theme === "dark"}
                                    className="gap-1.5"
                                >
                                    <Moon className="h-4 w-4" aria-hidden />
                                    Oscuro
                                </Button>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                El tema se aplica inmediatamente y se persiste
                                en los ajustes.
                            </p>
                        </div>
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => reopenOnboarding()}
                            className="h-8 w-fit gap-1.5 text-xs text-muted-foreground"
                        >
                            <RotateCcw className="h-3.5 w-3.5" aria-hidden />
                            Ver de nuevo el tour de bienvenida
                        </Button>
                    </section>

                    <div className="h-px bg-border" role="separator" />

                    {/* #20 #33 AC-17 */}
                    <section className="flex flex-col gap-3">
                        <h2 className="text-sm font-medium">Privacidad</h2>
                        <div className="flex flex-col gap-1.5">
                            <div className="flex items-center justify-between gap-3">
                                <span
                                    id="censura-label"
                                    className="flex items-center gap-1.5 text-xs text-muted-foreground"
                                >
                                    <EyeOff
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    Censura
                                </span>
                                <Switch
                                    checked={censorshipEnabled}
                                    onCheckedChange={setCensorshipEnabled}
                                    aria-labelledby="censura-label"
                                />
                            </div>
                            <p className="text-xs text-muted-foreground">
                                El modo censura enmascara identificadores (IP
                                <InfoTooltip term="IP" glossaryKey="ip" />, MAC
                                <InfoTooltip term="MAC" glossaryKey="mac" />,
                                hostname
                                <InfoTooltip
                                    term="Hostname"
                                    glossaryKey="hostname"
                                />
                                , gateway
                                <InfoTooltip
                                    term="Gateway"
                                    glossaryKey="gateway"
                                />
                                , DNS
                                <InfoTooltip term="DNS" glossaryKey="dns" />) en
                                la UI y los exports para evitar compartirlos por
                                error en capturas o archivos. Se aplica
                                inmediatamente y se persiste en los ajustes.
                            </p>
                        </div>

                        {/* AC-17 */}
                        <Collapsible>
                            <CollapsibleTrigger className="flex w-fit items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground">
                                <SettingsIcon
                                    className="h-3.5 w-3.5"
                                    aria-hidden
                                />
                                Avanzado
                                <ChevronDown
                                    className="h-3.5 w-3.5 transition-transform data-[state=closed]:-rotate-90"
                                    aria-hidden
                                />
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                                <div className="mt-2 flex flex-col gap-1.5">
                                    <label
                                        htmlFor="db-path"
                                        className="text-xs text-muted-foreground"
                                    >
                                        Ruta de la base de datos
                                    </label>
                                    <Input
                                        id="db-path"
                                        value={dbPathValue}
                                        onChange={(e) =>
                                            setDbPathValue(e.target.value)
                                        }
                                        className="max-w-xl"
                                        name="db-path"
                                        autoComplete="off"
                                        spellCheck={false}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Ubicación donde la app guarda sus datos.
                                        Solo necesaria si quieres usar una ruta
                                        personalizada.
                                    </p>
                                </div>
                            </CollapsibleContent>
                        </Collapsible>
                    </section>

                    <div className="h-px bg-border" role="separator" />

                    <section className="flex flex-col gap-3">
                        <div className="flex flex-col gap-0.5">
                            <h2 className="text-sm font-medium">
                                Actualizaciones de la aplicación
                            </h2>
                            <p className="text-xs text-muted-foreground">
                                Comprueba e instala nuevas versiones desde
                                GitHub Releases sin intermediarios ni descargas
                                manuales.
                            </p>
                        </div>

                        <div className="flex items-center justify-between gap-4 rounded-lg border bg-muted/20 p-3">
                            <div className="flex flex-col gap-0.5">
                                <span className="text-sm font-medium">
                                    Buscar al iniciar
                                </span>
                                <span className="text-xs text-muted-foreground">
                                    Comprobar en segundo plano si hay nuevas
                                    versiones al abrir MyLAN.
                                </span>
                            </div>
                            <Switch
                                checked={
                                    settings.check_updates_on_startup ?? true
                                }
                                onCheckedChange={async (checked) => {
                                    const next = {
                                        ...settings,
                                        check_updates_on_startup: checked,
                                    };
                                    setSettingsState(next);
                                    try {
                                        await setSettings(next);
                                    } catch (e) {
                                        toast(`Error: ${e}`, "error");
                                    }
                                }}
                                aria-label="Buscar actualizaciones automáticamente al iniciar"
                            />
                        </div>

                        <div className="flex items-center justify-between gap-2 pt-1">
                            <div className="flex items-center gap-2">
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
                                    <span className="inline-flex items-center gap-1 rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-semibold text-primary">
                                        v{updateInfo?.version} disponible
                                    </span>
                                )}
                                {status === "downloading" && (
                                    <span className="flex items-center gap-1 text-xs text-primary">
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
                                    <span className="text-xs text-destructive">
                                        Error al comprobar
                                    </span>
                                )}
                            </div>

                            {status === "available" ? (
                                <Button
                                    size="sm"
                                    onClick={() => void downloadAndInstall()}
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
                                    className="h-7 gap-1.5 text-xs"
                                >
                                    <RefreshCw
                                        className={cn(
                                            "h-3 w-3",
                                            status === "checking" &&
                                                "animate-spin",
                                        )}
                                        aria-hidden
                                    />
                                    Comprobar ahora
                                </Button>
                            )}
                        </div>
                    </section>

                    <Button
                        onClick={handleSave}
                        disabled={saving}
                        className="w-fit gap-1.5"
                    >
                        {saving ? (
                            <>
                                <Loader2
                                    className="h-4 w-4 animate-spin"
                                    aria-hidden
                                />
                                Guardando…
                            </>
                        ) : (
                            "Guardar"
                        )}
                    </Button>
                </CardContent>
            </Card>

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
