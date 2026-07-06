// AC-21: Tests de hook `useCensorship` con renderHook.
// Valida que el hook retorna el valor del CensorshipProvider y que throw
// cuando se usa fuera del provider.

import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
    CensorshipProvider,
    useCensorship,
} from "@/components/censorship-provider";
import { defaultSettings } from "./fixtures";

const getSettingsMock = vi.fn();
const setSettingsMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    getSettings: () => getSettingsMock(),
    setSettings: (settings: unknown) => setSettingsMock(settings),
}));

function wrapper({ children }: { children: React.ReactNode }) {
    return <CensorshipProvider>{children}</CensorshipProvider>;
}

describe("useCensorship — hook (AC-21)", () => {
    // Silenciar el stack trace de react-dom que React 18 loguea al throw del
    // hook fuera del provider (code-review MINOR: limpiar output CI).
    beforeEach(() => {
        getSettingsMock.mockReset();
        setSettingsMock.mockReset();
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: true,
        });
        setSettingsMock.mockResolvedValue(undefined);
        vi.spyOn(console, "error").mockImplementation(() => {});
    });
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("throw cuando se usa fuera de CensorshipProvider", () => {
        expect(() => renderHook(() => useCensorship())).toThrow(
            /useCensorship debe usarse dentro de CensorshipProvider/,
        );
    });

    it("retorna el valor del provider dentro de CensorshipProvider", async () => {
        const { result } = renderHook(() => useCensorship(), { wrapper });
        await waitFor(() => {
            expect(result.current.censorshipEnabled).toBe(true);
        });
        expect(typeof result.current.toggle).toBe("function");
        expect(typeof result.current.setCensorshipEnabled).toBe("function");
    });

    it("reflecta censorship_enabled=false cargado desde getSettings", async () => {
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: false,
        });
        const { result } = renderHook(() => useCensorship(), { wrapper });
        await waitFor(() => {
            expect(result.current.censorshipEnabled).toBe(false);
        });
    });
});