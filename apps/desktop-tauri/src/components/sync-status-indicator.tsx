import { useState, useEffect } from "react";
import { Cloud, CloudAlert, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { getGdriveSyncStatus, type GdriveSyncStatus } from "@/lib/tauri";
import { GdriveSyncDialog } from "@/components/gdrive-sync-dialog";

import { cn } from "@/lib/utils";

interface SyncStatusIndicatorProps {
    className?: string;
    showLabel?: boolean;
}

export function SyncStatusIndicator({ className, showLabel }: SyncStatusIndicatorProps) {
    const [status, setStatus] = useState<GdriveSyncStatus | null>(null);
    const [isDialogOpen, setIsDialogOpen] = useState(false);
    const [isSyncing, setIsSyncing] = useState(false);

    const refreshStatus = async () => {
        try {
            const s = await getGdriveSyncStatus();
            setStatus(s);
        } catch {
            // Silencioso si falla
        }
    };

    useEffect(() => {
        refreshStatus();
        const interval = setInterval(refreshStatus, 30_000);
        return () => clearInterval(interval);
    }, []);

    let indicatorIcon = <Cloud className="h-3.5 w-3.5 opacity-60" />;
    let label = "Nube";
    let tooltipText = "Google Drive Desconectado. Clic para configurar.";
    let badgeClass = "text-muted-foreground border-border hover:bg-muted/50";

    if (isSyncing) {
        indicatorIcon = <RefreshCw className="h-3.5 w-3.5 animate-spin text-amber-500" />;
        label = "Sincronizando...";
        tooltipText = "Sincronizando paquete cifrado con Google Drive...";
        badgeClass = "text-amber-500 border-amber-500/40 bg-amber-500/10";
    } else if (status?.is_authenticated && status?.has_master_key) {
        indicatorIcon = <Cloud className="h-3.5 w-3.5 text-emerald-500" />;
        label = "Sincronizado";
        tooltipText = `Zero-Knowledge Sync activo (${status.email || "Conectado"}). Última sync: ${
            status.last_synced_at ? new Date(status.last_synced_at).toLocaleTimeString() : "Pendiente"
        }`;
        badgeClass = "text-emerald-500 border-emerald-500/30 bg-emerald-500/10 hover:bg-emerald-500/20";
    } else if (status?.is_authenticated && !status?.has_master_key) {
        indicatorIcon = <CloudAlert className="h-3.5 w-3.5 text-amber-500" />;
        label = "Configurar Clave";
        tooltipText = "Google Drive conectado. Requiere configurar Contraseña Maestra.";
        badgeClass = "text-amber-500 border-amber-500/40 bg-amber-500/10 hover:bg-amber-500/20";
    }

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setIsDialogOpen(true)}
                        className={cn(
                            "h-8 gap-1.5 px-2.5 text-xs font-medium border transition-colors",
                            badgeClass,
                            className,
                        )}
                    >
                        {indicatorIcon}
                        <span className={showLabel ? "truncate" : "hidden sm:inline truncate"}>
                            {label}
                        </span>
                    </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom" className="text-xs max-w-xs">
                    {tooltipText}
                </TooltipContent>
            </Tooltip>

            <GdriveSyncDialog
                open={isDialogOpen}
                onOpenChange={setIsDialogOpen}
                status={status}
                onStatusChange={setStatus}
                onSyncStart={() => setIsSyncing(true)}
                onSyncEnd={() => {
                    setIsSyncing(false);
                    refreshStatus();
                }}
            />
        </TooltipProvider>
    );
}
