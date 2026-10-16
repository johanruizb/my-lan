import { useState } from "react";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { Check, ChevronDown, Plus, Settings } from "lucide-react";
import { useWorkspace } from "./workspace-provider";
import {
    WorkspaceIcon,
    WorkspaceManageDialog,
} from "./workspace-manage-dialog";
import { cn } from "@/lib/utils";

export function WorkspaceSelector({ className }: { className?: string }) {
    const { workspaces, activeWorkspace, setActiveWorkspaceId } =
        useWorkspace();
    const [dialogOpen, setDialogOpen] = useState(false);
    const [dialogMode, setDialogMode] = useState<"create" | "manage">("create");

    return (
        <>
            <div className={cn("w-full", className)}>
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button
                            variant="outline"
                            className="w-full justify-between gap-2 px-3 py-2 h-auto text-left border-border/60 hover:bg-muted/50"
                        >
                            <div className="flex items-center gap-2.5 min-w-0">
                                <span
                                    className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-white shadow-xs"
                                    style={{
                                        backgroundColor: activeWorkspace.color,
                                    }}
                                >
                                    <WorkspaceIcon
                                        icon={activeWorkspace.icon}
                                        className="h-3 w-3"
                                    />
                                </span>
                                <div className="flex flex-col min-w-0">
                                    <span className="truncate text-xs font-semibold leading-tight text-foreground">
                                        {activeWorkspace.name}
                                    </span>
                                    <span className="text-[10px] text-muted-foreground leading-tight">
                                        Workspace
                                    </span>
                                </div>
                            </div>
                            <ChevronDown className="h-3.5 w-3.5 shrink-0 opacity-50" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-56">
                        <DropdownMenuLabel className="text-xs text-muted-foreground">
                            Espacios de Trabajo
                        </DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        {workspaces.map((w) => {
                            const isSelected = w.id === activeWorkspace.id;
                            return (
                                <DropdownMenuItem
                                    key={w.id}
                                    onClick={() => setActiveWorkspaceId(w.id)}
                                    className="flex items-center justify-between gap-2 cursor-pointer"
                                >
                                    <div className="flex items-center gap-2 min-w-0">
                                        <span
                                            className="flex h-4 w-4 shrink-0 items-center justify-center rounded-xs text-white"
                                            style={{ backgroundColor: w.color }}
                                        >
                                            <WorkspaceIcon
                                                icon={w.icon}
                                                className="h-2.5 w-2.5"
                                            />
                                        </span>
                                        <span className="truncate text-xs font-medium">
                                            {w.name}
                                        </span>
                                    </div>
                                    {isSelected && (
                                        <Check className="h-3.5 w-3.5 text-primary shrink-0" />
                                    )}
                                </DropdownMenuItem>
                            );
                        })}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                            onClick={() => {
                                setDialogMode("create");
                                setDialogOpen(true);
                            }}
                            className="flex items-center gap-2 text-xs cursor-pointer"
                        >
                            <Plus className="h-3.5 w-3.5 text-muted-foreground" />
                            Nuevo Workspace...
                        </DropdownMenuItem>
                        <DropdownMenuItem
                            onClick={() => {
                                setDialogMode("manage");
                                setDialogOpen(true);
                            }}
                            className="flex items-center gap-2 text-xs cursor-pointer"
                        >
                            <Settings className="h-3.5 w-3.5 text-muted-foreground" />
                            Gestionar Workspaces...
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>

            <WorkspaceManageDialog
                open={dialogOpen}
                onOpenChange={setDialogOpen}
                initialMode={dialogMode}
            />
        </>
    );
}
