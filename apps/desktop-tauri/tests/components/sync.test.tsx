import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SyncStatusIndicator } from "@/components/sync-status-indicator";
import { GdriveSyncDialog } from "@/components/gdrive-sync-dialog";
import { WorkspaceProvider } from "@/components/workspace-provider";
import type { GdriveSyncStatus, SyncOutcome } from "@/lib/tauri";

const {
    mockGetGdriveSyncStatus,
    mockStartGdriveAuth,
    mockDisconnectGdrive,
    mockSetupMasterKey,
    mockRestoreMasterKey,
    mockSyncWorkspace,
    mockResolveSyncConflict,
} = vi.hoisted(() => ({
    mockGetGdriveSyncStatus: vi.fn(),
    mockStartGdriveAuth: vi.fn(),
    mockDisconnectGdrive: vi.fn(),
    mockSetupMasterKey: vi.fn(),
    mockRestoreMasterKey: vi.fn(),
    mockSyncWorkspace: vi.fn(),
    mockResolveSyncConflict: vi.fn(),
}));

vi.mock("@/lib/tauri", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/lib/tauri")>();
    return {
        ...actual,
        getGdriveSyncStatus: () => mockGetGdriveSyncStatus(),
        startGdriveAuth: () => mockStartGdriveAuth(),
        disconnectGdrive: () => mockDisconnectGdrive(),
        setupMasterKey: (pass: string) => mockSetupMasterKey(pass),
        restoreMasterKey: (words: string) => mockRestoreMasterKey(words),
        syncWorkspace: (wsId: string, override?: boolean) => mockSyncWorkspace(wsId, override),
        resolveSyncConflict: (wsId: string, res: string) => mockResolveSyncConflict(wsId, res),
        listWorkspaces: vi.fn().mockResolvedValue([
            {
                id: "default",
                name: "Predeterminado",
                description: "Espacio de trabajo local principal",
                icon: "network",
                color: "#0891b2",
                created_at: "2026-10-16T00:00:00Z",
                updated_at: "2026-10-16T00:00:00Z",
            },
        ]),
        getSettings: vi.fn().mockResolvedValue({ active_workspace_id: "default" }),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

describe("Zero-Knowledge Google Drive Sync (Fase 11)", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mockGetGdriveSyncStatus.mockResolvedValue({
            is_authenticated: false,
            email: null,
            has_master_key: false,
            last_synced_at: null,
            last_synced_epoch: null,
        });
    });

    it("renderiza el indicador en estado 'Nube' (desconectado)", async () => {
        render(
            <WorkspaceProvider>
                <SyncStatusIndicator showLabel />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Nube")).toBeInTheDocument();
        });
    });

    it("renderiza el indicador en estado 'Sincronizado' cuando está autenticado y tiene clave maestra", async () => {
        mockGetGdriveSyncStatus.mockResolvedValue({
            is_authenticated: true,
            email: "bob@example.com",
            has_master_key: true,
            last_synced_at: "2026-10-16T14:00:00Z",
            last_synced_epoch: 1792159200,
        });

        render(
            <WorkspaceProvider>
                <SyncStatusIndicator showLabel />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Sincronizado")).toBeInTheDocument();
        });
    });

    it("renderiza el indicador en estado 'Configurar Clave' cuando está autenticado pero falta clave", async () => {
        mockGetGdriveSyncStatus.mockResolvedValue({
            is_authenticated: true,
            email: "carol@example.com",
            has_master_key: false,
            last_synced_at: null,
            last_synced_epoch: null,
        });

        render(
            <WorkspaceProvider>
                <SyncStatusIndicator showLabel />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Configurar Clave")).toBeInTheDocument();
        });
    });

    it("abre el diálogo de sincronización al hacer clic en el botón", async () => {
        render(
            <WorkspaceProvider>
                <SyncStatusIndicator showLabel />
            </WorkspaceProvider>,
        );

        const btn = await screen.findByRole("button", { name: /Nube/i });
        fireEvent.click(btn);

        await waitFor(() => {
            expect(
                screen.getByText("Sincronización en la Nube"),
            ).toBeInTheDocument();
        });
        expect(
            screen.getAllByText(/Zero-Knowledge/i)[0],
        ).toBeInTheDocument();
    });

    it("permite configurar una nueva clave maestra y muestra el mnemónico BIP-39", async () => {
        const status: GdriveSyncStatus = {
            is_authenticated: true,
            email: "alice@example.com",
            has_master_key: false,
            last_synced_at: null,
            last_synced_epoch: null,
        };

        mockSetupMasterKey.mockResolvedValue({
            mnemonic_phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            key_hash: "abcd1234ef",
        });

        const onStatusChange = vi.fn();

        render(
            <WorkspaceProvider>
                <GdriveSyncDialog
                    open={true}
                    onOpenChange={vi.fn()}
                    status={status}
                    onStatusChange={onStatusChange}
                    onSyncStart={vi.fn()}
                    onSyncEnd={vi.fn()}
                />
            </WorkspaceProvider>,
        );

        // Encuentra el input de contraseña
        const passInput = screen.getByPlaceholderText(/mín\. 8 carácteres/i);
        fireEvent.change(passInput, { target: { value: "super-secure-passphrase-2026" } });

        // Envía el formulario para generar la clave
        const submitBtn = screen.getByRole("button", { name: /Crear Clave/i });
        fireEvent.click(submitBtn);

        await waitFor(() => {
            expect(mockSetupMasterKey).toHaveBeenCalledWith("super-secure-passphrase-2026");
        });

        // Debe renderizar la tarjeta con las 12 palabras de recuperación
        await waitFor(() => {
            expect(
                screen.getByText(/Frase de Recuperación \(12 palabras BIP-39\)/i),
            ).toBeInTheDocument();
        });
        expect(screen.getAllByText(/abandon/i).length).toBeGreaterThan(0);
        expect(screen.getByRole("button", { name: /Copiar/i })).toBeInTheDocument();
    });

    it("resuelve conflictos de sincronización permitiendo adoptar la versión de la nube", async () => {
        const conflictStatus: GdriveSyncStatus = {
            is_authenticated: true,
            email: "alice@example.com",
            has_master_key: true,
            last_synced_at: "2026-10-16T10:00:00Z",
            last_synced_epoch: 1792144800,
        };

        const conflictOutcome: SyncOutcome = {
            status: "conflict",
            remote_updated_at: "2026-10-16T14:30:00Z",
            remote_devices_count: 15,
            remote_md5: "abc123md5",
            backup_path: ".mylan/backups/conflict_default_1792150000.json",
        };

        mockSyncWorkspace.mockResolvedValue(conflictOutcome);
        mockResolveSyncConflict.mockResolvedValue({
            status: "success",
            action: "restored_cloud_version",
            devices_synced: 15,
            timestamp: "2026-10-16T14:35:00Z",
        });

        render(
            <WorkspaceProvider>
                <GdriveSyncDialog
                    open={true}
                    onOpenChange={vi.fn()}
                    status={conflictStatus}
                    onStatusChange={vi.fn()}
                    onSyncStart={vi.fn()}
                    onSyncEnd={vi.fn()}
                />
            </WorkspaceProvider>,
        );

        // Clic en "Sincronizar Ahora"
        const syncBtn = screen.getByRole("button", { name: /Sincronizar Ahora/i });
        fireEvent.click(syncBtn);

        // Debe detectar el conflicto y mostrar el banner
        await waitFor(() => {
            expect(
                screen.getByText(/Conflicto de versiones detectado/i),
            ).toBeInTheDocument();
        });

        // Elige "Adoptar versión de la nube"
        const adoptBtn = screen.getByRole("button", { name: /Adoptar versión de la nube/i });
        fireEvent.click(adoptBtn);

        await waitFor(() => {
            expect(mockResolveSyncConflict).toHaveBeenCalledWith("default", "adopt_cloud");
        });
    });

    it("requiere confirmación explícita de dos pasos antes de desconectar Google Drive", async () => {
        const authenticatedStatus: GdriveSyncStatus = {
            is_authenticated: true,
            email: "alice@example.com",
            has_master_key: true,
            last_synced_at: "2026-10-16T12:00:00Z",
            last_synced_epoch: 1792150000,
        };
        const onStatusChangeMock = vi.fn();
        mockDisconnectGdrive.mockResolvedValue(undefined);

        render(
            <WorkspaceProvider>
                <GdriveSyncDialog
                    open={true}
                    onOpenChange={vi.fn()}
                    status={authenticatedStatus}
                    onStatusChange={onStatusChangeMock}
                    onSyncStart={vi.fn()}
                    onSyncEnd={vi.fn()}
                />
            </WorkspaceProvider>,
        );

        // Primer clic en "Desconectar"
        const disconnectBtn = screen.getByRole("button", { name: /Desconectar/i });
        fireEvent.click(disconnectBtn);

        // Debe solicitar confirmación sin ejecutar la acción destructiva de inmediato
        expect(screen.getByText("¿Desconectar?")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Confirmar" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Cancelar" })).toBeInTheDocument();
        expect(mockDisconnectGdrive).not.toHaveBeenCalled();

        // Si cancela, vuelve al estado inicial sin desconectar
        const cancelBtn = screen.getByRole("button", { name: "Cancelar" });
        fireEvent.click(cancelBtn);
        expect(screen.queryByText("¿Desconectar?")).not.toBeInTheDocument();
        expect(mockDisconnectGdrive).not.toHaveBeenCalled();

        // Al pulsar de nuevo y confirmar, ahora sí se ejecuta la desconexión
        const disconnectAgainBtn = screen.getByRole("button", { name: /Desconectar/i });
        fireEvent.click(disconnectAgainBtn);
        const confirmBtn = screen.getByRole("button", { name: "Confirmar" });
        fireEvent.click(confirmBtn);

        await waitFor(() => {
            expect(mockDisconnectGdrive).toHaveBeenCalled();
            expect(onStatusChangeMock).toHaveBeenCalledWith(
                expect.objectContaining({
                    is_authenticated: false,
                    email: null,
                }),
            );
        });
    });
});
