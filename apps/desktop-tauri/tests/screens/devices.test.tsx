// AC-18/AC-22: Tests de screen `Devices` con MemoryRouter + providers + mock
// `@/lib/tauri` por función + fixtures Device[].

import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Devices } from "@/screens/Devices";
import { renderWithProviders } from "./helpers";
import { deviceFixtures } from "../fixtures";

// Polyfill pointer capture + scrollIntoView para Radix Select en jsdom (jsdom
// no implementa `hasPointerCapture`/`releasePointerCapture`/`scrollIntoView`;
// Radix los usa al abrir el Select y al enfocar la opción seleccionada).
if (!Element.prototype.hasPointerCapture) {
    Element.prototype.hasPointerCapture = (() => false) as () => boolean;
}
if (!Element.prototype.releasePointerCapture) {
    Element.prototype.releasePointerCapture = (() => {}) as () => void;
}
if (!Element.prototype.scrollIntoView) {
    Element.prototype.scrollIntoView = (() => {}) as () => void;
}

const listDevicesMock = vi.fn();
const exportDevicesMock = vi.fn();
const updateDeviceMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    listDevices: () => listDevicesMock(),
    exportDevices: (format: string) => exportDevicesMock(format),
    updateDevice: (id: string, fields: { displayName?: string; isTrusted?: boolean; notes?: string }) =>
        updateDeviceMock(id, fields),
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
        updateDeviceMock.mockReset();
        listDevicesMock.mockResolvedValue(deviceFixtures);
        exportDevicesMock.mockResolvedValue("/tmp/export.csv");
        updateDeviceMock.mockResolvedValue(deviceFixtures[0]);
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

        // #26: Filtros siempre visibles (sin Collapsible): el input es accesible
        // directamente sin abrir un panel.
        const input = screen.getByLabelText("Buscar dispositivos");
        fireEvent.change(input, { target: { value: "router" } });

        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(1\)/)).toBeInTheDocument();
        });
    });

    // #15: El descubrimiento se lanza desde el Dashboard; Devices solo Refrescar.
    it("muestra el botón Refrescar y omite el botón Escanear", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(
                screen.getByRole("button", { name: /Refrescar/ }),
            ).toBeInTheDocument();
        });
        expect(
            screen.queryByRole("button", { name: /Escanear/ }),
        ).not.toBeInTheDocument();
    });

    // AC-18: OnlineBadge renderiza "En línea" (online) / "Fuera de línea" (offline).
    it("OnlineBadge renderiza En línea para dispositivos online y Fuera de línea para offline", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        const cards = screen.getAllByRole("listitem");
        expect(cards).toHaveLength(3);
        // dev-1 y dev-2 online → "En línea"; dev-3 offline → "Fuera de línea".
        expect(within(cards[0]).getByText("En línea")).toBeInTheDocument();
        expect(within(cards[2]).getByText("Fuera de línea")).toBeInTheDocument();
    });

    // ADR-0006/T6: TrustBadge es manual binario (is_trusted) → Confiable / No
    // confiable. Sin estado "Reconocido" ni derivación de confidence.
    it("TrustBadge renderiza Confiable/No confiable según is_trusted", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        const cards = screen.getAllByRole("listitem");
        // dev-1 is_trusted=true → "Confiable"; dev-2/dev-3 is_trusted=false →
        // "No confiable".
        expect(within(cards[0]).getByText("Confiable")).toBeInTheDocument();
        expect(within(cards[1]).getByText("No confiable")).toBeInTheDocument();
        expect(within(cards[2]).getByText("No confiable")).toBeInTheDocument();
    });

    // AC-15/AC-18: filtro Estado "En línea" oculta offline; "Todos" los restaura.
    it("filtro Estado En línea muestra solo dispositivos online y Todos los restaura", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        // AC-15: sin filtro activo se ven todos (offline incluidos).
        expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();

        // #26: Filtros siempre visibles (sin Collapsible).
        // Toggle "En línea" → solo online (dev-1, dev-2) → 2 visibles.
        fireEvent.click(screen.getByRole("button", { name: "En línea" }));
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(2\)/)).toBeInTheDocument();
        });

        // Toggle "Todos" (Estado) → se restauran los 3 (offline incluido).
        // "Todos" aparece en los grupos Estado y Confianza; acotamos al grupo.
        const estadoGroup = screen.getByRole("group", { name: "Filtrar por estado" });
        fireEvent.click(
            within(estadoGroup).getByRole("button", { name: "Todos" }),
        );
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });
    });

    // ADR-0006/T14: filtro Confiable (binario is_trusted) "Confiables" muestra
    // solo el dispositivo con is_trusted=true (dev-1).
    it("filtro Confiable Confiables muestra solo dispositivos confiables", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        // #26: Filtros siempre visibles (sin Collapsible).
        fireEvent.click(screen.getByRole("button", { name: "Confiables" }));
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(1\)/)).toBeInTheDocument();
        });
    });

    // AC-14/AC-18: filtro Tipo "router" muestra solo dispositivos router.
    it("filtro Tipo router muestra solo dispositivos router", async () => {
        const user = userEvent.setup();
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        // #26: Filtros siempre visibles (sin Collapsible).
        // Radix Select: userEvent.click dispara la secuencia pointer+mouse
        // completa (jsdom no soporta `hasPointerCapture` con fireEvent solo).
        const trigger = screen.getByRole("combobox", { name: "Filtrar por tipo" });
        await user.click(trigger);

        const option = await screen.findByRole("option", { name: "Router" });
        await user.click(option);

        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(1\)/)).toBeInTheDocument();
        });
    });

    // AC-11/AC-18: el título de la tarjeta prioriza display_name sobre la IP.
    it("título de la tarjeta prioriza display_name sobre la IP", async () => {
        renderWithProviders(<Devices />);
        await waitFor(() => {
            expect(screen.getByText(/Dispositivos \(3\)/)).toBeInTheDocument();
        });

        const cards = screen.getAllByRole("listitem");
        // dev-1 tiene display_name="Router principal" → título de la tarjeta.
        expect(within(cards[0]).getByText("Router principal")).toBeInTheDocument();
        // La IP se renderiza como texto secundario en la tarjeta.
        expect(within(cards[0]).getByText("192.168.1.1")).toBeInTheDocument();
    });
});