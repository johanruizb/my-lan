import { useState } from "react";
import {
    Cloud,
    CloudAlert,
    Shield,
    ShieldCheck,
    Key,
    Lock,
    Copy,
    Check,
    RefreshCw,
    LogOut,
    AlertTriangle,
    ExternalLink,
    FileCheck,
} from "lucide-react";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";
import { useWorkspace } from "@/components/workspace-provider";
import {
    startGdriveAuth,
    disconnectGdrive,
    setupMasterKey,
    restoreMasterKey,
    syncWorkspace,
    resolveSyncConflict,
    type GdriveSyncStatus,
    type SyncOutcome,
} from "@/lib/tauri";

interface GdriveSyncDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    status: GdriveSyncStatus | null;
    onStatusChange: (status: GdriveSyncStatus) => void;
    onSyncStart: () => void;
    onSyncEnd: () => void;
}

export function GdriveSyncDialog({
    open,
    onOpenChange,
    status,
    onStatusChange,
    onSyncStart,
    onSyncEnd,
}: GdriveSyncDialogProps) {
    const { toast } = useToast();
    const { activeWorkspace } = useWorkspace();

    // Estados internos
    const [isAuthenticating, setIsAuthenticating] = useState(false);
    const [isSyncing, setIsSyncing] = useState(false);
    const [passphrase, setPassphrase] = useState("");
    const [restoreInput, setRestoreInput] = useState("");
    const [mnemonicDisplay, setMnemonicDisplay] = useState<string | null>(null);
    const [copiedMnemonic, setCopiedMnemonic] = useState(false);
    const [activeTab, setActiveTab] = useState<"new_key" | "restore_key">("new_key");
    const [showKeyForm, setShowKeyForm] = useState(false);
    const [pendingConflict, setPendingConflict] = useState<Extract<SyncOutcome, { status: "conflict" }> | null>(null);

    // Iniciar flujo OAuth 2.0 PKCE con Google
    const handleStartAuth = async () => {
        setIsAuthenticating(true);
        try {
            const res = await startGdriveAuth();
            // Abrir navegador
            window.open(res.auth_url, "_blank");
            toast({
                title: "Autorización en progreso",
                description: "Se abrió la ventana del navegador. Autoriza a MyLAN para continuar.",
            });
        } catch (e: any) {
            toast({
                title: "Error al iniciar conexión",
                description: String(e),
                variant: "destructive",
            });
            setIsAuthenticating(false);
        }
    };

    // Desconectar cuenta de Google
    const handleDisconnect = async () => {
        try {
            await disconnectGdrive();
            onStatusChange({
                is_authenticated: false,
                email: null,
                has_master_key: status?.has_master_key ?? false,
                last_synced_at: null,
                sync_enabled: false,
            });
            toast({
                title: "Cuenta desconectada",
                description: "Google Drive ha sido desvinculado de este equipo.",
            });
        } catch (e: any) {
            toast({
                title: "Error al desconectar",
                description: String(e),
                variant: "destructive",
            });
        }
    };

    // Configurar nueva contraseña maestra
    const handleSetupKey = async () => {
        if (!passphrase || passphrase.length < 8) {
            toast({
                title: "Contraseña muy corta",
                description: "La contraseña maestra debe tener al menos 8 caracteres.",
                variant: "destructive",
            });
            return;
        }

        try {
            const res = await setupMasterKey(passphrase);
            setMnemonicDisplay(res.mnemonic_phrase);
            onStatusChange({
                ...(status ?? {
                    is_authenticated: false,
                    email: null,
                    last_synced_at: null,
                    sync_enabled: false,
                }),
                has_master_key: true,
            });
            setShowKeyForm(false);
            setPassphrase("");
            toast({
                title: "Clave Maestra configurada",
                description: "Cifrado Zero-Knowledge activo. Guarda tu frase de recuperación.",
            });
        } catch (e: any) {
            toast({
                title: "Error al configurar clave",
                description: String(e),
                variant: "destructive",
            });
        }
    };

    // Restaurar clave maestra desde mnemónico o contraseña
    const handleRestoreKey = async () => {
        if (!restoreInput.trim()) return;

        try {
            await restoreMasterKey(restoreInput.trim());
            onStatusChange({
                ...(status ?? {
                    is_authenticated: false,
                    email: null,
                    last_synced_at: null,
                    sync_enabled: false,
                }),
                has_master_key: true,
            });
            setShowKeyForm(false);
            setRestoreInput("");
            toast({
                title: "Clave Maestra restaurada",
                description: "Se restauró correctamente el acceso Zero-Knowledge.",
            });
        } catch (e: any) {
            toast({
                title: "Error al restaurar clave",
                description: String(e),
                variant: "destructive",
            });
        }
    };

    // Copiar mnemónico al portapapeles
    const handleCopyMnemonic = () => {
        if (!mnemonicDisplay) return;
        navigator.clipboard.writeText(mnemonicDisplay);
        setCopiedMnemonic(true);
        setTimeout(() => setCopiedMnemonic(false), 2000);
        toast({
            title: "Copiado al portapapeles",
            description: "Frase de 12 palabras copiada.",
        });
    };

    // Ejecutar sincronización manual
    const handleSyncNow = async (force = false) => {
        setIsSyncing(true);
        onSyncStart();
        try {
            const outcome = await syncWorkspace(activeWorkspace.id, force);

            if (outcome.status === "conflict") {
                setPendingConflict(outcome);
            } else if (outcome.status === "uploaded") {
                setPendingConflict(null);
                toast({
                    title: "Sincronización exitosa",
                    description: `Paquete cifrado subido a Google Drive (Workspace: ${activeWorkspace.name}).`,
                });
            } else if (outcome.status === "downloaded") {
                setPendingConflict(null);
                toast({
                    title: "Sincronización exitosa",
                    description: `Se descargaron y aplicaron los cambios desde Google Drive (${outcome.summary.devices_imported} dispositivos).`,
                });
            } else {
                setPendingConflict(null);
                toast({
                    title: "Al día",
                    description: "El Workspace ya se encuentra totalmente sincronizado.",
                });
            }
        } catch (e: any) {
            toast({
                title: "Error en la sincronización",
                description: String(e),
                variant: "destructive",
            });
        } finally {
            setIsSyncing(false);
            onSyncEnd();
        }
    };

    // Resolver conflicto
    const handleResolveConflict = async (resolution: "adopt_cloud" | "keep_local_and_overwrite") => {
        setIsSyncing(true);
        try {
            const outcome = await resolveSyncConflict(activeWorkspace.id, resolution);
            setPendingConflict(null);
            toast({
                title: "Conflicto resuelto",
                description:
                    resolution === "adopt_cloud"
                        ? "Se adoptó la versión de la nube. La copia local quedó en backup preventivo."
                        : "Se conservó la versión local y se sobrescribió Google Drive.",
            });
        } catch (e: any) {
            toast({
                title: "Error resolviendo conflicto",
                description: String(e),
                variant: "destructive",
            });
        } finally {
            setIsSyncing(false);
            onSyncEnd();
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-xl max-h-[85vh] overflow-y-auto">
                <DialogHeader>
                    <div className="flex items-center gap-2">
                        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-cyan-500/10 text-cyan-500">
                            <Cloud className="h-5 w-5" />
                        </div>
                        <div>
                            <DialogTitle className="text-base font-semibold">
                                Sincronización en la Nube
                            </DialogTitle>
                            <DialogDescription className="text-xs text-muted-foreground flex items-center gap-1.5 mt-0.5">
                                <span>Google Drive</span>
                                <span>•</span>
                                <span className="text-emerald-500 font-medium flex items-center gap-1">
                                    <ShieldCheck className="h-3.5 w-3.5" /> Cifrado Zero-Knowledge
                                </span>
                            </DialogDescription>
                        </div>
                    </div>
                </DialogHeader>

                <div className="space-y-4 pt-2">
                    {/* Alerta de Conflicto Activo si existe */}
                    {pendingConflict && (
                        <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3.5 space-y-3">
                            <div className="flex items-start gap-2.5">
                                <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0 mt-0.5" />
                                <div className="space-y-1">
                                    <h4 className="text-xs font-semibold text-amber-500">
                                        Conflicto de versiones detectado
                                    </h4>
                                    <p className="text-[11px] text-muted-foreground leading-relaxed">
                                        Se detectaron cambios simultáneos en Google Drive generados desde otro dispositivo.
                                        Se ha creado automáticamente un respaldo preventivo local.
                                    </p>
                                </div>
                            </div>
                            <div className="flex items-center gap-2 pt-1">
                                <Button
                                    size="sm"
                                    variant="outline"
                                    onClick={() => handleResolveConflict("adopt_cloud")}
                                    className="h-8 text-xs flex-1"
                                    disabled={isSyncing}
                                >
                                    Adoptar versión de la nube
                                </Button>
                                <Button
                                    size="sm"
                                    onClick={() => handleResolveConflict("keep_local_and_overwrite")}
                                    className="h-8 text-xs flex-1 bg-amber-600 hover:bg-amber-700 text-white"
                                    disabled={isSyncing}
                                >
                                    Conservar versión local
                                </Button>
                            </div>
                        </div>
                    )}

                    {/* Card 1: Cuenta de Google Drive */}
                    <div className="rounded-lg border border-border/60 bg-card p-3.5 space-y-2.5">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <Cloud className="h-4 w-4 text-cyan-500" />
                                <span className="text-xs font-semibold">1. Cuenta de Google</span>
                            </div>
                            {status?.is_authenticated ? (
                                <span className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded">
                                    <Check className="h-3 w-3" /> Conectado
                                </span>
                            ) : (
                                <span className="text-[11px] text-muted-foreground">Desconectado</span>
                            )}
                        </div>

                        {status?.is_authenticated ? (
                            <div className="flex items-center justify-between pt-1">
                                <div className="text-xs">
                                    <span className="text-muted-foreground">Cuenta activa: </span>
                                    <strong className="text-foreground">{status.email || "Google Drive"}</strong>
                                </div>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={handleDisconnect}
                                    className="h-7 text-xs text-rose-500 hover:text-rose-600 hover:bg-rose-500/10"
                                >
                                    <LogOut className="h-3 w-3 mr-1" /> Desconectar
                                </Button>
                            </div>
                        ) : (
                            <div className="space-y-2 pt-1">
                                <p className="text-[11px] text-muted-foreground">
                                    MyLAN utiliza la carpeta privada <code className="text-[10px] bg-muted px-1 py-0.5 rounded">drive.appdata</code>.
                                    Tus archivos quedan aislados y son completamente invisibles en tu Google Drive personal.
                                </p>
                                <Button
                                    size="sm"
                                    onClick={handleStartAuth}
                                    disabled={isAuthenticating}
                                    className="w-full h-8 text-xs gap-1.5 bg-cyan-600 hover:bg-cyan-700 text-white"
                                >
                                    {isAuthenticating ? (
                                        <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                                    ) : (
                                        <ExternalLink className="h-3.5 w-3.5" />
                                    )}
                                    {isAuthenticating ? "Esperando autorización..." : "Conectar con Google Drive"}
                                </Button>
                            </div>
                        )}
                    </div>

                    {/* Card 2: Cifrado Zero-Knowledge (Contraseña Maestra) */}
                    <div className="rounded-lg border border-border/60 bg-card p-3.5 space-y-2.5">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <Shield className="h-4 w-4 text-emerald-500" />
                                <span className="text-xs font-semibold">2. Cifrado Zero-Knowledge</span>
                            </div>
                            {status?.has_master_key ? (
                                <span className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded">
                                    <Lock className="h-3 w-3" /> Protegido
                                </span>
                            ) : (
                                <span className="text-[11px] text-amber-500 font-medium">Requerido</span>
                            )}
                        </div>

                        {status?.has_master_key && !showKeyForm ? (
                            <div className="flex items-center justify-between pt-1">
                                <div className="text-[11px] text-muted-foreground flex items-center gap-1.5">
                                    <span>Argon2id (64MB) + XChaCha20-Poly1305</span>
                                </div>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => setShowKeyForm(true)}
                                    className="h-7 text-xs"
                                >
                                    Restaurar / Reconfigurar
                                </Button>
                            </div>
                        ) : (
                            <div className="space-y-3 pt-1">
                                <div className="flex gap-2 border-b border-border/60 pb-2">
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => setActiveTab("new_key")}
                                        className={cn(
                                            "h-7 rounded-none border-b-2 px-2 pb-1 text-xs font-medium transition-colors shadow-none",
                                            activeTab === "new_key"
                                                ? "border-primary text-primary"
                                                : "border-transparent text-muted-foreground hover:text-foreground hover:bg-transparent"
                                        )}
                                    >
                                        Crear Contraseña Maestra
                                    </Button>
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => setActiveTab("restore_key")}
                                        className={cn(
                                            "h-7 rounded-none border-b-2 px-2 pb-1 text-xs font-medium transition-colors shadow-none",
                                            activeTab === "restore_key"
                                                ? "border-primary text-primary"
                                                : "border-transparent text-muted-foreground hover:text-foreground hover:bg-transparent"
                                        )}
                                    >
                                        Restaurar (12 palabras)
                                    </Button>
                                </div>

                                {activeTab === "new_key" ? (
                                    <div className="space-y-2">
                                        <p className="text-[11px] text-muted-foreground">
                                            Ingresa una contraseña para derivar tu clave privada de cifrado local:
                                        </p>
                                        <div className="flex gap-2">
                                            <Input
                                                type="password"
                                                placeholder="Contraseña segura (mín. 8 carácteres)"
                                                value={passphrase}
                                                onChange={(e) => setPassphrase(e.target.value)}
                                                className="h-8 text-xs"
                                            />
                                            <Button
                                                size="sm"
                                                onClick={handleSetupKey}
                                                className="h-8 text-xs shrink-0 bg-emerald-600 hover:bg-emerald-700 text-white"
                                            >
                                                Crear Clave
                                            </Button>
                                        </div>
                                    </div>
                                ) : (
                                    <div className="space-y-2">
                                        <p className="text-[11px] text-muted-foreground">
                                            Ingresa tus 12 palabras de recuperación o tu contraseña previa:
                                        </p>
                                        <div className="flex gap-2">
                                            <Input
                                                placeholder="Ej. apple banana cat dog..."
                                                value={restoreInput}
                                                onChange={(e) => setRestoreInput(e.target.value)}
                                                className="h-8 text-xs"
                                            />
                                            <Button
                                                size="sm"
                                                onClick={handleRestoreKey}
                                                className="h-8 text-xs shrink-0"
                                            >
                                                Restaurar
                                            </Button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Mnemonic Display Card */}
                        {mnemonicDisplay && (
                            <div className="rounded-md border border-cyan-500/40 bg-cyan-500/5 p-3 space-y-2">
                                <div className="flex items-center justify-between">
                                    <span className="text-xs font-semibold text-cyan-500 flex items-center gap-1.5">
                                        <Key className="h-3.5 w-3.5" /> Frase de Recuperación (12 palabras BIP-39)
                                    </span>
                                    <Button
                                        size="sm"
                                        variant="outline"
                                        onClick={handleCopyMnemonic}
                                        className="h-7 text-xs gap-1"
                                    >
                                        {copiedMnemonic ? <Check className="h-3 w-3 text-emerald-500" /> : <Copy className="h-3 w-3" />}
                                        {copiedMnemonic ? "Copiado" : "Copiar"}
                                    </Button>
                                </div>
                                <div className="grid grid-cols-3 sm:grid-cols-4 gap-1.5 font-mono text-[11px] bg-background/80 p-2.5 rounded border border-border/50">
                                    {mnemonicDisplay.split(" ").map((word, i) => (
                                        <div key={i} className="flex gap-1 text-muted-foreground">
                                            <span className="opacity-40">{i + 1}.</span>
                                            <span className="text-foreground font-semibold">{word}</span>
                                        </div>
                                    ))}
                                </div>
                                <p className="text-[10px] text-muted-foreground italic">
                                    Guarda estas palabras en un lugar seguro. MyLAN nunca tiene acceso a tu clave sin descifrar.
                                </p>
                            </div>
                        )}
                    </div>

                    {/* Card 3: Estado y Sincronización del Workspace Activo */}
                    <div className="rounded-lg border border-border/60 bg-card p-3.5 space-y-3">
                        <div className="flex items-center justify-between">
                            <span className="text-xs font-semibold flex items-center gap-2">
                                <RefreshCw className={`h-4 w-4 text-cyan-500 ${isSyncing ? "animate-spin" : ""}`} />
                                3. Sincronizar Workspace Activo
                            </span>
                            <span className="text-xs font-medium text-foreground">
                                {activeWorkspace.name}
                            </span>
                        </div>

                        <div className="flex items-center justify-between text-xs text-muted-foreground pt-1">
                            <span>Última sincronización:</span>
                            <span className="font-medium text-foreground">
                                {status?.last_synced_at
                                    ? new Date(status.last_synced_at).toLocaleString()
                                    : "Nunca"}
                            </span>
                        </div>

                        <Button
                            size="sm"
                            onClick={() => handleSyncNow(false)}
                            disabled={!status?.is_authenticated || !status?.has_master_key || isSyncing}
                            className="w-full h-8 text-xs gap-1.5 bg-primary text-primary-foreground hover:bg-primary/90"
                        >
                            {isSyncing ? (
                                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                            ) : (
                                <Cloud className="h-3.5 w-3.5" />
                            )}
                            {isSyncing ? "Sincronizando..." : "Sincronizar Ahora"}
                        </Button>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    );
}
