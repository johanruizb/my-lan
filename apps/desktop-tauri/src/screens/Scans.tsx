import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { EmptyState } from "@/components/empty-state";
import { useToast } from "@/components/ui/toast";
import {
  Radar,
  Loader2,
  Play,
  Square,
  History,
  ShieldAlert,
  CircleCheck,
  CircleX,
} from "lucide-react";
import {
  cancelScan,
  listScans,
  onScanCancelled,
  onScanFinished,
  onScanHeartbeat,
  onScanProgress,
  scanPorts,
  type ScanProgress,
  type ScanSummaryDto,
  type Service,
  type UnlistenFn,
} from "@/lib/tauri";

function statusBadge(status: string) {
  const s = status.toLowerCase();
  if (s === "completed") return <Badge variant="success" className="gap-1"><CircleCheck className="h-3 w-3" aria-hidden />Completado</Badge>;
  if (s === "failed") return <Badge variant="destructive" className="gap-1"><CircleX className="h-3 w-3" aria-hidden />Fallido</Badge>;
  return <Badge variant="secondary" className="gap-1"><Loader2 className="h-3 w-3 animate-spin" aria-hidden />En curso</Badge>;
}

export function Scans() {
  const { toast } = useToast();
  const [ip, setIp] = useState("");
  const [profile, setProfile] = useState("normal");

  const [scanning, setScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [scanTimeout, setScanTimeout] = useState(0);
  const [openPorts, setOpenPorts] = useState<Service[]>([]);
  const [scanId, setScanId] = useState<string | null>(null);

  const [history, setHistory] = useState<ScanSummaryDto[]>([]);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [historyError, setHistoryError] = useState<string | null>(null);

  async function refreshHistory() {
    setHistoryLoading(true);
    try {
      const rows = await listScans();
      setHistory(rows);
      setHistoryError(null);
    } catch (e) {
      setHistoryError(String(e));
      setHistory([]);
    } finally {
      setHistoryLoading(false);
    }
  }

  useEffect(() => {
    refreshHistory();
  }, []);

  useEffect(() => {
    if (!scanning || !scanId) return;
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    onScanProgress((p) => {
      setProgress(p);
      if (p.latest_open_port) {
        setOpenPorts((prev) =>
          prev.some((s) => s.port === p.latest_open_port)
            ? prev
            : [...prev, { port: p.latest_open_port } as Service],
        );
      }
    }).then((u) => unlisteners.push(u));
    onScanHeartbeat((h) => {
      if (h.scan_id === scanId) {
        setElapsed(h.elapsed_ms);
        setScanTimeout(h.scan_timeout_ms);
      }
    }).then((u) => unlisteners.push(u));
    onScanCancelled((c) => {
      if (c.scan_id === scanId) {
        cancelled = true;
        setScanning(false);
        setScanId(null);
        toast("Escaneo cancelado.", "default");
        refreshHistory();
      }
    }).then((u) => unlisteners.push(u));
    onScanFinished((f) => {
      if (f.scan_id === scanId && !cancelled) {
        setScanning(false);
        setScanId(null);
        toast("Escaneo completado.", "success");
        refreshHistory();
      }
    }).then((u) => unlisteners.push(u));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [scanning, scanId]);

  async function handleStart() {
    if (!ip.trim()) {
      toast("Introduce una IP.", "error");
      return;
    }
    const id = newScanId();
    setScanId(id);
    setScanning(true);
    setProgress(null);
    setElapsed(0);
    setOpenPorts([]);
    try {
      const services = await scanPorts(ip.trim(), profile, id);
      setOpenPorts(services);
    } catch (e) {
      setScanning(false);
      setScanId(null);
      toast(`Error: ${e}`, "error");
    }
  }

  async function handleCancel() {
    if (scanId) await cancelScan(scanId).catch((e) => toast(`Error: ${e}`, "error"));
  }

  const pct = progress?.percent_done ?? 0;
  const remainMs = scanTimeout > elapsed ? scanTimeout - elapsed : 0;

  return (
    <div className="flex flex-col gap-4" aria-busy={scanning || historyLoading}>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Radar className="h-5 w-5 text-primary" aria-hidden />
            Escaneo de puertos
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex flex-col gap-1.5">
              <label htmlFor="scan-ip" className="text-xs text-muted-foreground">
                IP del host
              </label>
              <Input
                id="scan-ip"
                placeholder="192.168.1.10"
                value={ip}
                onChange={(e) => setIp(e.target.value)}
                className="w-48"
                disabled={scanning}
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <label htmlFor="scan-profile" className="text-xs text-muted-foreground">
                Perfil
              </label>
              <ProfileSelect
                value={profile}
                onChange={setProfile}
                className="w-40"
                id="scan-profile"
                disabled={scanning}
              />
            </div>
            <Button onClick={handleStart} disabled={scanning} className="mt-5 gap-1.5">
              {scanning ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                  Escaneando…
                </>
              ) : (
                <>
                  <Play className="h-4 w-4" aria-hidden />
                  Iniciar
                </>
              )}
            </Button>
            {scanning && (
              <Button variant="destructive" onClick={handleCancel} className="mt-5 gap-1.5">
                <Square className="h-4 w-4" aria-hidden />
                Cancelar
              </Button>
            )}
          </div>

          {scanning && (
            // Live region para progreso/heartbeat/cancel (AC-15).
            <div className="flex flex-col gap-2" aria-live="polite" aria-atomic="true">
              <Progress value={pct} indeterminate={pct === 0} />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>
                  {progress
                    ? `${progress.ports_tested}/${progress.ports_total} · ${pct}%`
                    : "en progreso…"}
                </span>
                <span>
                  {Math.round(elapsed / 100) / 10}s / {Math.round(remainMs / 100) / 10}s
                </span>
              </div>
            </div>
          )}

          <div>
            <div className="text-sm font-medium">Puertos abiertos ({openPorts.length})</div>
            <ul className="mt-2 flex flex-wrap gap-2" aria-label="Puertos abiertos">
              {openPorts.length === 0 && (
                <li className="text-xs text-muted-foreground">
                  Sin puertos abiertos detectados aún.
                </li>
              )}
              {openPorts.map((s) => (
                <li
                  key={s.port}
                  className="rounded-md border border-border bg-muted px-2 py-1 text-xs font-mono"
                >
                  {s.port}
                </li>
              ))}
            </ul>
          </div>
        </CardContent>
      </Card>

      {/* Historial de scans (AC-11, vía list_scans IPC). */}
      <section aria-label="Historial de escaneos">
        <Card>
          <CardHeader className="flex-row items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <History className="h-5 w-5 text-primary" aria-hidden />
              Historial ({history.length})
            </CardTitle>
            <Button variant="outline" size="sm" onClick={refreshHistory} disabled={historyLoading}>
              {historyLoading ? "Cargando…" : "Actualizar"}
            </Button>
          </CardHeader>
          <CardContent>
            {historyError && (
              <div role="alert" className="mb-4">
                <EmptyState
                  icon={ShieldAlert}
                  title="No se pudo cargar el historial"
                  description={historyError}
                />
              </div>
            )}

            {!historyError && historyLoading && (
              <div className="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                Cargando historial…
              </div>
            )}

            {!historyError && !historyLoading && history.length === 0 && (
              <EmptyState
                icon={History}
                title="Sin escaneos previos"
                description="Los escaneos de descubrimiento y de puertos aparecerán aquí."
              />
            )}

            {!historyError && !historyLoading && history.length > 0 && (
              <div className="overflow-x-auto rounded-md border border-border">
                <table className="w-full caption-bottom text-sm">
                  <thead className="[&_tr]:border-b">
                    <tr className="border-b">
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Perfil
                      </th>
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Estado
                      </th>
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Iniciado
                      </th>
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Finalizado
                      </th>
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Vivos
                      </th>
                      <th className="h-10 px-3 text-left align-middle font-medium text-muted-foreground">
                        Nuevos
                      </th>
                    </tr>
                  </thead>
                  <tbody className="[&_tr:last-child]:border-0">
                    {history.map((s) => (
                      <tr key={s.id} className="border-b transition-colors hover:bg-muted/50">
                        <td className="p-3 align-middle">
                          <Badge variant="outline" className="capitalize">{s.profile}</Badge>
                        </td>
                        <td className="p-3 align-middle">{statusBadge(s.status)}</td>
                        <td className="p-3 align-middle text-xs text-muted-foreground">{s.started_at}</td>
                        <td className="p-3 align-middle text-xs text-muted-foreground">
                          {s.finished_at ?? "—"}
                        </td>
                        <td className="p-3 align-middle font-medium">{s.hosts_alive}</td>
                        <td className="p-3 align-middle font-medium">{s.hosts_new}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}