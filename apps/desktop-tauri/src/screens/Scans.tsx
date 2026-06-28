import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
  cancelScan,
  onScanCancelled,
  onScanFinished,
  onScanHeartbeat,
  onScanProgress,
  scanPorts,
  type ScanProgress,
  type Service,
  type UnlistenFn,
} from "@/lib/tauri";

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

  useEffect(() => {
    if (!scanning || !scanId) return;
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    onScanProgress((p) => {
      // `scan:progress` no incluye scan_id (struct crudo de mylan-scanner).
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
      }
    }).then((u) => unlisteners.push(u));
    onScanFinished((f) => {
      if (f.scan_id === scanId && !cancelled) {
        setScanning(false);
        setScanId(null);
        toast("Escaneo completado.", "success");
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
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Escaneo de puertos</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="flex flex-wrap items-end gap-3">
            <div className="flex flex-col gap-1">
              <label className="text-xs text-muted-foreground">IP del host</label>
              <Input
                placeholder="192.168.1.10"
                value={ip}
                onChange={(e) => setIp(e.target.value)}
                className="w-48"
                disabled={scanning}
              />
            </div>
            <div className="flex flex-col gap-1">
              <label className="text-xs text-muted-foreground">Perfil</label>
              <ProfileSelect value={profile} onChange={setProfile} className="w-40" />
            </div>
            <Button onClick={handleStart} disabled={scanning}>
              {scanning ? "Escaneando…" : "Iniciar"}
            </Button>
            {scanning && (
              <Button variant="destructive" onClick={handleCancel}>
                Cancelar
              </Button>
            )}
          </div>

          {scanning && (
            <div className="flex flex-col gap-2">
              <Progress value={pct} />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>
                  {progress
                    ? `${progress.ports_tested}/${progress.ports_total} · ${pct}%`
                    : "en progreso…"}
                </span>
                <span>
                  {Math.round(elapsed / 100) / 10}s /{" "}
                  {Math.round(remainMs / 100) / 10}s
                </span>
              </div>
            </div>
          )}

          <div>
            <div className="text-sm font-medium">Puertos abiertos ({openPorts.length})</div>
            <ul className="mt-2 flex flex-wrap gap-2">
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
    </div>
  );
}