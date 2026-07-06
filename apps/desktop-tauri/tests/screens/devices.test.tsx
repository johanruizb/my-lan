// AC-22: Tests de screen `Devices` con MemoryRouter + providers + mock
// `@/lib/tauri` por función + fixtures Device[].

import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Devices } from "@/screens/Devices";
import { renderWithProviders } from "./helpers";
import { deviceFixtures } from "../fixtures";

const listDevicesMock = vi.fn();
const exportDevicesMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    listDevices: () => listDevicesMock(),
    exportDevices: (format: string) => exportDevicesMock(format),
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

describe("Devices screen (AC-22)", () => {
    beforeEach(() => {
        listDevicesMock.mockReset();
        exportDevicesMock.mockReset();
        listDevicesMock.mockResolvedValue(deviceFixtures);
        exportDevicesMock.mockResolvedValue("/tmp/export.csv");
    });

    it("renderiza y muestra el conteo de dispositivos de los fixtures", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });
    });

    it("el input de búsqueda filtra la lista por hostname", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        const input = screen.getByLabelText("Buscar dispositivos");
        fireEvent.change(input, { target: { value: "router" } });

        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(1\)/)).toBeInTheDocument();
        });
    });

    it("muestra el botón Escanear cuando no hay scan en curso", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(
                screen.getByRole("button", { name: /Escanear/ }),
            ).toBeInTheDocument();
        });
    });
});