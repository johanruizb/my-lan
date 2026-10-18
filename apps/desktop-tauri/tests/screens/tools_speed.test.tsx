import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Tools } from "@/screens/Tools";
import { renderWithProviders } from "./helpers";
import { CensorshipProvider } from "@/components/censorship-provider";
import type {
    NetworkHealthSample,
    SpeedTestDoneEvent,
    SpeedTestResult,
} from "@/lib/tauri";

const mockStartSpeedTest = vi.fn();
const mockCancelTool = vi.fn();
const mockGetRecentSpeedTests = vi.fn();
const mockGetNetworkHealthSamples = vi.fn();
const mockRunHealthProbe = vi.fn();
const toastMock = vi.fn();
// Callbacks registrados como Tauri: cada listen() añade el callback a la
// lista de listeners activos y devuelve el unlisten en una Promise que
// resuelve DESPUÉS (round-trip IPC), reproduciendo el race de StrictMode.
const activeDoneCbs: Array<(payload: SpeedTestDoneEvent) => void> = [];
let speedTestDoneCb: ((payload: SpeedTestDoneEvent) => void) | null = null;

vi.mock("@/lib/tauri", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/lib/tauri")>();
    return {
        ...actual,
        startSpeedTest: (sessionId: string, networkId?: string) =>
            mockStartSpeedTest(sessionId, networkId),
        cancelTool: (sessionId: string) => mockCancelTool(sessionId),
        getRecentSpeedTests: (params?: { networkId?: string; limit?: number }) =>
            mockGetRecentSpeedTests(params),
        getNetworkHealthSamples: (networkId?: string, limit?: number) =>
            mockGetNetworkHealthSamples(networkId, limit),
        runHealthProbe: (networkId?: string) => mockRunHealthProbe(networkId),
        onSpeedTestProgress: vi.fn().mockResolvedValue(() => {}),
        onSpeedTestDone: (cb: (payload: SpeedTestDoneEvent) => void) => {
            speedTestDoneCb = cb;
            activeDoneCbs.push(cb);
            return new Promise((resolve) => {
                setTimeout(() => resolve(() => {
                    const idx = activeDoneCbs.indexOf(cb);
                    if (idx >= 0) activeDoneCbs.splice(idx, 1);
                }), 0);
            });
        },
        onToolPingPacket: vi.fn().mockResolvedValue(() => {}),
        onToolPingSummary: vi.fn().mockResolvedValue(() => {}),
        onToolTraceHop: vi.fn().mockResolvedValue(() => {}),
        onToolTraceDone: vi.fn().mockResolvedValue(() => {}),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: toastMock }),
}));

