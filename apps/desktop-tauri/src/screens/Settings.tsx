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
import { Badge } from "@/components/ui/badge";
import { ProfileSelect } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import { useTheme } from "@/components/theme-provider";
import { Sun, Moon, Database, Gauge, Palette, Loader2 } from "lucide-react";
import {
    dbPath,
    getSettings,
    setSettings,
    type Settings as SettingsDto,
} from "@/lib/tauri";

export function Settings() {
    const { toast } = useToast();
    const { theme, setTheme } = useTheme();
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
        <div className="flex flex-col gap-4">
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
                <CardContent className="flex flex-col gap-6">
                    <div className="flex flex-col gap-1.5">
                        <label
                            htmlFor="db-path"
                            className="text-xs text-muted-foreground"
                        >
                            Path de la base de datos
                        </label>
                        <Input
                            id="db-path"
                            value={dbPathValue}
                            onChange={(e) => setDbPathValue(e.target.value)}
                            className="max-w-xl"
                        />
                        <p className="text-xs text-muted-foreground">
                            La app crea/migra la SQLite en este path al
                            arrancar. Para usuarios brownfield, la DB del CLI se
                            importa automáticamente la primera vez.
                        </p>
                    </div>

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

                    {/* Toggle de tema (AC-3). */}
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
                                    theme === "light" ? "secondary" : "outline"
                                }
                                size="sm"
                                onClick={() => setTheme("light")}
                                aria-pressed={theme === "light"}
                                className="gap-1.5"
                            >
                                <Sun className="h-4 w-4" aria-hidden />
                                Claro
                                {theme === "light" && (
                                    <Badge variant="success" className="ml-1">
                                        Activo
                                    </Badge>
                                )}
                            </Button>
                            <Button
                                variant={
                                    theme === "dark" ? "secondary" : "outline"
                                }
                                size="sm"
                                onClick={() => setTheme("dark")}
                                aria-pressed={theme === "dark"}
                                className="gap-1.5"
                            >
                                <Moon className="h-4 w-4" aria-hidden />
                                Oscuro
                                {theme === "dark" && (
                                    <Badge variant="success" className="ml-1">
                                        Activo
                                    </Badge>
                                )}
                            </Button>
                        </div>
                        <p className="text-xs text-muted-foreground">
                            El tema se aplica inmediatamente y se persiste en
                            los ajustes.
                        </p>
                    </div>

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
