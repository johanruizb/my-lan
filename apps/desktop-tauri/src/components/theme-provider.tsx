import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useState,
    type ReactNode,
} from "react";
import { getSettings, setSettings, type Settings } from "@/lib/tauri";

// ThemeProvider (AC-3): conmuta `class="dark"` en <html>, persiste el tema en
// settings (`theme: "light" | "dark"`). Todos los componentes usan tokens CSS
// var, así se adaptan automáticamente.

type Theme = "light" | "dark";

interface ThemeContextValue {
    theme: Theme;
    toggleTheme: () => void;
    setTheme: (t: Theme) => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

function applyTheme(theme: Theme) {
    const root = document.documentElement;
    if (theme === "dark") {
        root.classList.add("dark");
    } else {
        root.classList.remove("dark");
    }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
    const [theme, setThemeState] = useState<Theme>("light");

    // Carga inicial: aplica antes del primer render para evitar flash. Lee
    // settings persistidos; si falla, respeta la preferencia del sistema.
    useEffect(() => {
        const prefersDark = window.matchMedia(
            "(prefers-color-scheme: dark)",
        ).matches;
        getSettings()
            .then((s: Settings) => {
                const t: Theme = s.theme === "dark" ? "dark" : "light";
                setThemeState(t);
                applyTheme(t);
            })
            .catch(() => {
                const t: Theme = prefersDark ? "dark" : "light";
                setThemeState(t);
                applyTheme(t);
            });
    }, []);

    const persist = useCallback(async (next: Theme) => {
        setThemeState(next);
        applyTheme(next);
        try {
            const s = await getSettings();
            await setSettings({ ...s, theme: next });
        } catch {
            // Si persistencia falla, el toggle visual sigue funcionando en sesión.
        }
    }, []);

    const value: ThemeContextValue = {
        theme,
        toggleTheme: () => persist(theme === "dark" ? "light" : "dark"),
        setTheme: persist,
    };

    return (
        <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
    );
}

export function useTheme(): ThemeContextValue {
    const ctx = useContext(ThemeContext);
    if (!ctx) {
        throw new Error("useTheme debe usarse dentro de ThemeProvider");
    }
    return ctx;
}
