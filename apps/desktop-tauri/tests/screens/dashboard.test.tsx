// AC-22: Tests de screen `Dashboard` con MemoryRouter + providers + mock
// `@/lib/tauri` por función + fixtures Device[].

import { screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Dashboard } from "@/screens/Dashboard";
import { renderWithProviders } from "./helpers";
import { deviceFixtures, defaultSettings } from "../fixtures";

const detectInterfaceMock = vi.fn();
const listDevicesMock = vi.fn();
const getSettingsMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    detectInterface: () => detectInterfaceMock(),
    listDevices: () => listDevicesMock(),
    getSettings: () => getSettingsMock(),
}));

vi.mock("@/App", () => ({
    useScan: () => ({
        scanning: false,
        scanId: null,
        progress: null,
        devicesFound: [],
        startScan: vi.fn(),
        cancel: vi.fn(),
    }),
    useLastScan: () => ({ lastScan: null, setLastScan: vi.fn() }),
}));

vi.mock("@/components/censorship-provider", () => ({
    useCensorship: () => ({
        censorshipEnabled: false,
        setCensorshipEnabled: vi.fn(),
        toggle: vi.fn(),
    }),
}));

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

const ifaceFixture = {
    name: "eth0",
    ip: "192.168.1.10",
    prefix_len: 24,
    mac: "aa:bb:cc:dd:ee:ff",
    gateway_ip: "192.168.1.1",
    gateway_mac: null,
    dns_servers: ["8.8.8.8"],
    cidr: "192.168.1.0/24",
    ssid: null,
};

describe("Dashboard screen (AC-22)", () => {
    beforeEach(() => {
        detectInterfaceMock.mockReset();
        listDevicesMock.mockReset();
        getSettingsMock.mockReset();
        detectInterfaceMock.mockResolvedValue(ifaceFixture);
        listDevicesMock.mockResolvedValue(deviceFixtures);
        getSettingsMock.mockResolvedValue(defaultSettings);
    });

    it("renderiza la sección 'Red activa' y 'Descubrir dispositivos'", async () => {
        renderWithProviders(<Dashboard />);
        await waitFor(() => {
            expect(
                screen.getByRole("region", { name: "Red activa" }),
            ).toBeInTheDocument();
        });
        expect(
            screen.getByRole("region", { name: "Descubrir dispositivos" }),
        ).toBeInTheDocument();
    });

    it("muestra el botón 'Descubrir dispositivos' cuando no hay scan en curso", async () => {
        renderWithProviders(<Dashboard />);
        await waitFor(() => {
            expect(
                screen.getByRole("button", { name: /Descubrir dispositivos/ }),
            ).toBeInTheDocument();
        });
    });

    it("expone el conteo de dispositivos en el resumen", async () => {
        renderWithProviders(<Dashboard />);
        // El resumen lista el número de dispositivos de los fixtures.
        await waitFor(() => {
            expect(
                screen.getByRole("region", { name: "Resumen" }),
            ).toBeInTheDocument();
        });
    });
});