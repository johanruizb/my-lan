import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Tools } from "@/screens/Tools";
import { renderWithProviders } from "./helpers";

const startToolPingMock = vi.fn().mockResolvedValue(undefined);
const startToolTracerouteMock = vi.fn().mockResolvedValue(undefined);
const cancelToolMock = vi.fn().mockResolvedValue(true);
const dnsLookupMock = vi.fn();
const sendWolMock = vi.fn();
const checkPortMock = vi.fn();

let pingPacketCb: ((payload: any) => void) | null = null;
let pingSummaryCb: ((payload: any) => void) | null = null;
let traceHopCb: ((payload: any) => void) | null = null;
let traceDoneCb: ((payload: any) => void) | null = null;

vi.mock("@/lib/tauri", () => ({
    startToolPing: (params: unknown) => startToolPingMock(params),
    startToolTraceroute: (params: unknown) => startToolTracerouteMock(params),
    cancelTool: (sessionId: string) => cancelToolMock(sessionId),
    dnsLookup: (name: string, recordType: string, provider?: string) =>
        dnsLookupMock(name, recordType, provider),
    sendWol: (mac: string, broadcastIp?: string) => sendWolMock(mac, broadcastIp),
    checkPort: (target: string, port: number, timeoutMs?: number) =>
        checkPortMock(target, port, timeoutMs),
    onToolPingPacket: (cb: any) => {
        pingPacketCb = cb;
        return Promise.resolve(() => {
            pingPacketCb = null;
        });
    },
    onToolPingSummary: (cb: any) => {
        pingSummaryCb = cb;
        return Promise.resolve(() => {
            pingSummaryCb = null;
        });
    },
    onToolTraceHop: (cb: any) => {
        traceHopCb = cb;
        return Promise.resolve(() => {
            traceHopCb = null;
        });
    },
    onToolTraceDone: (cb: any) => {
        traceDoneCb = cb;
        return Promise.resolve(() => {
            traceDoneCb = null;
        });
    },
}));

vi.mock("@/components/censorship-provider", () => ({
    useCensorship: () => ({
        censorshipEnabled: false,
        setCensorshipEnabled: vi.fn(),
        toggle: vi.fn(),
    }),
}));

const toastMock = vi.fn();
vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: toastMock }),
}));

