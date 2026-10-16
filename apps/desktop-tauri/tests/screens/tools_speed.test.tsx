import { fireEvent, screen, waitFor } from "@testing-library/react";
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
        onSpeedTestDone: vi.fn().mockResolvedValue(() => {}),
        onToolPingPacket: vi.fn().mockResolvedValue(() => {}),
        onToolPingSummary: vi.fn().mockResolvedValue(() => {}),
        onToolTraceHop: vi.fn().mockResolvedValue(() => {}),
        onToolTraceDone: vi.fn().mockResolvedValue(() => {}),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

describe("Tools Screen - Speed & Network Health (Fase 9)", () => {
    beforeEach(() => {
        vi.clearAllMocks();
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
