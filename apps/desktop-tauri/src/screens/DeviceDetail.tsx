import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ProfileSelect, newScanId } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
  cancelScan,
  exportServices,
  getDevice,
  onScanCancelled,
  onScanFinished,
  onScanHeartbeat,
  onScanProgress,
  scanPorts,
  type DeviceDetailDto,
  type ScanProgress,
  type UnlistenFn,
} from "@/lib/tauri";

export function DeviceDetail() {
  const { ip = "" } = useParams();
  const navigate = useNavigate();
  const { toast } = useToast();
  const [detail, setDetail] = useState<DeviceDetailDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [profile, setProfile] = useState("normal");

  const [scanning, setScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [scanTimeout, setScanTimeout] = useState(0);
  const [scanId, setScanId] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    getDevice(decodeURIComponent(ip))
      .then((d) => {
        if (alive) {
          setDetail(d);
          setError(null);
        }
      })
      .catch((e) => {
        if (alive) setError(String(e));
      });
    return () => {
      alive = false;
    };
  }, [ip]);

  // Listeners de progreso/heartbeat/cancel/finish sólo mientras escanea.
  useEffect(() => {
    if (!scanning || !scanId) return;
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    onScanProgress((p) => {
      // `scan:progress` no incluye scan_id en el payload (struct crudo de
      // mylan-scanner); se asume el scan activo de esta pantalla.
      setProgress(p);
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
        toast("Escaneo de puertos completado.", "success");
        // Refresca servicios.
        getDevice(decodeURIComponent(ip)).then(setDetail).catch(() => {});
      }
    }).then((u) => unlisteners.push(u));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [scanning, scanId]);

  async function handleScanPorts() {
    const id = newScanId();
    setScanId(id);
    setScanning(true);
    setProgress(null);
    setElapsed(0);
    try {
      await scanPorts(decodeURIComponent(ip), profile, id);
    } catch (e) {
      setScanning(false);
      setScanId(null);
      toast(`Error: ${e}`, "error");
    }
  }

  async function handleCancel() {
    if (scanId) {
      try {
        await cancelScan(scanId);
      } catch (e) {
        toast(`Error cancelando: ${e}`, "error");
      }
    }
  }

  async function handleExport(format: string) {
    try {
      const path = await exportServices(format);
      toast(`Servicios exportados a: ${path}`, "success");
    } catch (e) {
      toast(`Error exportando: ${e}`, "error");
    }
  }

  const pct = progress?.percent_done ?? 0;
  const remainMs = scanTimeout > elapsed ? scanTimeout - elapsed : 0;

  if (error) return <p className="text-sm text-red-600">{error}</p>;
  if (!detail) return <p className="text-sm text-muted-foreground">Cargando…</p>;

  const d = detail.device;

  return (
    <div className="flex flex-col gap-4">
      <Button variant="ghost" size="sm" onClick={() => navigate("/devices")}>
        ← Volver
      </Button>

      <Card>
        <CardHeader>
          <CardTitle>{d.primary_ip ?? d.id}</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-2">
          <Field label="MAC" value={d.primary_mac ?? "—"} mono />
          <Field label="Hostname" value={d.hostname ?? d.display_name ?? "—"} />
          <Field label="Vendor" value={d.vendor ?? "—"} />
          <Field label="Tipo" value={d.device_type} />
          <Field label="Confianza" value={d.confidence} />
          <Field label="Último visto" value={d.last_seen_at} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Escaneo de puertos</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="flex flex-wrap items-end gap-3">
            <div className="flex flex-col gap-1">
              <label className="text-xs text-muted-foreground">Perfil</label>
              <ProfileSelect value={profile} onChange={setProfile} className="w-40" />
            </div>
            <Button onClick={handleScanPorts} disabled={scanning}>
              {scanning ? "Escaneando…" : "Escanear puertos"}
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
                  {progress ? `${progress.ports_tested}/${progress.ports_total} puertos · ${pct}%` : "en progreso…"}
                  {progress?.latest_open_port ? ` · último abierto: ${progress.latest_open_port}` : ""}
                </span>
                <span>
                  {Math.round(elapsed / 100) / 10}s / {Math.round(remainMs / 100) / 10}s
                </span>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>Servicios ({detail.services.length})</CardTitle>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => handleExport("csv")}>
              Exportar CSV
            </Button>
            <Button variant="outline" size="sm" onClick={() => handleExport("json")}>
              Exportar JSON
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Protocolo</TableHead>
                <TableHead>Puerto</TableHead>
                <TableHead>Servicio</TableHead>
                <TableHead>Producto</TableHead>
                <TableHead>Versión</TableHead>
                <TableHead>Estado</TableHead>
                <TableHead>Banner</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {detail.services.length === 0 && (
                <TableRow>
                  <TableCell colSpan={7} className="text-muted-foreground">
                    Sin servicios. Ejecuta un escaneo de puertos.
                  </TableCell>
                </TableRow>
              )}
              {detail.services.map((s) => (
                <TableRow key={s.id}>
                  <TableCell>{s.protocol}</TableCell>
                  <TableCell>{s.port}</TableCell>
                  <TableCell>{s.service_name ?? "—"}</TableCell>
                  <TableCell>{s.product ?? "—"}</TableCell>
                  <TableCell>{s.version ?? "—"}</TableCell>
                  <TableCell>{s.state}</TableCell>
                  <TableCell className="font-mono text-xs">{s.banner ?? "—"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className={`font-medium ${mono ? "font-mono text-xs" : ""}`}>{value}</span>
    </div>
  );
}