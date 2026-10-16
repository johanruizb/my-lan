// AC-18: Test del formulario de edición de `DeviceDetail` (#19 edición híbrida).
// Verifica que al pulsar "Editar", rellenar `display_name` y pulsar "Guardar"
// se llama a `updateDevice` con el `id` del device y el field `displayName`.

import { fireEvent, screen, waitFor } from "@testing-library/react";
import { Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DeviceDetail } from "@/screens/DeviceDetail";
import { renderWithProviders } from "./helpers";
import { makeDevice } from "../fixtures";

const getDeviceMock = vi.fn();
const updateDeviceMock = vi.fn();
const cancelScanMock = vi.fn();
const scanPortsMock = vi.fn();
const exportServicesMock = vi.fn();

vi.mock("@/lib/tauri", () => ({
    getDevice: (ip: string) => getDeviceMock(ip),
    updateDevice: (
        id: string,
        fields: { displayName?: string; isTrusted?: boolean; notes?: string },
    ) => updateDeviceMock(id, fields),
    cancelScan: (scanId: string) => cancelScanMock(scanId),
    scanPorts: (ip: string, profile: string, scanId: string) =>
        scanPortsMock(ip, profile, scanId),
    exportServices: (format: string) => exportServicesMock(format),
    // Listeners de scan: no-ops (no se disparan en este test).
    onScanProgress: () => Promise.resolve(() => {}),
    onScanHeartbeat: () => Promise.resolve(() => {}),
    onScanCancelled: () => Promise.resolve(() => {}),
    onScanFinished: () => Promise.resolve(() => {}),
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

const fixtureDevice = makeDevice({
    id: "dev-edit-1",
    primary_ip: "192.168.1.1",
    hostname: "router.lan",
    display_name: "Nombre viejo",
    device_type: "router",
    confidence: "90",
    is_trusted: false,
    is_online: true,
});

describe("DeviceDetail screen (AC-18)", () => {
    beforeEach(() => {
        getDeviceMock.mockReset();
        updateDeviceMock.mockReset();
        cancelScanMock.mockReset();
        scanPortsMock.mockReset();
        exportServicesMock.mockReset();
        getDeviceMock.mockResolvedValue({
            device: fixtureDevice,
            services: [],
        });
        updateDeviceMock.mockResolvedValue(fixtureDevice);
    });

    it("carga el dispositivo y el formulario llama a updateDevice con id y displayName", async () => {
        renderWithProviders(
            <Routes>
                <Route path="/devices/:ip" element={<DeviceDetail />} />
            </Routes>,
            ["/devices/192.168.1.1"],
        );

        // #19 edición híbrida: por defecto modo lectura (botón "Editar").
        const editButton = await screen.findByRole("button", {
            name: /^Editar$/,
        });
        fireEvent.click(editButton);

        // Rellena el input de display_name (revelado al entrar en edición).
        const input = await screen.findByLabelText("Nombre personalizado");
        fireEvent.change(input, { target: { value: "Nombre nuevo" } });

        // Click botón "Guardar".
        const saveButton = screen.getByRole("button", { name: /^Guardar$/ });
        fireEvent.click(saveButton);

        // Verifica updateDevice fue llamado con el id del device y displayName.
        await waitFor(() => {
            expect(updateDeviceMock).toHaveBeenCalledWith(
                "dev-edit-1",
                expect.objectContaining({ displayName: "Nombre nuevo" }),
            );
        });
    });

    it("separa servicios abiertos de puertos cerrados o filtrados en un collapsible", async () => {
        getDeviceMock.mockResolvedValue({
            device: fixtureDevice,
            services: [
                {
                    id: "svc-1",
                    device_id: "dev-edit-1",
                    protocol: "tcp",
                    port: 22,
                    service_name: "ssh",
                    product: "OpenSSH",
                    version: "9.2",
                    banner: "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3",
                    state: "open",
                    first_seen_at: "2026-01-01T00:00:00Z",
                    last_seen_at: "2026-01-01T00:00:00Z",
                },
                {
                    id: "svc-2",
                    device_id: "dev-edit-1",
                    protocol: "tcp",
                    port: 80,
                    service_name: "http",
                    product: null,
                    version: null,
                    banner: null,
                    state: "closed",
                    first_seen_at: "2026-01-01T00:00:00Z",
                    last_seen_at: "2026-01-01T00:00:00Z",
                },
            ],
        });

        renderWithProviders(
            <Routes>
                <Route path="/devices/:ip" element={<DeviceDetail />} />
            </Routes>,
            ["/devices/192.168.1.1"],
        );

        // Verifica que la card principal muestra Servicios abiertos (1)
        await waitFor(() => {
            expect(
                screen.getByText(/Servicios abiertos \(1\)/),
            ).toBeInTheDocument();
        });

        // Verifica que el puerto 22 está en el documento
        expect(screen.getByText("22")).toBeInTheDocument();

        // El collapsible muestra "Puertos cerrados o filtrados (1)"
        expect(
            screen.getByText(/Puertos cerrados o filtrados \(1\)/),
        ).toBeInTheDocument();
    });

    it("no duplica la IP cuando no hay display_name ni hostname", async () => {
        const noNameDevice = makeDevice({
            id: "dev-noname-1",
            primary_ip: "10.0.0.42",
            hostname: null,
            display_name: null,
            primary_mac: "00:11:22:33:44:55",
            vendor: "Acme Corp",
        });

        getDeviceMock.mockResolvedValue({
            device: noNameDevice,
            services: [],
        });

        renderWithProviders(
            <Routes>
                <Route path="/devices/:ip" element={<DeviceDetail />} />
            </Routes>,
            ["/devices/10.0.0.42"],
        );

        await waitFor(() => {
            expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
                "10.0.0.42",
            );
        });

        // Verifica que 10.0.0.42 solo aparece en el título h1 y no duplicado como subtitle
        const allIpInstances = screen.getAllByText("10.0.0.42");
        expect(allIpInstances).toHaveLength(1);
    });

    it("permite expandir y contraer el banner largo del servicio", async () => {
        getDeviceMock.mockResolvedValue({
            device: fixtureDevice,
            services: [
                {
                    id: "svc-1",
                    device_id: "dev-edit-1",
                    protocol: "tcp",
                    port: 22,
                    service_name: "ssh",
                    product: "OpenSSH",
                    version: "9.2",
                    banner: "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3 very long banner content",
                    state: "open",
                    first_seen_at: "2026-01-01T00:00:00Z",
                    last_seen_at: "2026-01-01T00:00:00Z",
                },
            ],
        });

        renderWithProviders(
            <Routes>
                <Route path="/devices/:ip" element={<DeviceDetail />} />
            </Routes>,
            ["/devices/192.168.1.1"],
        );

        await waitFor(() => {
            expect(
                screen.getByRole("button", { name: "Más" }),
            ).toBeInTheDocument();
        });

        fireEvent.click(screen.getByRole("button", { name: "Más" }));
        expect(
            screen.getByRole("button", { name: "Menos" }),
        ).toBeInTheDocument();
    });

    it("permite alternar confianza desde la cabecera y lanzar escaneo de puertos desde la barra integrada", async () => {
        scanPortsMock.mockResolvedValue({});

        renderWithProviders(
            <Routes>
                <Route path="/devices/:ip" element={<DeviceDetail />} />
            </Routes>,
            ["/devices/192.168.1.1"],
        );

        const trustSwitch = await screen.findByRole("switch", {
            name: /Dispositivo confiable/i,
        });
        fireEvent.click(trustSwitch);

        await waitFor(() => {
            expect(updateDeviceMock).toHaveBeenCalledWith("dev-edit-1", {
                isTrusted: true,
            });
        });

        const scanButton = screen.getByRole("button", {
            name: /Escanear puertos/i,
        });
        fireEvent.click(scanButton);

        await waitFor(() => {
            expect(scanPortsMock).toHaveBeenCalledWith(
                "192.168.1.1",
                "normal",
                expect.any(String),
            );
        });
    });
});
