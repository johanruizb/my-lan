import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useState,
    type ReactNode,
} from "react";
import {
    createWorkspace as apiCreateWorkspace,
    deleteWorkspace as apiDeleteWorkspace,
    getSettings,
    listWorkspaces as apiListWorkspaces,
    setSettings,
    updateWorkspace as apiUpdateWorkspace,
    type CreateWorkspacePayload,
    type UpdateWorkspacePayload,
    type Workspace,
} from "@/lib/tauri";

export const DEFAULT_WORKSPACE: Workspace = {
    id: "default",
    name: "Mi Red Local",
    description: "Workspace principal por defecto",
    icon: "home",
    color: "#06b6d4",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
};

interface WorkspaceContextValue {
    workspaces: Workspace[];
    activeWorkspaceId: string;
    activeWorkspace: Workspace;
    setActiveWorkspaceId: (id: string) => Promise<void>;
    refreshWorkspaces: () => Promise<void>;
    createWorkspace: (payload: CreateWorkspacePayload) => Promise<Workspace>;
    updateWorkspace: (payload: UpdateWorkspacePayload) => Promise<Workspace>;
    deleteWorkspace: (id: string) => Promise<void>;
    loading: boolean;
}

const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({
    children,
    initialWorkspaceId,
}: {
    children: ReactNode;
    initialWorkspaceId?: string;
}) {
    const [workspaces, setWorkspaces] = useState<Workspace[]>([DEFAULT_WORKSPACE]);
    const [activeWorkspaceId, setActiveState] = useState<string>(
        initialWorkspaceId ?? "default",
    );
    const [loading, setLoading] = useState<boolean>(true);

    const refreshWorkspaces = useCallback(async () => {
        try {
            const list = await apiListWorkspaces();
            if (list && list.length > 0) {
                setWorkspaces(list);
            }
        } catch {
            // Si falla la llamada inicial, se mantiene el default
        }
    }, []);

    useEffect(() => {
        let mounted = true;
        Promise.all([apiListWorkspaces().catch(() => [DEFAULT_WORKSPACE]), getSettings().catch(() => null)])
            .then(([list, settings]) => {
                if (!mounted) return;
                if (list && list.length > 0) {
                    setWorkspaces(list);
                }
                if (initialWorkspaceId) {
                    setActiveState(initialWorkspaceId);
                } else if (settings?.active_workspace_id) {
                    setActiveState(settings.active_workspace_id);
                }
            })
            .finally(() => {
                if (mounted) setLoading(false);
            });
        return () => {
            mounted = false;
        };
    }, [initialWorkspaceId]);

    const setActiveWorkspaceId = useCallback(async (id: string) => {
        setActiveState(id);
        try {
            const s = await getSettings();
            await setSettings({ ...s, active_workspace_id: id });
        } catch {
            // Mantener estado en memoria si falla persistencia
        }
    }, []);

    const createWorkspace = useCallback(
        async (payload: CreateWorkspacePayload) => {
            const created = await apiCreateWorkspace(payload);
            await refreshWorkspaces();
            await setActiveWorkspaceId(created.id);
            return created;
        },
        [refreshWorkspaces, setActiveWorkspaceId],
    );

    const updateWorkspace = useCallback(
        async (payload: UpdateWorkspacePayload) => {
            const updated = await apiUpdateWorkspace(payload);
            await refreshWorkspaces();
            return updated;
        },
        [refreshWorkspaces],
    );

    const deleteWorkspace = useCallback(
        async (id: string) => {
            await apiDeleteWorkspace(id);
            if (activeWorkspaceId === id) {
                await setActiveWorkspaceId("default");
            }
            await refreshWorkspaces();
        },
        [activeWorkspaceId, refreshWorkspaces, setActiveWorkspaceId],
    );

    const activeWorkspace = useMemo(() => {
        return (
            workspaces.find((w) => w.id === activeWorkspaceId) ??
            workspaces[0] ??
            DEFAULT_WORKSPACE
        );
    }, [workspaces, activeWorkspaceId]);

    const value = useMemo<WorkspaceContextValue>(
        () => ({
            workspaces,
            activeWorkspaceId,
            activeWorkspace,
            setActiveWorkspaceId,
            refreshWorkspaces,
            createWorkspace,
            updateWorkspace,
            deleteWorkspace,
            loading,
        }),
        [
            workspaces,
            activeWorkspaceId,
            activeWorkspace,
            setActiveWorkspaceId,
            refreshWorkspaces,
            createWorkspace,
            updateWorkspace,
            deleteWorkspace,
            loading,
        ],
    );

    return (
        <WorkspaceContext.Provider value={value}>
            {children}
        </WorkspaceContext.Provider>
    );
}

export function useWorkspace(): WorkspaceContextValue {
    const ctx = useContext(WorkspaceContext);
    if (!ctx) {
        return {
            workspaces: [DEFAULT_WORKSPACE],
            activeWorkspaceId: "default",
            activeWorkspace: DEFAULT_WORKSPACE,
            setActiveWorkspaceId: async () => {},
            refreshWorkspaces: async () => {},
            createWorkspace: async () => DEFAULT_WORKSPACE,
            updateWorkspace: async () => DEFAULT_WORKSPACE,
            deleteWorkspace: async () => {},
            loading: false,
        };
    }
    return ctx;
}
