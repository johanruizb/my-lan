// AC-22: Tests de screen `Settings` con MemoryRouter + providers + mock
// `@/lib/tauri` por función.

import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AppMark } from "@/App";
import { Settings } from "@/screens/Settings";
import { renderWithProviders } from "./helpers";
import { defaultSettings } from "../fixtures";

const getSettingsMock = vi.fn();
const setSettingsMock = vi.fn();
const dbPathMock = vi.fn();
const getMonitoringStatusMock = vi.fn();
const toggleMonitoringPauseMock = vi.fn();
const triggerBackgroundScanMock = vi.fn();
const onMonitoringStatusMock = vi.fn();
const getGdriveSyncStatusMock = vi.fn();
const syncWorkspaceMock = vi.fn();
const startGdriveAuthMock = vi.fn();
const disconnectGdriveMock = vi.fn();
const setupMasterKeyMock = vi.fn();
const restoreMasterKeyMock = vi.fn();
const resolveSyncConflictMock = vi.fn();
let currentMockTheme: "light" | "dark" = "light";

vi.mock("@/lib/tauri", () => ({
    getSettings: () => getSettingsMock(),
    setSettings: (settings: unknown) => setSettingsMock(settings),
    dbPath: () => dbPathMock(),
    getMonitoringStatus: () => getMonitoringStatusMock(),
    toggleMonitoringPause: () => toggleMonitoringPauseMock(),
    triggerBackgroundScan: () => triggerBackgroundScanMock(),
    onMonitoringStatus: (cb: unknown) => onMonitoringStatusMock(cb),
    getGdriveSyncStatus: () => getGdriveSyncStatusMock(),
    syncWorkspace: (wsId: string, override?: boolean) => syncWorkspaceMock(wsId, override),
    startGdriveAuth: () => startGdriveAuthMock(),
    disconnectGdrive: () => disconnectGdriveMock(),
    setupMasterKey: (pass: string) => setupMasterKeyMock(pass),
    restoreMasterKey: (input: string) => restoreMasterKeyMock(input),
    resolveSyncConflict: (wsId: string, res: string) => resolveSyncConflictMock(wsId, res),
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
        theme: currentMockTheme,
        setTheme: vi.fn(),
        toggleTheme: vi.fn(),
    }),
}));

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

const checkForUpdatesMock = vi.fn();
let updaterMockState: {
    status: string;
    updateInfo: unknown;
    progress: unknown;
    error: string | null;
    checkForUpdates: ReturnType<typeof vi.fn>;
    downloadAndInstall: ReturnType<typeof vi.fn>;
    restartApp: ReturnType<typeof vi.fn>;
    dismissUpdate: ReturnType<typeof vi.fn>;
} = {
    status: "idle",
    updateInfo: null,
    progress: null,
    error: null,
    checkForUpdates: checkForUpdatesMock,
    downloadAndInstall: vi.fn(),
    restartApp: vi.fn(),
    dismissUpdate: vi.fn(),
};

vi.mock("@/components/updater-provider", () => ({
    useUpdater: () => updaterMockState,
}));

