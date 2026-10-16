import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
    Info,
    Tag,
    Scale,
    User,
    Github,
    Bug,
    Package,
    ExternalLink,
    RefreshCw,
    Download,
    CheckCircle2,
    AlertCircle,
    Loader2,
    RotateCw,
} from "lucide-react";
import { getAppVersion } from "@/lib/tauri";
import { useUpdater } from "@/components/updater-provider";
import { Progress } from "@/components/ui/progress";

import { cn } from "@/lib/utils";

// Acerca de (AC-5/AC-6): versión unificada (leída en runtime con getVersion,
// misma fuente que el pie de la sidebar), repo, licencia, autor y enlaces a
// Issues/releases. Los enlaces externos se abren en el navegador del sistema
// vía `tauri-plugin-opener` (NUNCA con <a target="_blank">, que el webview no
// abre de forma fiable).
//
// T18: migrado de pantalla propia a Dialog Radix (ui/dialog.tsx) con focus
// trap/restore/Escape/scroll lock. Se monta una sola vez en App.tsx como
// <AboutDialog open onOpenChange />, y se abre desde el botón "Acerca de" del
// SidebarFooter y el 5º item del top-nav móvil. La pantalla /about legacy se
// elimina en T12.

const REPO_URL = "https://github.com/johanruizb/my-lan";
const ISSUES_URL = `${REPO_URL}/issues`;
const RELEASES_URL = `${REPO_URL}/releases`;

function useAppVersion() {
    const [version, setVersion] = useState("");
    useEffect(() => {
        getAppVersion()
            .then(setVersion)
            .catch(() => {});
    }, []);
    return version;
}

