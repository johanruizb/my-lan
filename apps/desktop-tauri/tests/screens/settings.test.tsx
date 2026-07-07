// AC-22: Tests de screen `Settings` con MemoryRouter + providers + mock
// `@/lib/tauri` por función.

import { screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Settings } from "@/screens/Settings";
import { renderWithProviders } from "./helpers";
import { defaultSettings } from "../fixtures";

const getSettingsMock = vi.fn();
const setSettingsMock = vi.fn();
const dbPathMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    getSettings: () => getSettingsMock(),
    setSettings: (settings: unknown) => setSettingsMock(settings),
    dbPath: () => dbPathMock(),
}));

vi.mock("@/components/censorship-provider", () => ({
    useCensorship: () => ({
        censorshipEnabled: true,
        setCensorshipEnabled: vi.fn(),
        toggle: vi.fn(),
    }),
}));

vi.mock("@/components/theme-provider", () => ({
    useTheme: () => ({
        theme: "light",
        setTheme: vi.fn(),
        toggleTheme: vi.fn(),
    }),
}));

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

describe("Settings screen (AC-22)", () => {
    beforeEach(() => {
        getSettingsMock.mockReset();
        setSettingsMock.mockReset();
        dbPathMock.mockReset();
        getSettingsMock.mockResolvedValue(defaultSettings);
        setSettingsMock.mockResolvedValue(undefined);
        dbPathMock.mockResolvedValue("/tmp/mylan.db");
    });

    it("renderiza el título 'Ajustes' y los selectores de tema y censura", async () => {
        renderWithProviders(<Settings />);
        await waitFor(() => {
            expect(screen.getByText("Ajustes")).toBeInTheDocument();
        });
        expect(
            screen.getByLabelText("Selector de tema"),
        ).toBeInTheDocument();
        // Censura ahora es un switch accesible (#33) con nombre vía
        // aria-labelledby sobre el span "Censura".
        expect(
            screen.getByRole("switch", { name: /censura/i }),
        ).toBeInTheDocument();
    });

    it("carga el db_path desde el backend al montar", async () => {
        renderWithProviders(<Settings />);
        await waitFor(() => {
            expect(getSettingsMock).toHaveBeenCalled();
            expect(dbPathMock).toHaveBeenCalled();
        });
    });
});