describe("Settings screen (AC-22)", () => {
    beforeEach(() => {
        currentMockTheme = "light";
        getSettingsMock.mockReset();
        setSettingsMock.mockReset();
        dbPathMock.mockReset();
        checkForUpdatesMock.mockReset();
        updaterMockState = {
            status: "idle",
            updateInfo: null,
            progress: null,
            error: null,
            checkForUpdates: checkForUpdatesMock,
            downloadAndInstall: vi.fn(),
            restartApp: vi.fn(),
            dismissUpdate: vi.fn(),
        };
        getSettingsMock.mockResolvedValue(defaultSettings);
        setSettingsMock.mockResolvedValue(undefined);
        dbPathMock.mockResolvedValue("/tmp/mylan.db");
        getMonitoringStatusMock.mockReset();
        toggleMonitoringPauseMock.mockReset();
        triggerBackgroundScanMock.mockReset();
        onMonitoringStatusMock.mockReset();
        getMonitoringStatusMock.mockResolvedValue({
            enabled: true,
            paused: false,
            is_scanning: false,
            interval_secs: 300,
            last_scan_at: "2026-10-16T10:00:00Z",
            next_scan_in_secs: 180,
            current_network: "Wi-Fi Test",
        });
        toggleMonitoringPauseMock.mockResolvedValue(true);
        triggerBackgroundScanMock.mockResolvedValue(undefined);
        onMonitoringStatusMock.mockResolvedValue(() => {});
        getGdriveSyncStatusMock.mockReset();
        getGdriveSyncStatusMock.mockResolvedValue({
            is_authenticated: false,
            email: null,
            has_master_key: false,
            last_synced_at: null,
            last_synced_epoch: null,
        });
        syncWorkspaceMock.mockReset();
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

    it("carga el db_path desde el backend al montar y lo muestra en el campo de almacenamiento", async () => {
        renderWithProviders(<Settings />);
        await waitFor(() => {
            expect(getSettingsMock).toHaveBeenCalled();
            expect(dbPathMock).toHaveBeenCalled();
        });
        expect(
            screen.getByLabelText("Ruta de la base de datos"),
        ).toHaveValue("/tmp/mylan.db");
    });

    it("persiste inmediatamente el perfil de descubrimiento al seleccionarlo en la cuadrícula", async () => {
        renderWithProviders(<Settings />);
        const deepRadio = await screen.findByRole("radio", {
            name: /Profundo/,
        });
        fireEvent.click(deepRadio);
        await waitFor(() => {
            expect(setSettingsMock).toHaveBeenCalledWith(
                expect.objectContaining({ default_profile: "deep" }),
            );
        });
    });

    it("muestra mensaje de error claro y acción de reintentar cuando falla la comprobación", async () => {
        updaterMockState = {
            status: "error",
            updateInfo: null,
            progress: null,
            error: "Fallo de conexión con el servidor",
            checkForUpdates: checkForUpdatesMock,
            downloadAndInstall: vi.fn(),
            restartApp: vi.fn(),
            dismissUpdate: vi.fn(),
        };

        renderWithProviders(<Settings />);
        await waitFor(() => {
            expect(
                screen.getByText("Error al comprobar actualizaciones"),
            ).toBeInTheDocument();
        });
        expect(
            screen.getByText("Fallo de conexión con el servidor"),
        ).toBeInTheDocument();

        const retryButton = screen.getByRole("button", { name: "Reintentar" });
        expect(retryButton).toBeInTheDocument();
        fireEvent.click(retryButton);
        expect(checkForUpdatesMock).toHaveBeenCalledWith({ silent: false });
    });

    it("cambia el logo AppMark entre variante clara (#F1F5F9 + #0891B2) y oscura (#090D16 + #06B6D4) según el tema activo", () => {
        currentMockTheme = "light";
        const { container, rerender } = renderWithProviders(<AppMark />);
        const lightSvg = container.querySelector("svg[data-theme-variant='light']");
        expect(lightSvg).not.toBeNull();
        expect(lightSvg?.querySelector("rect")?.getAttribute("fill")).toBe("#F1F5F9");
        expect(lightSvg?.querySelector("circle")?.getAttribute("fill")).toBe("#0891B2");

        currentMockTheme = "dark";
        rerender(<AppMark />);
        const darkSvg = container.querySelector("svg[data-theme-variant='dark']");
        expect(darkSvg).not.toBeNull();
        expect(darkSvg?.querySelector("rect")?.getAttribute("fill")).toBe("#090D16");
        expect(darkSvg?.querySelector("circle")?.getAttribute("fill")).toBe("#06B6D4");
    });

    it("renderiza la sección de monitoreo continuo 24/7 y permite conmutar pausas y frecuencias", async () => {
        renderWithProviders(<Settings />);

        await waitFor(() => {
            expect(
                screen.getByText("Monitoreo continuo 24/7 y bandeja"),
            ).toBeInTheDocument();
        });

        expect(
            screen.getByText("Monitoreando activamente"),
        ).toBeInTheDocument();
        expect(screen.getByText("Wi-Fi Test")).toBeInTheDocument();

        // Pulsar botón de pausar
        const pauseBtn = screen.getByRole("button", { name: /pausar/i });
        fireEvent.click(pauseBtn);
        expect(toggleMonitoringPauseMock).toHaveBeenCalled();

        // Cambiar frecuencia a 10 min
        const intervalBtn = screen.getByRole("radio", { name: /10 min/i });
        fireEvent.click(intervalBtn);
        expect(setSettingsMock).toHaveBeenCalledWith(
            expect.objectContaining({
                monitoring_interval_secs: 600,
            }),
        );
    });

    it("permite disparar un escaneo en segundo plano desde el botón 'Escanear ahora'", async () => {
        renderWithProviders(<Settings />);

        await waitFor(() => {
            expect(
                screen.getByText("Monitoreo continuo 24/7 y bandeja"),
            ).toBeInTheDocument();
        });

        const scanNowBtn = screen.getByRole("button", { name: /escanear ahora/i });
        fireEvent.click(scanNowBtn);
        expect(triggerBackgroundScanMock).toHaveBeenCalled();
    });

    it("renderiza la tarjeta de 'Notificaciones de escritorio' y permite conmutar el switch maestro y disparadores", async () => {
        renderWithProviders(<Settings />);

        await waitFor(() => {
            expect(
                screen.getByText("Notificaciones de escritorio"),
            ).toBeInTheDocument();
        });

        // Switch maestro de notificaciones
        const masterSwitch = screen.getByRole("switch", {
            name: /Habilitar notificaciones del SO/i,
        });
        expect(masterSwitch).toBeInTheDocument();
        expect(masterSwitch).toBeChecked();

        // Switch de dispositivos no reconocidos (visible mientras master esté activo)
        const untrustedSwitch = screen.getByRole("switch", {
            name: /Dispositivos no reconocidos/i,
        });
        expect(untrustedSwitch).toBeInTheDocument();
        expect(untrustedSwitch).toBeChecked();

        fireEvent.click(untrustedSwitch);
        expect(setSettingsMock).toHaveBeenCalledWith(
            expect.objectContaining({
                notify_untrusted_devices: false,
            }),
        );

        // Conmutar el switch maestro a off
        fireEvent.click(masterSwitch);
        expect(setSettingsMock).toHaveBeenCalledWith(
            expect.objectContaining({
                notifications_enabled: false,
            }),
        );

        // Al estar apagado el switch maestro, los disparadores quedan ocultos
        expect(
            screen.queryByRole("switch", {
                name: /Dispositivos no reconocidos/i,
            }),
        ).not.toBeInTheDocument();
    });

    it("renderiza la tarjeta de 'Sincronización en la Nube (Google Drive)'", async () => {
        getGdriveSyncStatusMock.mockResolvedValue({
            is_authenticated: true,
            email: "alice@example.com",
            has_master_key: true,
            last_synced_at: "2026-10-16T12:00:00Z",
            last_synced_epoch: 1792152000,
        });

        renderWithProviders(<Settings />);

        await waitFor(() => {
            expect(
                screen.getByText("Sincronización en la Nube (Google Drive)"),
            ).toBeInTheDocument();
        });

        expect(screen.getByText("alice@example.com")).toBeInTheDocument();
        expect(
            screen.getByText("Clave activa protegida en el llavero del sistema"),
        ).toBeInTheDocument();
        expect(screen.getByText("Sincronizar ahora")).toBeInTheDocument();
    });
});