export function AboutDialog({
    open,
    onOpenChange,
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}) {
    const version = useAppVersion();
    const {
        status,
        updateInfo,
        progress,
        error,
        checkForUpdates,
        downloadAndInstall,
        restartApp,
    } = useUpdater();

    async function openLink(url: string) {
        try {
            await openUrl(url);
        } catch {
            // Opener no disponible: no-op (no navegar dentro del webview).
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <div className="flex items-center gap-2">
                    <Info className="h-5 w-5 text-primary" aria-hidden />
                    <DialogTitle>
                        Acerca de <span translate="no">MyLAN</span>
                    </DialogTitle>
                </div>
                <DialogDescription>
                    Escáner de red local de escritorio.
                </DialogDescription>
                <div className="flex flex-col gap-4">
                    <MetaRow label="Versión" icon={Tag}>
                        <span
                            className="font-mono text-xs text-muted-foreground"
                            translate="no"
                        >
                            v{version || "…"}
                        </span>
                    </MetaRow>

                    {/* Panel interactivo de actualización in-app */}
                    <div className="flex flex-col gap-2.5 rounded-lg border bg-muted/30 p-3">
                        <div className="flex items-center justify-between gap-2">
                            <span className="text-xs font-medium text-foreground">
                                Actualizaciones de la app
                            </span>
                            {status === "checking" ? (
                                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                    <Loader2
                                        className="h-3 w-3 animate-spin text-primary"
                                        aria-hidden
                                    />
                                    Buscando...
                                </span>
                            ) : status === "available" ? (
                                <span className="inline-flex items-center gap-1 rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-semibold text-primary">
                                    v{updateInfo?.version} disponible
                                </span>
                            ) : status === "downloading" ? (
                                <span className="flex items-center gap-1.5 text-xs text-primary">
                                    <Loader2
                                        className="h-3 w-3 animate-spin"
                                        aria-hidden
                                    />
                                    Descargando...
                                </span>
                            ) : status === "ready" ? (
                                <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-semibold text-emerald-600 dark:text-emerald-400">
                                    Lista para reiniciar
                                </span>
                            ) : status === "up-to-date" ? (
                                <span className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
                                    <CheckCircle2
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    Al día
                                </span>
                            ) : status === "error" ? (
                                <span className="flex items-center gap-1 text-xs text-destructive">
                                    <AlertCircle
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    Error
                                </span>
                            ) : null}
                        </div>

                        {status === "available" && (
                            <div className="flex flex-col gap-2">
                                <p className="text-xs text-muted-foreground">
                                    Hay una nueva versión disponible (v
                                    {updateInfo?.version}).
                                </p>
                                <Button
                                    size="sm"
                                    onClick={() => void downloadAndInstall()}
                                    className="h-8 gap-1.5 text-xs font-medium"
                                >
                                    <Download
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    Descargar e instalar ahora
                                </Button>
                            </div>
                        )}

                        {status === "downloading" && (
                            <div className="flex flex-col gap-1.5">
                                <Progress
                                    value={progress?.percent ?? 0}
                                    className="h-2"
                                />
                                <div className="flex justify-between font-mono text-[11px] text-muted-foreground">
                                    <span>
                                        {progress?.downloaded
                                            ? `${(
                                                  progress.downloaded /
                                                  (1024 * 1024)
                                              ).toFixed(1)} MB`
                                            : "0 MB"}
                                        {progress?.total
                                            ? ` / ${(
                                                  progress.total /
                                                  (1024 * 1024)
                                              ).toFixed(1)} MB`
                                            : ""}
                                    </span>
                                    <span>
                                        {progress?.percent != null
                                            ? `${progress.percent}%`
                                            : "..."}
                                    </span>
                                </div>
                            </div>
                        )}

                        {status === "ready" && (
                            <div className="flex flex-col gap-2">
                                <p className="text-xs text-muted-foreground">
                                    La actualización se ha instalado
                                    correctamente. Reinicia para aplicar los
                                    cambios.
                                </p>
                                <Button
                                    size="sm"
                                    onClick={() => void restartApp()}
                                    className="h-8 gap-1.5 text-xs font-medium"
                                >
                                    <RotateCw
                                        className="h-3.5 w-3.5"
                                        aria-hidden
                                    />
                                    Reiniciar MyLAN
                                </Button>
                            </div>
                        )}

                        {status === "error" && error && (
                            <Alert
                                variant="destructive"
                                className="py-2.5 px-3"
                            >
                                <AlertCircle className="h-4 w-4" />
                                <AlertTitle className="text-xs">
                                    Error de actualización
                                </AlertTitle>
                                <AlertDescription className="break-words text-[11px] leading-tight">
                                    {error}
                                </AlertDescription>
                            </Alert>
                        )}

                        {status !== "available" &&
                            status !== "downloading" &&
                            status !== "ready" && (
                                <div className="flex justify-end">
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() =>
                                            void checkForUpdates({
                                                silent: false,
                                            })
                                        }
                                        disabled={status === "checking"}
                                        className="h-7 gap-1.5 text-xs"
                                    >
                                        <RefreshCw
                                            className={cn(
                                                "h-3 w-3",
                                                status === "checking" &&
                                                    "animate-spin",
                                            )}
                                            aria-hidden
                                        />
                                        Buscar actualizaciones
                                    </Button>
                                </div>
                            )}
                    </div>
                    <MetaRow label="Licencia" icon={Scale}>
                        <span className="text-sm font-medium" translate="no">
                            AGPL-3.0
                        </span>
                    </MetaRow>
                    <MetaRow label="Autor" icon={User}>
                        <span className="text-sm font-medium" translate="no">
                            johanruizb
                        </span>
                    </MetaRow>

                    <div className="flex flex-wrap gap-2 pt-1">
                        <LinkButton
                            icon={Github}
                            onClick={() => openLink(REPO_URL)}
                        >
                            Repositorio
                        </LinkButton>
                        <LinkButton
                            icon={Bug}
                            onClick={() => openLink(ISSUES_URL)}
                        >
                            Issues
                        </LinkButton>
                        <LinkButton
                            icon={Package}
                            onClick={() => openLink(RELEASES_URL)}
                        >
                            Releases
                        </LinkButton>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    );
}

function MetaRow({
    label,
    icon: Icon,
    children,
}: {
    label: string;
    icon: typeof Info;
    children: ReactNode;
}) {
    return (
        <div className="flex items-center justify-between gap-4">
            <span className="flex items-center gap-2 text-sm text-muted-foreground">
                <Icon className="h-4 w-4" aria-hidden />
                {label}
            </span>
            {children}
        </div>
    );
}

function LinkButton({
    icon: Icon,
    onClick,
    children,
}: {
    icon: typeof Info;
    onClick: () => void;
    children: ReactNode;
}) {
    return (
        <Button
            variant="outline"
            size="sm"
            onClick={onClick}
            className="gap-1.5"
        >
            <Icon className="h-4 w-4" aria-hidden />
            {children}
            <ExternalLink className="h-3 w-3 opacity-60" aria-hidden />
        </Button>
    );
}

// Default export (decisión #11 / T18): App.tsx (T12) lo importa como
// `import AboutDialog from "@/screens/About"`. Se mantiene además el named
// export para compatibilidad con el estilo del resto de dialogs del proyecto.
export default AboutDialog;
