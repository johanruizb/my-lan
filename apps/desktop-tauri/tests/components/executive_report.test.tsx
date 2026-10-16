import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
    ExecutiveReportDocument,
    type ExecutiveReportProps,
} from "@/components/executive-report-pdf";
import { ExportReportModal } from "@/components/export-report-modal";
import { WorkspaceProvider, DEFAULT_WORKSPACE } from "@/components/workspace-provider";
import { CensorshipProvider } from "@/components/censorship-provider";
import type { Device, SecurityFinding } from "@/lib/tauri";

const mockListDevices = vi.fn();
const mockGetSecurityFindings = vi.fn();
const mockGetSecurityScore = vi.fn();
const mockGetRecentSpeedTests = vi.fn();
const mockGetNetworkHealthSamples = vi.fn();
const mockGetAppVersion = vi.fn();

vi.mock("@/lib/tauri", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/lib/tauri")>();
    return {
        ...actual,
        listDevices: (ws?: string) => mockListDevices(ws),
        getSecurityFindings: (net?: string) => mockGetSecurityFindings(net),
        getSecurityScore: (net?: string) => mockGetSecurityScore(net),
        getRecentSpeedTests: (params?: any) => mockGetRecentSpeedTests(params),
        getNetworkHealthSamples: (net?: string, lim?: number) =>
            mockGetNetworkHealthSamples(net, lim),
        getAppVersion: () => mockGetAppVersion(),
        listWorkspaces: vi.fn().mockResolvedValue([
            {
                id: "default",
                name: "Mi Red Local",
                description: "Espacio de trabajo principal",
                icon: "home",
                color: "#06b6d4",
                created_at: "2026-10-16T00:00:00Z",
                updated_at: "2026-10-16T00:00:00Z",
            },
        ]),
        getSettings: vi.fn().mockResolvedValue({
            active_workspace_id: "default",
            censorship_enabled: true,
        }),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

// Mock react-pdf downloadExecutiveReport
vi.mock("@/components/executive-report-pdf", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/components/executive-report-pdf")>();
    return {
        ...actual,
        downloadExecutiveReport: vi.fn().mockResolvedValue(undefined),
    };
});

describe("Executive PDF Report & Export Modal (Fase 10)", () => {
    const sampleDevice: Device = {
        id: "dev-test-1",
        network_id: "net-1",
        primary_ip: "192.168.1.50",
        primary_mac: "a4:83:e7:12:34:56",
        hostname: "server.lan",
        display_name: "Servidor Principal",
        vendor: "Apple, Inc.",
        manufacturer: null,
        model: null,
        device_type: "nas" as any,
        os_family: "Linux",
        confidence: 90 as any,
        first_seen_at: "2026-10-16T10:00:00Z",
        last_seen_at: "2026-10-16T12:00:00Z",
        is_trusted: true,
        is_hidden: false,
        is_online: true,
        notes: null,
    };

    const sampleFinding: SecurityFinding = {
        id: "find-1",
        device_id: "dev-test-1",
        network_id: "net-1",
        rule_id: "SEC-001-TELNET-OPEN",
        severity: "Critical",
        title: "Puerto Telnet 23 Inseguro Abierto",
        description: "Servicio de administración sin cifrado.",
        evidence: "192.168.1.50:23",
        remediation_advice: "Desactivar Telnet y usar SSH.",
        status: "open",
        first_seen_at: "2026-10-16T10:00:00Z",
        last_seen_at: "2026-10-16T12:00:00Z",
        resolved_at: null,
    };

    beforeEach(() => {
        vi.clearAllMocks();
        mockListDevices.mockResolvedValue([sampleDevice]);
        mockGetSecurityFindings.mockResolvedValue([sampleFinding]);
        mockGetSecurityScore.mockResolvedValue(75);
        mockGetRecentSpeedTests.mockResolvedValue([]);
        mockGetNetworkHealthSamples.mockResolvedValue([]);
        mockGetAppVersion.mockResolvedValue("1.0.0");
    });

    it("ExecutiveReportDocument renderiza la estructura con datos enmascarados si el Modo Censura está activo", () => {
        const props: ExecutiveReportProps = {
            workspace: DEFAULT_WORKSPACE,
            networkName: "Casa",
            networkCidr: "192.168.1.0/24",
            generatedAt: "2026-10-16 12:00",
            censorshipEnabled: true,
            devices: [sampleDevice],
            securityScore: 75,
            findings: [sampleFinding],
            version: "1.0.0",
        };

        const doc = ExecutiveReportDocument(props);
        expect(doc).toBeDefined();
        // Verificar que los campos sensibles se pasaron enmascarados a los componentes
        const json = JSON.stringify(doc);
        expect(json).toContain("MODO CENSURA ACTIVO");
        expect(json).toContain("192.168.*.*");
        expect(json).toContain("••••");
        expect(json).not.toContain("a4:83:e7:12:34:56");
    });

    it("ExecutiveReportDocument renderiza IPs y MACs reales cuando el Modo Censura está desactivado", () => {
        const props: ExecutiveReportProps = {
            workspace: DEFAULT_WORKSPACE,
            networkName: "Casa",
            networkCidr: "192.168.1.0/24",
            generatedAt: "2026-10-16 12:00",
            censorshipEnabled: false,
            devices: [sampleDevice],
            securityScore: 100,
            findings: [],
            version: "1.0.0",
        };

        const doc = ExecutiveReportDocument(props);
        expect(doc).toBeDefined();
        const json = JSON.stringify(doc);
        expect(json).not.toContain("MODO CENSURA ACTIVO");
        expect(json).toContain("192.168.1.50");
        expect(json).toContain("a4:83:e7:12:34:56");
    });

    it("ExportReportModal renderiza el diálogo y permite iniciar la descarga", async () => {
        const onOpenChange = vi.fn();
        const { downloadExecutiveReport } = await import(
            "@/components/executive-report-pdf"
        );

        render(
            <CensorshipProvider initialEnabled={true}>
                <WorkspaceProvider>
                    <ExportReportModal open={true} onOpenChange={onOpenChange} />
                </WorkspaceProvider>
            </CensorshipProvider>,
        );

        await waitFor(() => {
            expect(
                screen.getByText("Exportar Informe Ejecutivo en PDF"),
            ).toBeInTheDocument();
            expect(screen.getByText("Mi Red Local")).toBeInTheDocument();
            expect(
                screen.getByText("Exportar con Modo Censura"),
            ).toBeInTheDocument();
        });

        const downloadBtn = screen.getByRole("button", {
            name: /Descargar Informe PDF/i,
        });
        fireEvent.click(downloadBtn);

        await waitFor(() => {
            expect(downloadExecutiveReport).toHaveBeenCalled();
            expect(onOpenChange).toHaveBeenCalledWith(false);
        });
    });
});
