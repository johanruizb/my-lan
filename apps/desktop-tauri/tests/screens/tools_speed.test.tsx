import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Tools } from "@/screens/Tools";
import { renderWithProviders } from "./helpers";
import { CensorshipProvider } from "@/components/censorship-provider";
import type { NetworkHealthSample, SpeedTestResult } from "@/lib/tauri";

const mockStartSpeedTest = vi.fn();
const mockCancelTool = vi.fn();
const mockGetRecentSpeedTests = vi.fn();
const mockGetNetworkHealthSamples = vi.fn();
const mockRunHealthProbe = vi.fn();
const toastMock = vi.fn();
let speedTestDoneCallbacks: ((payload: any) => void)[] = [];

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
        onSpeedTestDone: (cb: (payload: any) => void) => {
            speedTestDoneCallbacks.push(cb);
            // Simula la asincronía real de Tauri: el unlisten llega luego de un microtask.
            return new Promise<() => void>((resolve) => {
                setTimeout(() => {
                    resolve(() => {
                        speedTestDoneCallbacks = speedTestDoneCallbacks.filter((c) => c !== cb);
                    });
                }, 5);
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
        speedTestDoneCallbacks = [];
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
            expect(speedTestDoneCallbacks.length).toBeGreaterThan(0);
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
            speedTestDoneCallbacks.forEach((cb) => cb({ session_id: sessionId, result, error: null }));
            speedTestDoneCallbacks.forEach((cb) => cb({ session_id: sessionId, result, error: null }));
        });

        await waitFor(() => {
            expect(toastMock).toHaveBeenCalledTimes(1);
            expect(toastMock).toHaveBeenCalledWith(
                "Test de velocidad completado exitosamente.",
                "success",
            );
        });
    });

    it("evita toasts duplicados ante doble suscripción por re-montaje del panel", async () => {
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockStartSpeedTest.mockResolvedValue(undefined);

        const { unmount } = renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Iniciar Test/i })).toBeInTheDocument();
        });

        fireEvent.click(screen.getByRole("button", { name: /Iniciar Test/i }));

        await waitFor(() => {
            expect(mockStartSpeedTest).toHaveBeenCalled();
        });

        const firstSessionId = mockStartSpeedTest.mock.calls[0][0];

        // Simular re-montaje rápido (StrictMode / cambio de pestaña): desmontar
        // ANTES de que Tauri resuelva el unlisten, reproduciendo el escenario
        // que deja listeners colgados en la app real.
        unmount();

        // Esperar a que la promesa del primer listener se resuelva y se limpie.
        await new Promise((resolve) => setTimeout(resolve, 20));

        renderWithProviders(
            <CensorshipProvider initialEnabled={false}>
                <Tools />
            </CensorshipProvider>,
            ["/tools?tool=speed"],
        );

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Iniciar Test/i })).toBeInTheDocument();
        });

        fireEvent.click(screen.getByRole("button", { name: /Iniciar Test/i }));

        await waitFor(() => {
            expect(mockStartSpeedTest).toHaveBeenCalledTimes(2);
            expect(speedTestDoneCallbacks.length).toBe(1);
        });

        const secondSessionId = mockStartSpeedTest.mock.calls[1][0];
        const result: SpeedTestResult = {
            id: "st-remount",
            network_id: "net-1",
            server_name: "Cloudflare (BOG)",
            server_host: "speed.cloudflare.com",
            server_city: "Bogotá",
            server_country: "CO",
            download_mbps: 200.0,
            upload_mbps: 100.0,
            latency_ms: 8,
            jitter_ms: 0.5,
            bufferbloat_ms: null,
            retransmits: null,
            protocol: "cloudflare",
            created_at: "2026-10-16T12:00:00Z",
        };

        act(() => {
            speedTestDoneCallbacks.forEach((cb) => cb({ session_id: secondSessionId, result, error: null }));
        });

        await waitFor(() => {
            expect(toastMock).toHaveBeenCalledTimes(1);
            expect(toastMock).toHaveBeenCalledWith(
                "Test de velocidad completado exitosamente.",
                "success",
            );
        });

        // El listener del primer montaje debe haber quedado inactivo: eventos de su sesión no generan toast.
        expect(toastMock).not.toHaveBeenCalledWith(
            expect.stringContaining(firstSessionId),
            expect.anything(),
        );
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
});
