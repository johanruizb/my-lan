import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Routes, Route } from "react-router-dom";
import { SecurityScoreCard } from "@/components/security-score-card";
import { DeviceDetail } from "@/screens/DeviceDetail";
import { renderWithProviders } from "./helpers";
import { CensorshipProvider } from "@/components/censorship-provider";
import type { SecurityFinding } from "@/lib/tauri";

const mockGetSecurityScore = vi.fn();
const mockGetSecurityFindings = vi.fn();
const mockRunSecurityAudit = vi.fn();
const mockUpdateFindingStatus = vi.fn();
const mockGetDevice = vi.fn();

vi.mock("@/lib/tauri", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/lib/tauri")>();
    return {
        ...actual,
        getSecurityScore: () => mockGetSecurityScore(),
        getSecurityFindings: (params?: { networkId?: string; deviceId?: string }) =>
            mockGetSecurityFindings(params),
        runSecurityAudit: (networkId?: string) => mockRunSecurityAudit(networkId),
        updateFindingStatus: (id: string, status: string) =>
            mockUpdateFindingStatus(id, status),
        getDevice: (ip: string) => mockGetDevice(ip),
        onScanFinished: vi.fn().mockResolvedValue(() => {}),
        onScanProgress: vi.fn().mockResolvedValue(() => {}),
        onScanHeartbeat: vi.fn().mockResolvedValue(() => {}),
        onScanCancelled: vi.fn().mockResolvedValue(() => {}),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

describe("Security Audit UI (Fase 8)", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("SecurityScoreCard renderiza el score 0-100 y el desglose de severidades", async () => {
        mockGetSecurityScore.mockResolvedValue(60);
        const findings: SecurityFinding[] = [
            {
                id: "f-1",
                device_id: "dev-1",
                network_id: "net-1",
                rule_id: "SEC-001-TELNET-OPEN",
                severity: "critical",
                title: "Telnet abierto",
                description: "Texto claro",
                evidence: "Puerto 23",
                remediation_advice: "Cerrar telnet",
                status: "open",
                first_seen_at: "t0",
                last_seen_at: "t0",
                resolved_at: null,
            },
            {
                id: "f-2",
                device_id: "dev-1",
                network_id: "net-1",
                rule_id: "SEC-002-FTP-PLAINTEXT",
                severity: "high",
                title: "FTP abierto",
                description: "Texto claro",
                evidence: "Puerto 21",
                remediation_advice: null,
                status: "open",
                first_seen_at: "t0",
                last_seen_at: "t0",
                resolved_at: null,
            },
        ];
        mockGetSecurityFindings.mockResolvedValue(findings);

        renderWithProviders(<SecurityScoreCard />);

        await waitFor(() => {
            expect(screen.getByText("60")).toBeInTheDocument();
            expect(screen.getByText(/Riesgo moderado en la red/i)).toBeInTheDocument();
            expect(screen.getByText(/1 Crítico/i)).toBeInTheDocument();
            expect(screen.getByText(/1 Alto/i)).toBeInTheDocument();
        });
    });

    it("SecurityScoreCard botón Re-auditar ejecuta runSecurityAudit y actualiza score", async () => {
        mockGetSecurityScore.mockResolvedValue(100);
        mockGetSecurityFindings.mockResolvedValue([]);
        mockRunSecurityAudit.mockResolvedValue({
            network_id: "net-1",
            score: 75,
            total_findings: 1,
            critical_count: 1,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            findings: [
                {
                    id: "f-crit",
                    device_id: "dev-1",
                    network_id: "net-1",
                    rule_id: "SEC-001-TELNET-OPEN",
                    severity: "critical",
                    title: "Telnet detectado",
                    description: "Inseguro",
                    evidence: "23/tcp",
                    remediation_advice: null,
                    status: "open",
                    first_seen_at: "t0",
                    last_seen_at: "t0",
                    resolved_at: null,
                },
            ],
        });

        renderWithProviders(<SecurityScoreCard />);

        await waitFor(() => {
            expect(screen.getByText("100")).toBeInTheDocument();
        });

        const reauditBtn = screen.getByRole("button", { name: /Re-auditar/i });
        fireEvent.click(reauditBtn);

        await waitFor(() => {
            expect(mockRunSecurityAudit).toHaveBeenCalled();
            expect(screen.getByText("75")).toBeInTheDocument();
        });
    });

    it("DeviceDetail muestra pestaña 'Seguridad y Riesgos' y permite aceptar o ignorar riesgo", async () => {
        mockGetDevice.mockResolvedValue({
            device: {
                id: "dev-test",
                network_id: "net-1",
                primary_mac: "aa:bb:cc:dd:ee:ff",
                primary_ip: "192.168.1.50",
                hostname: "vulnerable-host",
                display_name: null,
                vendor: null,
                manufacturer: null,
                model: null,
                device_type: "device",
                os_family: null,
                confidence: 90,
                first_seen_at: "t0",
                last_seen_at: "t0",
                is_trusted: false,
                is_hidden: false,
                is_online: true,
                notes: null,
            },
            services: [],
        });

        mockGetSecurityFindings.mockResolvedValue([
            {
                id: "find-smb",
                device_id: "dev-test",
                network_id: "net-1",
                rule_id: "SEC-004-SMB-LEGACY",
                severity: "high",
                title: "SMB expuesto en host no verificado",
                description: "Vector de malware",
                evidence: "Puerto TCP 445 en 192.168.1.50",
                remediation_advice: "Desactivar SMB en ajustes",
                status: "open",
                first_seen_at: "t0",
                last_seen_at: "t0",
                resolved_at: null,
            },
        ]);

        mockUpdateFindingStatus.mockResolvedValue(true);

        renderWithProviders(
            <CensorshipProvider initialEnabled={true}>
                <Routes>
                    <Route path="/devices/:ip" element={<DeviceDetail />} />
                </Routes>
            </CensorshipProvider>,
            ["/devices/192.168.1.50"],
        );

        // Espera a que cargue el dispositivo
        await waitFor(() => {
            expect(screen.getByRole("button", { name: /Seguridad y Riesgos/i })).toBeInTheDocument();
        });

        // Cambia a pestaña Seguridad y Riesgos
        const secTab = screen.getByRole("button", { name: /Seguridad y Riesgos/i });
        fireEvent.click(secTab);

        await waitFor(() => {
            expect(screen.getByText("SMB expuesto en host no verificado")).toBeInTheDocument();
            expect(screen.getByText(/SEC-004-SMB-LEGACY/)).toBeInTheDocument();
            expect(screen.getByText(/Alto \(-15 pts\)/)).toBeInTheDocument();
        });

        // Pulsa "Aceptar riesgo"
        const ackBtn = screen.getByRole("button", { name: /Aceptar riesgo/i });
        fireEvent.click(ackBtn);

        await waitFor(() => {
            expect(mockUpdateFindingStatus).toHaveBeenCalledWith("find-smb", "acknowledged");
        });
    });
});