describe("Tools Screen - Speed & Network Health (Fase 9)", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        speedTestDoneCb = null;
        activeDoneCbs.length = 0;
    });

    it("muestra un único toast cuando el evento speedtest_done se dispara dos veces para la misma sesión", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockStartSpeedTest.mockResolvedValue(undefined);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Iniciar Test/i })).toBeInTheDocument();
        });

        const startBtn = screen.getByRole("button", { name: /Iniciar Test/i });
        fireEvent.click(startBtn);

        await waitFor(() => {
            expect(mockStartSpeedTest).toHaveBeenCalled();
            expect(speedTestDoneCb).toBeDefined();
        });

        const sessionId = mockStartSpeedTest.mock.calls[0][0];
        const result: SpeedTestResult = {
            id: "st-dup",
            network_id: "net-1",
            server_name: "Cloudflare (BOG)",
            server_host: "speed.cloudflare.com",
            server_city: "Bogotá",
            server_country: "CO",
            download_mbps: 100.0,
            upload_mbps: 50.0,
            latency_ms: 10,
            jitter_ms: 1.0,
            bufferbloat_ms: null,
            retransmits: null,
            protocol: "cloudflare",
            created_at: "2026-10-16T12:00:00Z",
        };

        act(() => {
            speedTestDoneCb!({ session_id: sessionId, result, error: null });
            speedTestDoneCb!({ session_id: sessionId, result, error: null });
        });

        await waitFor(() => {
            expect(toastMock).toHaveBeenCalledTimes(1);
            expect(toastMock).toHaveBeenCalledWith(
                "Test de velocidad completado exitosamente.",
                "success",
            );
        });
    });

    it("un solo toast cuando StrictMode deja listeners zombie por el race de unlisten", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockStartSpeedTest.mockResolvedValue(undefined);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
            { strictMode: true },
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Iniciar Test/i })).toBeInTheDocument();
        });

        // Flush de micro/macro-tareas: las Promises de listen() resuelven tras
        // el primer cleanup de StrictMode (round-trip IPC simulado).
        await act(async () => {
            await new Promise((r) => setTimeout(r, 0));
        });

        // Con el patrón disposed, el listener del primer setup se desregistra
        // solo cuando su Promise resuelve tras el cleanup: queda EXACTAMENTE 1.
        expect(activeDoneCbs.length).toBe(1);

        fireEvent.click(screen.getByRole("button", { name: /Iniciar Test/i }));
        await waitFor(() => expect(mockStartSpeedTest).toHaveBeenCalled());

        const sessionId = mockStartSpeedTest.mock.calls[0][0];
        const result: SpeedTestResult = {
            id: "st-strict",
            network_id: "net-1",
            server_name: "Cloudflare (BOG)",
            server_host: "speed.cloudflare.com",
            server_city: "Bogotá",
            server_country: "CO",
            download_mbps: 120.0,
            upload_mbps: 60.0,
            latency_ms: 9,
            jitter_ms: 0.9,
            bufferbloat_ms: null,
            retransmits: null,
            protocol: "cloudflare",
            created_at: "2026-10-16T12:30:00Z",
        };

        await act(async () => {
            activeDoneCbs.slice().forEach((cb) =>
                cb({ session_id: sessionId, result, error: null }),
            );
            await Promise.resolve();
        });

        await waitFor(() => {
            expect(toastMock).toHaveBeenCalledTimes(1);
        });
    });

    it("renderiza la pestaña 'Velocidad y Salud' y muestra métricas históricas", async () => {
        const recentTests: SpeedTestResult[] = [
            {
                id: "st-1",
                network_id: "net-1",
                server_name: "Cloudflare (BOG)",
                server_host: "speed.cloudflare.com",
                server_city: "Bogotá",
                server_country: "CO",
                download_mbps: 185.5,
                upload_mbps: 42.0,
                latency_ms: 12,
                jitter_ms: 1.5,
                bufferbloat_ms: null,
                retransmits: null,
                protocol: "cloudflare",
                created_at: "2026-10-16T12:00:00Z",
            },
        ];

        const healthSamples: NetworkHealthSample[] = [
            {
                id: "h-1",
                network_id: "net-1",
                target_type: "gateway",
                target_ip: "192.168.1.1",
                rtt_ms: 2,
                jitter_ms: 0.4,
                loss_pct: 0.0,
                sample_count: 4,
                probes_failed: 0,
                method: "icmp",
                created_at: "2026-10-16T12:00:00Z",
            },
            {
                id: "h-2",
                network_id: "net-1",
                target_type: "dns",
                target_ip: "1.1.1.1",
                rtt_ms: 14,
                jitter_ms: 1.2,
                loss_pct: 0.0,
                sample_count: 4,
                probes_failed: 0,
                method: "icmp",
                created_at: "2026-10-16T12:00:00Z",
            },
        ];

        mockGetRecentSpeedTests.mockResolvedValue(recentTests);
        mockGetNetworkHealthSamples.mockResolvedValue(healthSamples);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByText(/Test de Velocidad \(speed.cloudflare.com\)/i)).toBeInTheDocument();
            expect(screen.getByText("185.5")).toBeInTheDocument();
            expect(screen.getByText("42.0")).toBeInTheDocument();
            expect(screen.getByText("12")).toBeInTheDocument();
            expect(screen.getByText("1.5")).toBeInTheDocument();
            expect(screen.getByText("Router / Gateway Local")).toBeInTheDocument();
            expect(screen.getByText("Servidor DNS / Enlace Salida")).toBeInTheDocument();
        });
    });

    it("permite iniciar una medición con 'Iniciar Test' y llama a startSpeedTest", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockStartSpeedTest.mockResolvedValue(undefined);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Iniciar Test/i })).toBeInTheDocument();
        });

        const startBtn = screen.getByRole("button", { name: /Iniciar Test/i });
        fireEvent.click(startBtn);

        await waitFor(() => {
            expect(mockStartSpeedTest).toHaveBeenCalled();
        });
    });

    it("permite sondear salud bajo demanda con 'Sondear Ahora'", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockRunHealthProbe.mockResolvedValue([
            {
                id: "probe-gw",
                network_id: "net-1",
                target_type: "gateway",
                target_ip: "192.168.1.1",
                rtt_ms: 3,
                jitter_ms: 0.8,
                loss_pct: 0.0,
                sample_count: 4,
                probes_failed: 0,
                method: "icmp",
                created_at: "2026-10-16T12:05:00Z",
            },
        ]);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Sondear Ahora/i })).toBeInTheDocument();
        });

        const probeBtn = screen.getByRole("button", { name: /Sondear Ahora/i });
        fireEvent.click(probeBtn);

        await waitFor(() => {
            expect(mockRunHealthProbe).toHaveBeenCalled();
            expect(screen.getAllByText("192.168.1.1").length).toBeGreaterThan(0);
        });
    });

    it("enmascara IPs cuando el Modo Censura está habilitado", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([
            {
                id: "h-censor",
                network_id: "net-1",
                target_type: "gateway",
                target_ip: "192.168.1.254",
                rtt_ms: 4,
                jitter_ms: 0.5,
                loss_pct: 0.0,
                sample_count: 4,
                probes_failed: 0,
                method: "icmp",
                created_at: "2026-10-16T12:00:00Z",
            },
        ]);

        renderWithProviders(
            <CensorshipProvider initialEnabled={true}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.queryByText("192.168.1.254")).not.toBeInTheDocument();
            expect(screen.getAllByText("192.168.*.*").length).toBeGreaterThan(0);
        });
    });

    it("identifica y marca muestras previas con DNS loopback (stub local) en vez de calificarlas como excelente", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([
            {
                id: "h-loopback",
                network_id: "net-1",
                target_type: "dns",
                target_ip: "127.0.0.53",
                rtt_ms: 0,
                jitter_ms: 0.0,
                loss_pct: 0.0,
                sample_count: 4,
                probes_failed: 0,
                method: "icmp",
                created_at: "2026-10-16T12:00:00Z",
            },
        ]);

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByText("Stub Local (No representativo)")).toBeInTheDocument();
            expect(screen.getByText("stub local")).toBeInTheDocument();
            expect(screen.getByText("0 ms (loopback)")).toBeInTheDocument();
        });
    });
});
