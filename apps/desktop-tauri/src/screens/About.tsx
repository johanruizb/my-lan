import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
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

const REPO_URL = "https://github.com/johanruizb/my-lan";
const ISSUES_URL = `${REPO_URL}/issues`;
const RELEASES_URL = `${REPO_URL}/releases`;

export function About() {
    const [version, setVersion] = useState("");

    useEffect(() => {
        getAppVersion()
            .then(setVersion)
            .catch(() => {});
    }, []);

    async function open(url: string) {
        try {
            await openUrl(url);
        } catch {
            // Opener no disponible: no-op (no navegar dentro del webview).
        }
    }

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Info className="h-5 w-5 text-primary" aria-hidden />
                        Acerca de MyLAN
                    </CardTitle>
                    <CardDescription>
                        Escáner de red local de escritorio.
                    </CardDescription>
                </CardHeader>
                <CardContent className="flex flex-col gap-4">
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
                            onClick={() => open(REPO_URL)}
                        >
                            Repositorio
                        </LinkButton>
                        <LinkButton icon={Bug} onClick={() => open(ISSUES_URL)}>
                            Issues
                        </LinkButton>
                        <LinkButton
                            icon={Package}
                            onClick={() => open(RELEASES_URL)}
                        >
                            Releases
                        </LinkButton>
                    </div>
                </CardContent>
            </Card>
        </div>
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
