// AC-17: Tests de `censorship-provider.tsx` (mock getSettings/setSettings).
// Valida la carga inicial desde Settings, el toggle y que la persistencia
// llama a setSettings con el nuevo valor.

import { act, renderHook, waitFor } from "@testing-library/react";
import { vi } from "vitest";
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

describe("CensorshipProvider", () => {
    beforeEach(() => {
        getSettingsMock.mockReset();
        setSettingsMock.mockReset();
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: true,
        });
        setSettingsMock.mockResolvedValue(undefined);
    });

    it("arranca con censura ON por defecto antes de cargar Settings", () => {
        // getSettings nunca resuelve: el estado inicial debe quedar en true.
        getSettingsMock.mockReturnValue(new Promise(() => {}));
        const { result } = renderHook(() => useCensorship(), { wrapper });
        expect(result.current.censorshipEnabled).toBe(true);
    });

    it("carga censorship_enabled desde getSettings al montar", async () => {
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: false,
        });
        const { result } = renderHook(() => useCensorship(), { wrapper });
        await waitFor(() => {
            expect(result.current.censorshipEnabled).toBe(false);
        });
    });

    it("toggle invoca setSettings con el nuevo valor y actualiza el estado", async () => {
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: true,
        });
        const { result } = renderHook(() => useCensorship(), { wrapper });
        await waitFor(() => {
            expect(result.current.censorshipEnabled).toBe(true);
        });
        await act(async () => {
            result.current.toggle();
        });
        await waitFor(() => {
            expect(setSettingsMock).toHaveBeenCalledWith(
                expect.objectContaining({ censorship_enabled: false }),
            );
            expect(result.current.censorshipEnabled).toBe(false);
        });
    });

    it("setCensorshipEnabled persiste el valor exacto", async () => {
        getSettingsMock.mockResolvedValue({
            ...defaultSettings,
            censorship_enabled: false,
        });
        const { result } = renderHook(() => useCensorship(), { wrapper });
        await waitFor(() => {
            expect(result.current.censorshipEnabled).toBe(false);
        });
        await act(async () => {
            result.current.setCensorshipEnabled(true);
        });
        await waitFor(() => {
            expect(setSettingsMock).toHaveBeenCalledWith(
                expect.objectContaining({ censorship_enabled: true }),
            );
            expect(result.current.censorshipEnabled).toBe(true);
        });
    });
});