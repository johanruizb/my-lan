import { useEffect, useState, useCallback } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
    ShieldCheck,
    ShieldAlert,
    ShieldX,
    RefreshCw,
    AlertTriangle,
    Info,
    Loader2,
} from "lucide-react";
import {
    getSecurityFindings,
    getSecurityScore,
    runSecurityAudit,
    onScanFinished,
    type SecurityFinding,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface SecurityScoreCardProps {
    className?: string;
}

export function SecurityScoreCard({ className }: SecurityScoreCardProps) {
    const [score, setScore] = useState<number | null>(null);
    const [findings, setFindings] = useState<SecurityFinding[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [auditing, setAuditing] = useState(false);

    const loadData = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            if (
                typeof getSecurityScore !== "function" ||
                typeof getSecurityFindings !== "function"
            ) {
                throw new Error("API de seguridad no disponible");
            }
            const [currentScore, currentFindings] = await Promise.all([
                getSecurityScore(),
                getSecurityFindings(),
            ]);
            setScore(currentScore);
            setFindings(currentFindings);
        } catch (e) {
            // No inventar un 100: sin datos el score es desconocido.
            setScore(null);
            setFindings([]);
            setError(e instanceof Error ? e.message : String(e));
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadData();
    }, [loadData]);

    // Recargar automáticamente cuando finaliza cualquier escaneo
    useEffect(() => {
        let unlisten: (() => void) | undefined;
        if (typeof onScanFinished === "function") {
            onScanFinished(() => {
                loadData();
            })
                .then((fn) => {
                    unlisten = fn;
                })
                .catch(() => {});
        }
        return () => {
            if (unlisten) unlisten();
        };
    }, [loadData]);

    const handleReaudit = async () => {
        try {
            setAuditing(true);
            const report = await runSecurityAudit();
            setScore(report.score);
            setFindings(report.findings);
        } catch (e) {
            console.error("Error al re-auditar seguridad:", e);
        } finally {
            setAuditing(false);
        }
    };

    const openFindings = findings.filter((f) => f.status === "open");
    const criticalCount = openFindings.filter((f) => f.severity === "critical").length;
    const highCount = openFindings.filter((f) => f.severity === "high").length;
    const mediumCount = openFindings.filter((f) => f.severity === "medium").length;
    const lowCount = openFindings.filter((f) => f.severity === "low").length;

    const displayScore = score ?? 0;
    const hasScore = score !== null;
    const isWarning = hasScore && displayScore >= 60 && displayScore < 85;
    const isDanger = hasScore && displayScore < 60;

    const ScoreIcon = !hasScore ? ShieldAlert : isDanger ? ShieldX : isWarning ? ShieldAlert : ShieldCheck;
    const scoreColorClass = !hasScore
        ? "text-muted-foreground border-border bg-muted/40"
        : isDanger
          ? "text-destructive-strong border-destructive/30 bg-destructive/10"
          : isWarning
            ? "text-warning-strong border-warning/30 bg-warning/10"
            : "text-success-strong border-success/30 bg-success/10";

    const scoreTitle = !hasScore
        ? "Postura de seguridad no disponible"
        : isDanger
          ? "Riesgo crítico detectado"
          : isWarning
            ? "Riesgo moderado en la red"
            : "Postura de seguridad sólida";

    return (
        <Card className={cn("overflow-hidden", className)}>
            <CardContent className="p-4 sm:p-5">
                <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div className="flex items-center gap-4 min-w-0">
                        {/* Gauge / Badge del Score */}
                        <div
                            className={cn(
                                "flex h-14 w-14 shrink-0 flex-col items-center justify-center rounded-xl border text-center transition-colors",
                                scoreColorClass,
                            )}
                            title={
                                hasScore
                                    ? `Score de seguridad: ${displayScore}/100`
                                    : "Score de seguridad no disponible"
                            }
                        >
                            <span className="text-xl font-bold font-mono leading-none tracking-tight">
                                {hasScore ? displayScore : "—"}
                            </span>
                            <span className="text-[11px] font-medium opacity-80 uppercase tracking-wider">
                                / 100
                            </span>
                        </div>

                        <div className="flex flex-col min-w-0 gap-1">
                            <div className="flex items-center gap-2">
                                <ScoreIcon className={cn("h-4 w-4 shrink-0", scoreColorClass.split(" ")[0])} />
                                <h3 className="text-sm font-semibold text-foreground truncate">
                                    {scoreTitle}
                                </h3>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                {!hasScore
                                    ? error ?? "Cargando análisis de seguridad…"
                                    : openFindings.length === 0
                                      ? "No se han detectado servicios vulnerables ni configuraciones de riesgo en la red."
                                      : `${openFindings.length} hallazgo(s) activo(s) requieren atención o mitigación.`}
                            </p>
                        </div>
                    </div>

                    {/* Desglose de severidades y botón de re-auditar */}
                    <div className="flex flex-wrap items-center justify-between sm:justify-end gap-2.5 pt-2 sm:pt-0 border-t border-border sm:border-t-0">
                        <div className="flex items-center gap-1.5 flex-wrap">
                            {criticalCount > 0 && (
                                <Badge
                                    variant="destructive"
                                    className="gap-1 h-6 px-2 font-semibold"
                                    title={`${criticalCount} hallazgos críticos (-25 pts c/u)`}
                                >
                                    <AlertTriangle className="h-3 w-3" />
                                    {criticalCount} Crítico
                                </Badge>
                            )}
                            {highCount > 0 && (
                                <Badge
                                    variant="warning"
                                    className="gap-1 h-6 px-2 font-semibold"
                                    title={`${highCount} hallazgos altos (-15 pts c/u)`}
                                >
                                    {highCount} Alto
                                </Badge>
                            )}
                            {mediumCount > 0 && (
                                <Badge
                                    variant="outline"
                                    className="gap-1 h-6 px-2 font-semibold"
                                    title={`${mediumCount} hallazgos medios (-5 pts c/u)`}
                                >
                                    {mediumCount} Medio
                                </Badge>
                            )}
                            {lowCount > 0 && (
                                <Badge
                                    variant="secondary"
                                    className="gap-1 h-6 px-2 font-semibold"
                                    title={`${lowCount} hallazgos informativos (-1 pt c/u)`}
                                >
                                    <Info className="h-3 w-3" />
                                    {lowCount} Bajo
                                </Badge>
                            )}
                        </div>

                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleReaudit}
                            disabled={auditing || loading}
                            className="h-8 gap-1.5 text-xs font-medium"
                            title="Re-evaluar el inventario y servicios con el catálogo de reglas pasivas"
                        >
                            {auditing ? (
                                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                            ) : (
                                <RefreshCw className="h-3.5 w-3.5" />
                            )}
                            Re-auditar
                        </Button>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
