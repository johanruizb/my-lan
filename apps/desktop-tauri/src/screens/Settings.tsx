import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ProfileSelect } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
  dbPath,
  getSettings,
  setSettings,
  type Settings as SettingsDto,
} from "@/lib/tauri";

export function Settings() {
  const { toast } = useToast();
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

  if (!settings) return <p className="text-sm text-muted-foreground">Cargando…</p>;

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Ajustes</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-muted-foreground">Path de la base de datos</label>
            <Input
              value={dbPathValue}
              onChange={(e) => setDbPathValue(e.target.value)}
              className="max-w-xl"
            />
            <p className="text-xs text-muted-foreground">
              La app crea/migra la SQLite en este path al arrancar. Para usuarios
              brownfield, la DB del CLI se importa automáticamente la primera vez.
            </p>
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-xs text-muted-foreground">Perfil de scan por defecto</label>
            <ProfileSelect
              value={settings.default_profile}
              onChange={(v) =>
                setSettingsState({ ...settings, default_profile: v })
              }
              className="w-40"
            />
          </div>
          <Button onClick={handleSave} disabled={saving} className="w-fit">
            {saving ? "Guardando…" : "Guardar"}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}