// AC-22 / ADR-0007: Tests de la pantalla principal unificada `Dashboard` (Red)
// con MemoryRouter + providers + mock `@/lib/tauri` por función + fixtures Device[].

import { screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Dashboard } from "@/screens/Dashboard";
import { renderWithProviders } from "./helpers";
import { NetworkNameProvider } from "@/lib/use-network-name";
import { deviceFixtures, defaultSettings } from "../fixtures";

const detectInterfaceMock = vi.fn();
const listDevicesMock = vi.fn();
const getSettingsMock = vi.fn();
const getNetworkNameMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    detectInterface: () => detectInterfaceMock(),
    listDevices: () => listDevicesMock(),
    getSettings: () => getSettingsMock(),
    getNetworkName: (cidr: string) => getNetworkNameMock(cidr),
    getSecurityScore: vi.fn().mockResolvedValue(100),
    getSecurityFindings: vi.fn().mockResolvedValue([]),
    runSecurityAudit: vi.fn().mockResolvedValue({ score: 100, findings: [] }),
    onScanFinished: vi.fn().mockResolvedValue(() => {}),
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
    deviceKey: (d: {
        primary_ip: string | null;
        primary_mac: string | null;
        id: string;
    }) => d.primary_ip ?? d.primary_mac ?? d.id,
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

describe("Dashboard screen (AC-22 / ADR-0007)", () => {
    beforeEach(() => {
        detectInterfaceMock.mockReset();
        listDevicesMock.mockReset();
        getSettingsMock.mockReset();
        getNetworkNameMock.mockReset();
        detectInterfaceMock.mockResolvedValue(ifaceFixture);
        listDevicesMock.mockResolvedValue(deviceFixtures);
        getSettingsMock.mockResolvedValue(defaultSettings);
        getNetworkNameMock.mockResolvedValue({
            name: "Casa-5G",
            source: "auto",
        });
    });

    it("renderiza la sección 'Red activa' con datos técnicos visibles y la sección 'Dispositivos'", async () => {
        renderWithProviders(
            <NetworkNameProvider>
                <Dashboard />
            </NetworkNameProvider>,
        );
        await waitFor(() => {
            expect(
                screen.getByRole("region", { name: "Red activa" }),
            ).toBeInTheDocument();
        });
        const netSection = screen.getByRole("region", { name: "Red activa" });
        await waitFor(() => {
            expect(within(netSection).getByText("eth0")).toBeInTheDocument();
            expect(
                within(netSection).getByText("192.168.1.10"),
            ).toBeInTheDocument();
        });
        expect(
            screen.getByRole("region", { name: "Dispositivos" }),
        ).toBeInTheDocument();
    });

    it("muestra el botón 'Descubrir dispositivos' junto al selector de perfil cuando no hay scan en curso", async () => {
        renderWithProviders(
            <NetworkNameProvider>
                <Dashboard />
            </NetworkNameProvider>,
        );
        await waitFor(() => {
            expect(
                screen.getByRole("button", { name: /Descubrir dispositivos/ }),
            ).toBeInTheDocument();
        });
        expect(
            screen.getByRole("combobox", { name: "Perfil de escaneo" }),
        ).toBeInTheDocument();
    });

    it("integra la vista previa de dispositivos con conteo y enlace 'Ver todos' hacia /devices", async () => {
        renderWithProviders(
            <NetworkNameProvider>
                <Dashboard />
            </NetworkNameProvider>,
        );
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });
        const viewAllLink = screen.getByRole("link", { name: /Ver todos/ });
        expect(viewAllLink).toHaveAttribute("href", "/devices");
    });
});
