import { useState } from "react";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FormField } from "@/components/ui/form-field";
import { cn } from "@/lib/utils";
import { useWorkspace } from "./workspace-provider";
import { useToast } from "@/components/ui/toast";
import type { Workspace } from "@/lib/tauri";
import {
    Briefcase,
    Building,
    Folder,
    Home,
    Laptop,
    Network,
    Plus,
    Server,
    Shield,
    Trash2,
    Wifi,
} from "lucide-react";

export const PRESET_COLORS = [
    "#06b6d4", // Cyan
    "#3b82f6", // Blue
    "#8b5cf6", // Purple
    "#ec4899", // Pink
    "#10b981", // Emerald
    "#f59e0b", // Amber
    "#ef4444", // Red
    "#64748b", // Slate
];

export const WORKSPACE_ICONS: Record<string, typeof Network> = {
    network: Network,
    home: Home,
    building: Building,
    briefcase: Briefcase,
    server: Server,
    laptop: Laptop,
    wifi: Wifi,
    shield: Shield,
    folder: Folder,
};

export function WorkspaceIcon({
    icon,
    className,
}: {
    icon: string;
    className?: string;
}) {
    const IconComponent = WORKSPACE_ICONS[icon.toLowerCase()] ?? Network;
    return <IconComponent className={className ?? "h-4 w-4"} aria-hidden />;
}

interface WorkspaceManageDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    initialMode?: "create" | "manage";
}

