import {
    createContext,
    createElement,
    useCallback,
    useContext,
    useEffect,
    useRef,
    useState,
    type ReactNode,
} from "react";
import { detectInterface, getNetworkName, setNetworkName } from "@/lib/tauri";
import { useScan } from "@/App";

// NetworkNameProvider: mantiene el nombre de la red activa (SSID Wi-Fi auto,
// etiqueta de usuario, o el CIDR como fallback) keyed por CIDR. Sigue el patrón
// de CensorshipProvider: un contexto reactivo que el pie de la sidebar consume
// y que se re-renderiza en vivo.
//
// El nombre de la red NO se enmascara por censura (es la etiqueta de la propia
// red del usuario, no un identificador sensible): se renderiza en claro, nunca
// vía <MaskedValue> (AC-11).

interface NetworkNameContextValue {
    /** Nombre humano de la red: SSID / etiqueta / CIDR. Vacío hasta cargar. */
    name: string;
    /** Origen del nombre: `"auto"` (SSID/CIDR) o `"user"` (editado). */
    source: string;
    /** CIDR de la red activa = clave de persistencia; `null` si no detectada. */
    cidr: string | null;
    /** Persiste una etiqueta de usuario para el CIDR activo (override gana). */
    editName: (label: string) => Promise<void>;
    /** Re-detecta la interfaz y recarga el nombre persistido. */
    refresh: () => Promise<void>;
}

const NetworkNameContext = createContext<NetworkNameContextValue | null>(null);

export function NetworkNameProvider({ children }: { children: ReactNode }) {
    const { scanning } = useScan();
    const [name, setName] = useState("");
    const [source, setSource] = useState("auto");
    const [cidr, setCidr] = useState<string | null>(null);

    const refresh = useCallback(async () => {
        try {
            const iface = await detectInterface();
            setCidr(iface.cidr);
            const nn = await getNetworkName(iface.cidr);
            setName(nn.name);
            setSource(nn.source);
        } catch {
            // Sin interfaz / backend no listo: el pie muestra el fallback.
        }
    }, []);

    // Carga inicial al montar.
    useEffect(() => {
        refresh();
    }, [refresh]);

    // Tras cada scan (el backend persiste el SSID auto-detectado en
    // `networks.name` durante run_discovery_cmd), recarga para reflejarlo.
    const prevScanning = useRef(scanning);
    useEffect(() => {
        if (prevScanning.current && !scanning) refresh();
        prevScanning.current = scanning;
    }, [scanning, refresh]);

    const editName = useCallback(
        async (label: string) => {
            if (!cidr) return;
            await setNetworkName(cidr, label);
            // Optimista: el override del usuario gana; refleja de inmediato.
            setName(label);
            setSource("user");
        },
        [cidr],
    );

    const value: NetworkNameContextValue = {
        name,
        source,
        cidr,
        editName,
        refresh,
    };

    return createElement(NetworkNameContext.Provider, { value }, children);
}

export function useNetworkName(): NetworkNameContextValue {
    const ctx = useContext(NetworkNameContext);
    if (!ctx) {
        throw new Error(
            "useNetworkName debe usarse dentro de NetworkNameProvider",
        );
    }
    return ctx;
}
