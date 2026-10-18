import { useEffect, useState } from "react";
import { NetworkHeader } from "@/components/network-header";
import { SecurityScoreCard } from "@/components/security-score-card";
import { Devices } from "@/screens/Devices";
import { getSettings } from "@/lib/tauri";
import { SECTION_GAP } from "@/lib/design-tokens";
import { cn } from "@/lib/utils";

// Vista principal unificada (Red / Dashboard): une en una sola pantalla la red
// activa, el lanzamiento del descubrimiento y el inventario de dispositivos en
// modo vista previa (`variant="preview"`), con enlace directo a la interfaz
// dedicada `/devices` que reutiliza el componente `Devices` completo.
export function Dashboard() {
    const [profile, setProfile] = useState("normal");

    useEffect(() => {
        getSettings()
            .then((s) => setProfile(s.default_profile))
            .catch(() => {});
    }, []);

    return (
        <div className={cn("flex flex-col", SECTION_GAP)}>
            <div className="flex flex-col gap-0.5">
                <h1 className="text-xl font-bold tracking-tight">Red</h1>
                <p className="text-xs text-muted-foreground">
                    Estado de la red local, auditoría de seguridad y dispositivos conectados.
                </p>
            </div>
            <NetworkHeader profile={profile} onProfileChange={setProfile} />
            <SecurityScoreCard />
            <Devices variant="preview" />
        </div>
    );
}