export function WorkspaceManageDialog({
    open,
    onOpenChange,
    initialMode = "create",
}: WorkspaceManageDialogProps) {
    const {
        workspaces,
        createWorkspace,
        updateWorkspace,
        deleteWorkspace,
        activeWorkspaceId,
    } = useWorkspace();
    const { toast } = useToast();

    const [mode, setMode] = useState<"create" | "manage" | "edit">(initialMode);
    const [editingWs, setEditingWs] = useState<Workspace | null>(null);
    const [deletingWs, setDeletingWs] = useState<Workspace | null>(null);

    // Form fields
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [color, setColor] = useState(PRESET_COLORS[0]);
    const [icon, setIcon] = useState("network");
    const [nameError, setNameError] = useState<string | null>(null);
    const [saving, setSaving] = useState(false);

    function startCreate() {
        setName("");
        setDescription("");
        setColor(PRESET_COLORS[0]);
        setIcon("network");
        setNameError(null);
        setEditingWs(null);
        setMode("create");
    }

    function startEdit(ws: Workspace) {
        setEditingWs(ws);
        setName(ws.name);
        setDescription(ws.description ?? "");
        setColor(ws.color);
        setIcon(ws.icon);
        setNameError(null);
        setMode("edit");
    }

    async function handleSave() {
        const trimmed = name.trim();
        if (!trimmed) {
            setNameError("El nombre del workspace es obligatorio");
            return;
        }
        setNameError(null);

        setSaving(true);
        try {
            if (mode === "create") {
                await createWorkspace({
                    name: trimmed,
                    description: description.trim() || undefined,
                    color,
                    icon,
                });
                toast(`Workspace "${trimmed}" creado correctamente`, "success");
                onOpenChange(false);
            } else if (mode === "edit" && editingWs) {
                await updateWorkspace({
                    id: editingWs.id,
                    name: trimmed,
                    description: description.trim() || undefined,
                    color,
                    icon,
                });
                toast(`Workspace "${trimmed}" actualizado`, "success");
                setMode("manage");
            }
        } catch (e) {
            toast(`Error al guardar: ${e}`, "error");
        } finally {
            setSaving(false);
        }
    }

    function handleDelete(ws: Workspace) {
        if (ws.id === "default") {
            toast("El workspace por defecto no puede ser eliminado", "error");
            return;
        }
        setDeletingWs(ws);
    }

    async function confirmDeleteWorkspace() {
        if (!deletingWs) return;
        const ws = deletingWs;
        setDeletingWs(null);
        try {
            await deleteWorkspace(ws.id);
            toast(`Workspace "${ws.name}" eliminado`, "success");
        } catch (e) {
            toast(`Error al eliminar: ${e}`, "error");
        }
    }

    return (
        <>
            <Dialog
                open={open}
                onOpenChange={(isOpen) => {
                    if (!isOpen) {
                        setDeletingWs(null);
                        setNameError(null);
                    }
                    onOpenChange(isOpen);
                }}
            >
            <DialogContent className="sm:max-w-[480px]">
                <DialogHeader>
                    <DialogTitle>
                        {mode === "create" && "Crear Nuevo Workspace"}
                        {mode === "edit" && `Editar Workspace: ${editingWs?.name}`}
                        {mode === "manage" && "Gestionar Workspaces"}
                    </DialogTitle>
                    <DialogDescription>
                        {mode === "create" &&
                            "Organiza tus redes locales y auditorías en particiones aisladas por ubicación o proyecto."}
                        {mode === "edit" &&
                            "Modifica el nombre, etiqueta visual, icono o descripción del espacio de trabajo."}
                        {mode === "manage" &&
                            "Lista de todos los workspaces registrados en esta instalación de MyLAN."}
                    </DialogDescription>
                </DialogHeader>

                {mode === "manage" ? (
                    <div className="flex flex-col gap-3 py-2">
                        <div className="flex justify-end">
                            <Button
                                size="sm"
                                onClick={startCreate}
                                className="gap-1.5"
                            >
                                <Plus className="h-4 w-4" />
                                Nuevo Workspace
                            </Button>
                        </div>
                        <div className="flex flex-col divide-y rounded-md border">
                            {workspaces.map((w) => {
                                const isDefault = w.id === "default";
                                const isActive = w.id === activeWorkspaceId;
                                return (
                                    <div
                                        key={w.id}
                                        className="flex items-center justify-between p-3"
                                    >
                                        <div className="flex items-center gap-2.5 min-w-0">
                                            <span
                                                className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-white"
                                                style={{ backgroundColor: w.color }}
                                            >
                                                <WorkspaceIcon
                                                    icon={w.icon}
                                                    className="h-4 w-4"
                                                />
                                            </span>
                                            <div className="flex flex-col min-w-0">
                                                <div className="flex items-center gap-1.5">
                                                    <span className="truncate text-sm font-medium">
                                                        {w.name}
                                                    </span>
                                                    {isDefault && (
                                                        <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground font-mono">
                                                            por defecto
                                                        </span>
                                                    )}
                                                    {isActive && (
                                                        <span className="rounded bg-primary/20 px-1.5 py-0.5 text-[10px] font-semibold text-primary">
                                                            activo
                                                        </span>
                                                    )}
                                                </div>
                                                {w.description && (
                                                    <span className="truncate text-xs text-muted-foreground">
                                                        {w.description}
                                                    </span>
                                                )}
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => startEdit(w)}
                                            >
                                                Editar
                                            </Button>
                                            {!isDefault && (
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    className="text-destructive hover:bg-destructive/10"
                                                    onClick={() => handleDelete(w)}
                                                    title="Eliminar workspace"
                                                >
                                                    <Trash2 className="h-4 w-4" />
                                                </Button>
                                            )}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                ) : (
                    <div className="flex flex-col gap-4 py-2">
                        <FormField
                            label="Nombre del Workspace"
                            htmlFor="ws-name-input"
                            required
                            error={nameError || undefined}
                        >
                            <Input
                                id="ws-name-input"
                                value={name}
                                aria-invalid={Boolean(nameError)}
                                onChange={(e) => {
                                    setName(e.target.value);
                                    if (nameError && e.target.value.trim()) {
                                        setNameError(null);
                                    }
                                }}
                                placeholder="Ej. Oficina Principal, Laboratorio, Casa..."
                                autoFocus
                            />
                        </FormField>

                        <FormField
                            label="Descripción (opcional)"
                            htmlFor="ws-desc-input"
                        >
                            <Input
                                id="ws-desc-input"
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                placeholder="Notas o detalles de este espacio..."
                            />
                        </FormField>

                        <div className="flex flex-col gap-1.5">
                            <label className="text-xs font-semibold text-muted-foreground">
                                Color Distintivo
                            </label>
                            <div className="flex items-center gap-2">
                                {PRESET_COLORS.map((c) => (
                                    <Button
                                        key={c}
                                        type="button"
                                        variant="ghost"
                                        size="icon"
                                        onClick={() => setColor(c)}
                                        className={cn(
                                            "h-7 w-7 rounded-full p-0 transition-transform",
                                            color === c
                                                ? "scale-110 ring-2 ring-primary ring-offset-2"
                                                : "hover:scale-105"
                                        )}
                                        style={{ backgroundColor: c }}
                                        title={c}
                                    />
                                ))}
                            </div>
                        </div>

                        <div className="flex flex-col gap-1.5">
                            <label className="text-xs font-semibold text-muted-foreground">
                                Icono
                            </label>
                            <div className="grid grid-cols-5 gap-2">
                                {Object.keys(WORKSPACE_ICONS).map((iconKey) => {
                                    const IconComp = WORKSPACE_ICONS[iconKey];
                                    const isSelected = icon === iconKey;
                                    return (
                                        <Button
                                            key={iconKey}
                                            type="button"
                                            variant="ghost"
                                            onClick={() => setIcon(iconKey)}
                                            className={cn(
                                                "h-auto flex flex-col items-center justify-center gap-1 rounded-md border p-2 text-xs transition-colors",
                                                isSelected
                                                    ? "border-primary bg-primary/10 text-primary font-medium hover:bg-primary/15 hover:text-primary"
                                                    : "border-border text-muted-foreground hover:bg-muted"
                                            )}
                                        >
                                            <IconComp className="h-5 w-5" />
                                            <span className="capitalize text-[10px]">
                                                {iconKey}
                                            </span>
                                        </Button>
                                    );
                                })}
                            </div>
                        </div>
                    </div>
                )}

                <DialogFooter className="gap-2 sm:gap-0">
                    {mode !== "manage" && (
                        <>
                            {mode === "edit" && (
                                <Button
                                    variant="outline"
                                    onClick={() => setMode("manage")}
                                >
                                    Volver
                                </Button>
                            )}
                            <Button
                                variant="outline"
                                onClick={() => onOpenChange(false)}
                            >
                                Cancelar
                            </Button>
                            <Button onClick={handleSave} disabled={saving}>
                                {saving ? "Guardando..." : "Guardar Workspace"}
                            </Button>
                        </>
                    )}
                    {mode === "manage" && (
                        <Button
                            variant="outline"
                            onClick={() => onOpenChange(false)}
                        >
                            Cerrar
                        </Button>
                    )}
                </DialogFooter>
            </DialogContent>
        </Dialog>

        {/* Modal de confirmación para eliminar workspace */}
        <Dialog
            open={Boolean(deletingWs)}
            onOpenChange={(isOpen) => {
                if (!isOpen) setDeletingWs(null);
            }}
        >
            <DialogContent className="sm:max-w-[420px]">
                <DialogHeader>
                    <DialogTitle>Eliminar Workspace</DialogTitle>
                    <DialogDescription>
                        ¿Deseas eliminar el workspace "{deletingWs?.name}"? Los datos asociados se moverán al workspace por defecto.
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter className="gap-2 sm:gap-0">
                    <Button
                        type="button"
                        variant="outline"
                        onClick={() => setDeletingWs(null)}
                    >
                        Cancelar
                    </Button>
                    <Button
                        type="button"
                        variant="destructive"
                        onClick={confirmDeleteWorkspace}
                    >
                        Eliminar
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
        </>
    );
}
