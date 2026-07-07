import { useEffect, useState } from "react";
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
    Sun,
    Moon,
    Database,
    Gauge,
    Palette,
    EyeOff,
    Loader2,
    ChevronDown,
    RotateCcw,
    Settings as SettingsIcon,
} from "lucide-react";
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
                        Configuración de MyLAN Desktop.
                    </CardDescription>
                </CardHeader>
                <CardContent className="flex flex-col gap-5">
                    {/* #20 */}
                    <section className="flex flex-col gap-3">
                        <h3 className="text-sm font-medium">Escaneo</h3>
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
                        <h3 className="text-sm font-medium">Apariencia</h3>
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
                        <h3 className="text-sm font-medium">Privacidad</h3>
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
        </div>
    );
}