describe("Tools Screen", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        cancelToolMock.mockResolvedValue(true);
        startToolPingMock.mockResolvedValue(undefined);
        startToolTracerouteMock.mockResolvedValue(undefined);
        pingPacketCb = null;
        pingSummaryCb = null;
        traceHopCb = null;
        traceDoneCb = null;
    });

    it("renders default Ping Interactivo tab and tool selector buttons", () => {
        renderWithProviders(<Tools />);

        expect(screen.getByText("Herramientas de red")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /Ping Interactivo/i })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /Traceroute Visual/i })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /DNS Lookup/i })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /Port Checker/i })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /Wake-on-LAN/i })).toBeInTheDocument();

        expect(screen.getByRole("button", { name: /Iniciar Ping/i })).toBeInTheDocument();
    });

    it("pre-fills tool and parameters from URL search params", () => {
        renderWithProviders(<Tools />, ["/tools?tool=traceroute&ip=192.168.1.100"]);

        const targetInput = screen.getByDisplayValue("192.168.1.100");
        expect(targetInput).toBeInTheDocument();
        expect(screen.getByRole("button", { name: /Trazar ruta/i })).toBeInTheDocument();
    });

    it("executes Ping and processes incoming packets via event listener", async () => {
        let capturedSessionId = "";
        startToolPingMock.mockImplementationOnce((params: { sessionId: string }) => {
            capturedSessionId = params.sessionId;
            return Promise.resolve();
        });

        renderWithProviders(<Tools />);

        const targetInput = screen.getByPlaceholderText(/192\.168\.1\.1/);
        fireEvent.change(targetInput, { target: { value: "1.1.1.1" } });

        const startButton = screen.getByRole("button", { name: /Iniciar Ping/i });
        fireEvent.click(startButton);

        await waitFor(() => {
            expect(startToolPingMock).toHaveBeenCalledWith(
                expect.objectContaining({ target: "1.1.1.1" }),
            );
        });

        expect(pingPacketCb).toBeDefined();

        act(() => {
            pingPacketCb!({
                session_id: capturedSessionId,
                packet: {
                    seq: 1,
                    target: "1.1.1.1",
                    reachable: true,
                    latency_ms: 14.5,
                    jitter_ms: 1.2,
                    method: "icmp",
                },
            });
        });

        await waitFor(() => {
            expect(screen.getByText(/Transmitidos/i)).toBeInTheDocument();
            expect(screen.getByText("14.5 ms")).toBeInTheDocument();
        });
    });

    it("can stop active ping session via cancel button", async () => {
        let capturedSessionId = "";
        startToolPingMock.mockImplementationOnce((params: { sessionId: string }) => {
            capturedSessionId = params.sessionId;
            return Promise.resolve();
        });

        renderWithProviders(<Tools />);

        const targetInput = screen.getByPlaceholderText(/192\.168\.1\.1/);
        fireEvent.change(targetInput, { target: { value: "8.8.8.8" } });

        fireEvent.click(screen.getByRole("button", { name: /Iniciar Ping/i }));

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Detener/i })).toBeInTheDocument();
        });

        fireEvent.click(screen.getByRole("button", { name: /Detener/i }));

        await waitFor(() => {
            expect(cancelToolMock).toHaveBeenCalledWith(capturedSessionId);
        });
    });

    it("respects ping packet limit configuration and presets", async () => {
        renderWithProviders(<Tools />);

        const targetInput = screen.getByPlaceholderText(/192\.168\.1\.1/);
        fireEvent.change(targetInput, { target: { value: "1.1.1.1" } });

        // Default limit is 10
        const countInput = screen.getByTitle(/Cantidad de paquetes a enviar/i);
        expect(countInput).toHaveValue(10);

        // Click preset button 5
        const preset5Btn = screen.getByRole("button", { name: "5" });
        fireEvent.click(preset5Btn);
        expect(countInput).toHaveValue(5);

        // Click preset continuous
        const continuousBtn = screen.getByRole("button", { name: /∞ Continuo/i });
        fireEvent.click(continuousBtn);
        expect(countInput).toHaveValue(null);

        // Type custom limit
        fireEvent.change(countInput, { target: { value: "25" } });
        expect(countInput).toHaveValue(25);

        fireEvent.click(screen.getByRole("button", { name: /Iniciar Ping/i }));

        await waitFor(() => {
            expect(startToolPingMock).toHaveBeenCalledWith(
                expect.objectContaining({
                    target: "1.1.1.1",
                    count: 25,
                }),
            );
        });
    });

    it("cleans up packets and restores readiness HUD when clicking Limpiar", async () => {
        let capturedSessionId = "";
        startToolPingMock.mockImplementationOnce((params: { sessionId: string }) => {
            capturedSessionId = params.sessionId;
            return Promise.resolve();
        });

        renderWithProviders(<Tools />);

        const targetInput = screen.getByPlaceholderText(/192\.168\.1\.1/);
        fireEvent.change(targetInput, { target: { value: "1.1.1.1" } });
        fireEvent.click(screen.getByRole("button", { name: /Iniciar Ping/i }));

        await waitFor(() => {
            expect(startToolPingMock).toHaveBeenCalled();
        });

        act(() => {
            pingPacketCb!({
                session_id: capturedSessionId,
                packet: {
                    seq: 1,
                    target: "1.1.1.1",
                    reachable: true,
                    latency_ms: 12,
                    jitter_ms: 0.5,
                    method: "icmp",
                },
            });
            pingSummaryCb!({
                session_id: capturedSessionId,
                summary: {
                    target: "1.1.1.1",
                    transmitted: 1,
                    received: 1,
                    packet_loss_percent: 0,
                    min_rtt_ms: 12,
                    avg_rtt_ms: 12,
                    max_rtt_ms: 12,
                    jitter_ms: 0.5,
                },
            });
        });

        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Limpiar/i })).toBeInTheDocument();
            expect(screen.getByText("Stream de paquetes (1/10)")).toBeInTheDocument();
        });

        // Click Limpiar
        fireEvent.click(screen.getByRole("button", { name: /Limpiar/i }));

        await waitFor(() => {
            expect(screen.getByText("Sondeo de latencia en espera")).toBeInTheDocument();
            expect(screen.queryByText("Stream de paquetes")).not.toBeInTheDocument();
            expect(screen.queryByText(/Transmitidos/i)).not.toBeInTheDocument();
        });
    });

    it("reserves layout space in tool tabs so heartbeat does not shift tabs", () => {
        renderWithProviders(<Tools />);

        // Every tool tab button should have the reserved space (aria-hidden span with h-2 w-2)
        const tabButtons = screen.getAllByRole("button", { name: /(Ping Interactivo|Traceroute Visual|DNS Lookup|Port Checker|Wake-on-LAN|Velocidad y Salud)/i });
        expect(tabButtons.length).toBe(6);

        tabButtons.forEach((btn) => {
            const heartbeatSlot = btn.querySelector("span[aria-hidden='true']");
            expect(heartbeatSlot).toBeInTheDocument();
            expect(heartbeatSlot?.className).toContain("h-2");
            expect(heartbeatSlot?.className).toContain("w-2");
            expect(heartbeatSlot?.className).toContain("shrink-0");
        });
    });

    it("executes DNS lookup and displays resolved records", async () => {
        dnsLookupMock.mockResolvedValueOnce([
            {
                name: "example.com",
                record_type: "A",
                value: "93.184.216.34",
                ttl: 300,
            },
        ]);

        renderWithProviders(<Tools />, ["/tools?tool=dns&ip=example.com"]);

        const queryButton = screen.getByRole("button", { name: /Consultar DNS/i });
        fireEvent.click(queryButton);

        await waitFor(() => {
            expect(dnsLookupMock).toHaveBeenCalledWith("example.com", "A", undefined);
            expect(screen.getByText("93.184.216.34")).toBeInTheDocument();
        });
    });

    it("allows selecting DNS record type and provider via shadcn select", async () => {
        const user = userEvent.setup();
        dnsLookupMock.mockResolvedValueOnce([
            {
                name: "example.com",
                record_type: "AAAA",
                value: "2606:2800:220:1:248:1893:25c8:1946",
                ttl: 300,
            },
        ]);

        renderWithProviders(<Tools />, ["/tools?tool=dns&ip=example.com"]);

        const typeTrigger = screen.getByRole("combobox", { name: "Tipo de registro" });
        await user.click(typeTrigger);
        const aaaaOption = await screen.findByRole("option", { name: "AAAA" });
        await user.click(aaaaOption);

        const providerTrigger = screen.getByRole("combobox", { name: "Servidor DNS" });
        await user.click(providerTrigger);
        const cloudflareOption = await screen.findByRole("option", { name: /Cloudflare/i });
        await user.click(cloudflareOption);

        const queryButton = screen.getByRole("button", { name: /Consultar DNS/i });
        await user.click(queryButton);

        await waitFor(() => {
            expect(dnsLookupMock).toHaveBeenCalledWith("example.com", "AAAA", "cloudflare");
            expect(screen.getByText("2606:2800:220:1:248:1893:25c8:1946")).toBeInTheDocument();
        });
    });

    it("checks TCP port and displays latency and state", async () => {
        checkPortMock.mockResolvedValueOnce({
            target: "192.168.1.1",
            port: 80,
            state: "open",
            latency_ms: 2.4,
            banner: null,
        });

        renderWithProviders(<Tools />, ["/tools?tool=ports&ip=192.168.1.1"]);

        const testButton = screen.getByRole("button", { name: /Comprobar puerto/i });
        fireEvent.click(testButton);

        await waitFor(() => {
            expect(checkPortMock).toHaveBeenCalledWith("192.168.1.1", 80, 2000);
            expect(screen.getByText("Abierto")).toBeInTheDocument();
            expect(screen.getByText(/2\.4 ms RTT/i)).toBeInTheDocument();
        });
    });

    it("sends Wake-on-LAN magic packet and triggers toast", async () => {
        sendWolMock.mockResolvedValueOnce(undefined);

        renderWithProviders(<Tools />, ["/tools?tool=wol&mac=AA:BB:CC:DD:EE:FF"]);

        const wolButton = screen.getByRole("button", { name: /Enviar Magic Packet/i });
        fireEvent.click(wolButton);

        await waitFor(() => {
            expect(sendWolMock).toHaveBeenCalledWith("AA:BB:CC:DD:EE:FF", undefined);
            expect(toastMock).toHaveBeenCalledWith(
                "Magic Packet de Wake-on-LAN enviado.",
                "success",
            );
        });
    });
});
