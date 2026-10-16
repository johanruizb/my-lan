import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WorkspaceSelector } from "@/components/workspace-selector";
import {
    WorkspaceManageDialog,
    PRESET_COLORS,
} from "@/components/workspace-manage-dialog";
import {
    WorkspaceProvider,
    DEFAULT_WORKSPACE,
} from "@/components/workspace-provider";
import type { Workspace } from "@/lib/tauri";

const mockListWorkspaces = vi.fn();
const mockCreateWorkspace = vi.fn();
const mockUpdateWorkspace = vi.fn();
const mockDeleteWorkspace = vi.fn();
const mockGetSettings = vi.fn();
const mockSetSettings = vi.fn();

vi.mock("@/lib/tauri", async (importOriginal) => {
    const actual = await importOriginal<typeof import("@/lib/tauri")>();
    return {
        ...actual,
        listWorkspaces: () => mockListWorkspaces(),
        createWorkspace: (payload: any) => mockCreateWorkspace(payload),
        updateWorkspace: (payload: any) => mockUpdateWorkspace(payload),
        deleteWorkspace: (id: string) => mockDeleteWorkspace(id),
        getSettings: () => mockGetSettings(),
        setSettings: (s: any) => mockSetSettings(s),
    };
});

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: vi.fn() }),
}));

describe("Workspace Management (Fase 10)", () => {
    const sampleWorkspaces: Workspace[] = [
        DEFAULT_WORKSPACE,
        {
            id: "ws-office",
            name: "Oficina Central",
            description: "Red corporativa y servidores",
            icon: "building",
            color: "#3b82f6",
            created_at: "2026-10-16T10:00:00Z",
            updated_at: "2026-10-16T10:00:00Z",
        },
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        mockListWorkspaces.mockResolvedValue(sampleWorkspaces);
        mockGetSettings.mockResolvedValue({
            active_workspace_id: "default",
            censorship_enabled: true,
        });
        mockSetSettings.mockResolvedValue(undefined);
    });

    it("renderiza el WorkspaceSelector con el workspace activo por defecto", async () => {
        render(
            <WorkspaceProvider>
                <WorkspaceSelector />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Mi Red Local")).toBeInTheDocument();
            expect(screen.getByText("Workspace")).toBeInTheDocument();
        });
    });

    it("despliega el menú al hacer clic y permite conmutar de workspace", async () => {
        render(
            <WorkspaceProvider>
                <WorkspaceSelector />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Mi Red Local")).toBeInTheDocument();
        });

        const trigger = screen.getByRole("button");
        fireEvent.pointerDown(trigger, { button: 0, ctrlKey: false });
        fireEvent.keyDown(trigger, { key: "Enter", code: "Enter" });

        await waitFor(() => {
            expect(screen.getByText("Espacios de Trabajo")).toBeInTheDocument();
            expect(screen.getByText("Oficina Central")).toBeInTheDocument();
        });

        const officeItem = screen.getByText("Oficina Central");
        fireEvent.click(officeItem);

        await waitFor(() => {
            expect(mockSetSettings).toHaveBeenCalledWith(
                expect.objectContaining({ active_workspace_id: "ws-office" }),
            );
        });
    });

    it("WorkspaceManageDialog permite crear un nuevo workspace", async () => {
        const onOpenChange = vi.fn();
        mockCreateWorkspace.mockResolvedValue({
            id: "ws-new",
            name: "Laboratorio IoT",
            description: "Pruebas de domótica",
            icon: "network",
            color: PRESET_COLORS[0],
            created_at: "2026-10-16T12:00:00Z",
            updated_at: "2026-10-16T12:00:00Z",
        });

        render(
            <WorkspaceProvider>
                <WorkspaceManageDialog
                    open={true}
                    onOpenChange={onOpenChange}
                    initialMode="create"
                />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Crear Nuevo Workspace")).toBeInTheDocument();
        });

        const nameInput = screen.getByPlaceholderText(/Oficina Principal/i);
        fireEvent.change(nameInput, { target: { value: "Laboratorio IoT" } });

        const saveBtn = screen.getByRole("button", {
            name: /Guardar Workspace/i,
        });
        fireEvent.click(saveBtn);

        await waitFor(() => {
            expect(mockCreateWorkspace).toHaveBeenCalledWith(
                expect.objectContaining({ name: "Laboratorio IoT" }),
            );
            expect(onOpenChange).toHaveBeenCalledWith(false);
        });
    });

    it("WorkspaceManageDialog en modo gestión muestra la lista y no permite eliminar el default", async () => {
        const onOpenChange = vi.fn();
        render(
            <WorkspaceProvider>
                <WorkspaceManageDialog
                    open={true}
                    onOpenChange={onOpenChange}
                    initialMode="manage"
                />
            </WorkspaceProvider>,
        );

        await waitFor(() => {
            expect(screen.getByText("Gestionar Workspaces")).toBeInTheDocument();
            expect(screen.getByText("por defecto")).toBeInTheDocument();
        });

        // Solo el workspace personalizado (ws-office) tiene botón de eliminar
        const deleteButtons = screen.queryAllByTitle("Eliminar workspace");
        expect(deleteButtons.length).toBe(1);
    });
});
