import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
    Info,
    Tag,
    Scale,
    User,
    Github,
    Bug,
    Package,
    ExternalLink,
} from "lucide-react";
import { getAppVersion } from "@/lib/tauri";

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
                    <DialogTitle>Acerca de MyLAN</DialogTitle>
                </div>
                <DialogDescription>
                    Escáner de red local de escritorio.
                </DialogDescription>
                <div className="flex flex-col gap-4">
                    <MetaRow label="Versión" icon={Tag}>
                        <Badge variant="secondary">v{version || "…"}</Badge>
                    </MetaRow>
                    <MetaRow label="Licencia" icon={Scale}>
                        <span className="text-sm font-medium">AGPL-3.0</span>
                    </MetaRow>
                    <MetaRow label="Autor" icon={User}>
                        <span className="text-sm font-medium">johanruizb</span>
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
