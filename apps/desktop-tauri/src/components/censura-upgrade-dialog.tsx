import { useEffect, useState } from "react";
import {
    Dialog,
    DialogContent,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import { useCensorship } from "@/components/censorship-provider";
import { onCensorshipFresh } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Eye, EyeOff } from "lucide-react";

// Dialog one-shot de upgrade (AC-4): distingue install nuevo de upgrade.
//
// - Install nuevo: `lib.rs` emite el evento Tauri `censorship:fresh` cuando el
//   archivo de ajustes NO existía antes de esta versión. Al recibirlo, se
//   marca `censorship_dialog_shown=1` (censura ya está ON por defecto) y NO se
//   muestra el dialog.
// - Upgrade (archivo ya existía): el evento no se emite. Si además
//   `censorship_dialog_shown` no está set, se muestra el dialog una vez; al
//   elegir, se persiste la elección y se marca el flag.
//
// El evento puede emitirse antes de que el listener se registre (setup corre
// en el backend antes de que el frontend cargue). Se espera un tiempo acotado
// (FRESH_WAIT_MS): si el evento llega dentro de la ventana, se suprime; si no,
// se asume upgrade. El edge case (install nuevo donde el evento se pierde)
// muestra el dialog una vez, pero la opción recomendada coincide con el
// default ON, así que el resultado final es el mismo.
//
// Usa ui/dialog.tsx (Radix Dialog con focus trap/restore/Escape/scroll lock +
// DialogTitle/DialogDescription/close aria-label). Si el usuario cierra con X
// o Escape sin elegir, se persiste el flag (no re-aparece) pero NO se cambia
// el estado de censura: se respeta la configuración existente del upgrade.

const SHOWN_KEY = "censorship_dialog_shown";
const FRESH_WAIT_MS = 2000;

export function CensuraUpgradeDialog() {
    const { setCensorshipEnabled } = useCensorship();
    const [open, setOpen] = useState(false);

    useEffect(() => {
        if (localStorage.getItem(SHOWN_KEY) === "1") return;

        let resolved = false;
        let unlistenFn: (() => void) | null = null;

        const finish = (fresh: boolean) => {
            if (resolved) return;
            resolved = true;
            if (unlistenFn) {
                unlistenFn();
                unlistenFn = null;
            }
            if (fresh) {
                // Install nuevo: censura ON por defecto, no mostrar dialog.
                localStorage.setItem(SHOWN_KEY, "1");
            } else {
                // Upgrade: mostrar dialog una sola vez.
                setOpen(true);
            }
        };

        onCensorshipFresh(() => finish(true))
            .then((fn) => {
                if (resolved) {
                    // Ya resuelto por timeout: limpiar listener y salir.
                    fn();
                } else {
                    unlistenFn = fn;
                }
            })
            .catch(() => {});

        const timer = setTimeout(() => finish(false), FRESH_WAIT_MS);

        return () => {
            clearTimeout(timer);
            if (unlistenFn) unlistenFn();
        };
    }, []);

    function dismiss() {
        // Cierre sin elegir (X/Escape): persiste el flag para no re-aparecer,
        // pero NO cambia el estado de censura (respeta la configuración
        // existente del upgrade).
        localStorage.setItem(SHOWN_KEY, "1");
        setOpen(false);
    }

    function choose(activate: boolean) {
        setCensorshipEnabled(activate);
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
                    <EyeOff className="h-5 w-5 text-primary" aria-hidden />
                    <DialogTitle>Modo censura</DialogTitle>
                </div>
                <DialogDescription>
                    MyLAN puede enmascarar identificadores sensibles (IP, MAC,
                    hostname, gateway, DNS) en la interfaz y los exports para
                    evitar que se compartan por error en capturas de pantalla o
                    archivos. Puedes cambiarlo cuando quieras desde Ajustes.
                </DialogDescription>
                <div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => choose(false)}
                        className="gap-1.5"
                    >
                        <Eye className="h-4 w-4" aria-hidden />
                        Mantener visible
                    </Button>
                    <Button
                        size="sm"
                        onClick={() => choose(true)}
                        className="gap-1.5"
                    >
                        <EyeOff className="h-4 w-4" aria-hidden />
                        Activar (recomendado)
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
