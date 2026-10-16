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
import { Switch } from "@/components/ui/switch";
import { useCensorship } from "./censorship-provider";
import { useWorkspace } from "./workspace-provider";
import { useToast } from "@/components/ui/toast";
import { downloadExecutiveReport } from "./executive-report-pdf";
import {
    getAppVersion,
    getNetworkHealthSamples,
    getRecentSpeedTests,
    getSecurityFindings,
    getSecurityScore,
    listDevices,
    type Device,
    type NetworkHealthSample,
    type SecurityFinding,
    type SpeedTestResult,
} from "@/lib/tauri";
import { useNetworkName } from "@/lib/use-network-name";
import { FileText, Loader2, ShieldCheck, ShieldAlert } from "lucide-react";

interface ExportReportModalProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function ExportReportModal({ open, onOpenChange }: ExportReportModalProps) {
    const { censorshipEnabled } = useCensorship();
    const { activeWorkspace } = useWorkspace();
    const { name: networkName, cidr: networkCidr } = useNetworkName();
    const { toast } = useToast();

    // Estado local del toggle de censura (premarcado según el modo global, AC-7)
    const [censorExport, setCensorExport] = useState<boolean>(censorshipEnabled);
    const [generating, setGenerating] = useState<boolean>(false);

    async function handleExport() {
        setGenerating(true);
        try {
            const [
                devices,
                findings,
                securityScore,
                speedTests,
                healthSamples,
                version,
            ] = await Promise.all([
                listDevices(activeWorkspace.id).catch(() => [] as Device[]),
                getSecurityFindings(networkCidr).catch(
                    () => [] as SecurityFinding[],
                ),
                getSecurityScore(networkCidr).catch(() => 100),
                getRecentSpeedTests({ networkId: networkCidr, limit: 1 }).catch(
                    () => [] as SpeedTestResult[],
                ),
                getNetworkHealthSamples(networkCidr, 1).catch(
                    () => [] as NetworkHealthSample[],
                ),
                getAppVersion().catch(() => "1.0"),
            ]);

            await downloadExecutiveReport({
                workspace: activeWorkspace,
                networkName: networkName || "Red Local",
                networkCidr: networkCidr || "192.168.1.0/24",
                generatedAt: new Date().toLocaleString(),
                censorshipEnabled: censorExport,
                devices,
                securityScore,
                findings,
                speedTest: speedTests[0] ?? null,
                healthSample: healthSamples[0] ?? null,
                version,
            });

            toast("Informe ejecutivo descargado correctamente", "success");
            onOpenChange(false);
        } catch (e) {
            toast(`Error al generar el informe: ${e}`, "error");
        } finally {
            setGenerating(false);
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[480px]">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <FileText className="h-5 w-5 text-primary" />
                        Exportar Informe Ejecutivo en PDF
                    </DialogTitle>
                    <DialogDescription>
                        Genera un documento PDF formal con el inventario
                        técnico, matriz de riesgos de seguridad y métricas de salud
                        de red del workspace activo.
                    </DialogDescription>
                </DialogHeader>

                <div className="flex flex-col gap-4 py-3">
                    {/* Tarjeta de resumen de destino */}
                    <div className="flex flex-col gap-1.5 rounded-lg border bg-muted/30 p-3">
                        <span className="text-xs text-muted-foreground font-semibold">
                            Espacio de Trabajo a Exportar
                        </span>
                        <div className="flex items-center gap-2">
                            <span
                                className="h-3.5 w-3.5 rounded-full shrink-0"
                                style={{ backgroundColor: activeWorkspace.color }}
                            />
                            <span className="text-sm font-bold">
                                {activeWorkspace.name}
                            </span>
                            <span className="text-xs text-muted-foreground">
                                • {networkName || networkCidr || "Red Local"}
                            </span>
                        </div>
                    </div>

                    {/* Toggle Modo Censura */}
                    <div className="flex items-start justify-between gap-3 rounded-lg border p-3 hover:bg-muted/10 transition-colors">
                        <div className="flex flex-col gap-1">
                            <div className="flex items-center gap-1.5 font-medium text-sm">
                                {censorExport ? (
                                    <ShieldCheck className="h-4 w-4 text-emerald-500" />
                                ) : (
                                    <ShieldAlert className="h-4 w-4 text-amber-500" />
                                )}
                                <span>Exportar con Modo Censura</span>
                            </div>
                            <span className="text-xs text-muted-foreground leading-relaxed">
                                Enmascara rigurosamente direcciones IP, MACs,
                                SSIDs y hostnames para distribución y auditorías
                                externas seguras.
                            </span>
                        </div>
                        <Switch
                            checked={censorExport}
                            onCheckedChange={setCensorExport}
                            aria-label="Exportar con Modo Censura"
                        />
                    </div>
                </div>

                <DialogFooter className="gap-2 sm:gap-0">
                    <Button
                        variant="outline"
                        onClick={() => onOpenChange(false)}
                        disabled={generating}
                    >
                        Cancelar
                    </Button>
                    <Button
                        onClick={handleExport}
                        disabled={generating}
                        className="gap-2"
                    >
                        {generating ? (
                            <>
                                <Loader2 className="h-4 w-4 animate-spin" />
                                Compilando PDF...
                            </>
                        ) : (
                            <>
                                <FileText className="h-4 w-4" />
                                Descargar Informe PDF
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
