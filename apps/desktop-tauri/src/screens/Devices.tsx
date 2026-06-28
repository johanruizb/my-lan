import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useToast } from "@/components/ui/toast";
import { exportDevices, listDevices, type Device } from "@/lib/tauri";

export function Devices() {
  const { toast } = useToast();
  const navigate = useNavigate();
  const [devices, setDevices] = useState<Device[]>([]);
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function refresh() {
    setLoading(true);
    try {
      const rows = await listDevices();
      setDevices(rows);
      setError(null);
    } catch (e) {
      setError(String(e));
      setDevices([]);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return devices;
    return devices.filter((d) =>
      [d.primary_ip, d.primary_mac, d.hostname, d.display_name, d.vendor]
        .filter(Boolean)
        .some((v) => (v as string).toLowerCase().includes(q)),
    );
  }, [devices, query]);

  async function handleExport(format: string) {
    try {
      const path = await exportDevices(format);
      toast(`Dispositivos exportados a: ${path}`, "success");
    } catch (e) {
      toast(`Error exportando: ${e}`, "error");
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>Dispositivos ({filtered.length})</CardTitle>
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
          <Input
            placeholder="Buscar por IP, MAC, hostname, vendor…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="mb-4 max-w-md"
          />
          {error && <p className="text-sm text-red-600">{error}</p>}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>IP</TableHead>
                <TableHead>MAC</TableHead>
                <TableHead>Hostname</TableHead>
                <TableHead>Vendor</TableHead>
                <TableHead>Tipo</TableHead>
                <TableHead>Confianza</TableHead>
                <TableHead>Último visto</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loading && (
                <TableRow>
                  <TableCell colSpan={7} className="text-muted-foreground">
                    Cargando…
                  </TableCell>
                </TableRow>
              )}
              {!loading && filtered.length === 0 && (
                <TableRow>
                  <TableCell colSpan={7} className="text-muted-foreground">
                    Sin dispositivos. Ejecuta un escaneo desde el Dashboard.
                  </TableCell>
                </TableRow>
              )}
              {filtered.map((d) => (
                <TableRow
                  key={d.id}
                  className="cursor-pointer"
                  onClick={() => navigate(`/devices/${encodeURIComponent(d.primary_ip ?? d.id)}`)}
                >
                  <TableCell>{d.primary_ip ?? "—"}</TableCell>
                  <TableCell className="font-mono text-xs">{d.primary_mac ?? "—"}</TableCell>
                  <TableCell>{d.hostname ?? d.display_name ?? "—"}</TableCell>
                  <TableCell>{d.vendor ?? "—"}</TableCell>
                  <TableCell>{d.device_type}</TableCell>
                  <TableCell>{d.confidence}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">{d.last_seen_at}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}