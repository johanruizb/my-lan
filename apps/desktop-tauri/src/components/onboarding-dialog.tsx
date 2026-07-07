import { useEffect, useState } from "react";
import {
    Dialog,
    DialogContent,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Wifi, Radar, EyeOff } from "lucide-react";

// Onboarding primera ejecución (AC-4): tour sobre qué hace la app y la
// diferencia entre descubrimiento (Dashboard) y escaneo de puertos (Scans /
// DeviceDetail). Persistido en localStorage `onboarding_shown` (one-shot).
// Consume ui/dialog.tsx (Radix Dialog con focus trap/restore/Escape/scroll
// lock + DialogTitle/DialogDescription/close aria-label). NO copia el patrón
// custom div de censura-upgrade-dialog.tsx.
//
// Tres cards: Descubrimiento, Escaneo de puertos y Modo censura.
//
// Re-abrible desde Ajustes: `reopenOnboarding()` limpia `onboarding_shown` y
// vuelve a abrir el dialog. Mecanismo con setter a nivel de módulo (no
// requiere context provider ni envolver App). OnboardingDialog debe estar
// montado (lo está, en App.tsx raíz) para que el setter esté registrado.

const SHOWN_KEY = "onboarding_shown";

// Setter registrado por el componente mientras está montado. Si el componente
// no está montado, reopenOnboarding() es no-op (devuelve false).
let reopenFn: (() => void) | null = null;

/**
 * Re-abre el dialog de onboarding desde Ajustes. Limpia el flag de
 * persistencia y abre el dialog. Devuelve true si se pudo invocar (el
 * componente está montado), false en caso contrario.
 */
export function reopenOnboarding(): boolean {
    if (!reopenFn) return false;
    reopenFn();
    return true;
}

export function OnboardingDialog() {
    const [open, setOpen] = useState(false);

    useEffect(() => {
        try {
            if (localStorage.getItem(SHOWN_KEY) !== "1") setOpen(true);
        } catch {
            setOpen(true);
        }
    }, []);

    useEffect(() => {
        reopenFn = () => {
            try {
                localStorage.removeItem(SHOWN_KEY);
            } catch {
                // ignore: si localStorage falla, igual abrimos el dialog.
            }
            setOpen(true);
        };
        return () => {
            reopenFn = null;
        };
    }, []);

    function dismiss() {
        localStorage.setItem(SHOWN_KEY, "1");
        setOpen(false);
    }

    return (
        <Dialog
            open={open}
            onOpenChange={(o) => {
                if (!o) dismiss();
            }}
        >
            <DialogContent className="max-w-md">
                <div className="flex items-center gap-2">
                    <Wifi className="h-5 w-5 text-primary" aria-hidden />
                    <DialogTitle>Bienvenido a MyLAN</DialogTitle>
                </div>
                <DialogDescription>
                    MyLAN te ayuda a entender tu red local: qué dispositivos hay
                    conectados, qué servicios exponen y cómo cambia tu red con
                    el tiempo.
                </DialogDescription>

                <div className="flex flex-col gap-4">
                    <div className="flex items-start gap-3">
                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                            <Wifi className="h-4 w-4" aria-hidden />
                        </div>
                        <div className="flex flex-col gap-0.5">
                            <p className="text-sm font-medium">
                                Descubrimiento (Dashboard)
                            </p>
                            <p className="text-xs text-muted-foreground">
                                Recorre toda tu red y lista los dispositivos
                                conectados (PC, móvil, impresora, IoT). Es el
                                punto de partida.
                            </p>
                        </div>
                    </div>
                    <div className="flex items-start gap-3">
                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                            <Radar className="h-4 w-4" aria-hidden />
                        </div>
                        <div className="flex flex-col gap-0.5">
                            <p className="text-sm font-medium">
                                Escaneo de puertos (Scans)
                            </p>
                            <p className="text-xs text-muted-foreground">
                                Analiza un dispositivo concreto para ver qué
                                puertos tiene abiertos y qué servicios ofrece
                                (web, SSH, impresora…).
                            </p>
                        </div>
                    </div>
                    <div className="flex items-start gap-3">
                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                            <EyeOff className="h-4 w-4" aria-hidden />
                        </div>
                        <div className="flex flex-col gap-0.5">
                            <p className="text-sm font-medium">Modo censura</p>
                            <p className="text-xs text-muted-foreground">
                                Enmascara identificadores sensibles (IP, MAC,
                                hostname) en la interfaz y los exports. Pasa el
                                cursor o enfoca con el teclado un valor para
                                revelarlo temporalmente. Actívalo o desactívalo
                                cuando quieras desde Ajustes.
                            </p>
                        </div>
                    </div>
                </div>

                <div className="flex justify-end">
                    <Button size="sm" onClick={dismiss} className="gap-1.5">
                        Entendido
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
