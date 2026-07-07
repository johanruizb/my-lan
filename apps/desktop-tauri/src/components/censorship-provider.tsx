import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useState,
    type ReactNode,
} from "react";
import { getSettings, setSettings } from "@/lib/tauri";

// CensorshipProvider (AC-3): modo censura enmascara identificadores estrictos
// (IP/MAC/hostname/display_name/gateway/dns/cidr) en la UI y los exports.
// Sigue exactamente el patrón de ThemeProvider: un contexto reactivo que
// persiste en `Settings.censorship_enabled` y re-renderiza en vivo todas las
// pantallas consumidoras sin reinicio. ON por defecto para installs nuevos.

interface CensorshipContextValue {
    censorshipEnabled: boolean;
    setCensorshipEnabled: (v: boolean) => void;
    toggle: () => void;
}

const CensorshipContext = createContext<CensorshipContextValue | null>(null);

export function CensorshipProvider({ children }: { children: ReactNode }) {
    // Default true hasta que se cargue `Settings` (AC-1: ON por defecto).
    const [censorshipEnabled, setCensorshipState] = useState(true);

    // Carga inicial: lee `Settings.censorship_enabled`, default true si falta
    // (backward-compat con `mylan-desktop.json` antiguos, AC-6).
    useEffect(() => {
        getSettings()
            .then((s) => {
                setCensorshipState(s.censorship_enabled ?? true);
            })
            .catch(() => {
                // Si falla la lectura, mantiene el default ON.
            });
    }, []);

    const persist = useCallback(async (next: boolean) => {
        setCensorshipState(next);
        try {
            const s = await getSettings();
            await setSettings({ ...s, censorship_enabled: next });
        } catch {
            // Si persistencia falla, el toggle visual sigue funcionando en sesión.
        }
    }, []);

    const value: CensorshipContextValue = {
        censorshipEnabled,
        setCensorshipEnabled: persist,
        toggle: () => persist(!censorshipEnabled),
    };

    return (
        <CensorshipContext.Provider value={value}>
            {children}
        </CensorshipContext.Provider>
    );
}

export function useCensorship(): CensorshipContextValue {
    const ctx = useContext(CensorshipContext);
    if (!ctx) {
        throw new Error(
            "useCensorship debe usarse dentro de CensorshipProvider",
        );
    }
    return ctx;
}
