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
});