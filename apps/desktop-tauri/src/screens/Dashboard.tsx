import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProfileSelect } from "@/components/profile-select";
import { useToast } from "@/components/ui/toast";
import {
  detectInterface,
  listDevices,
  runDiscovery,
  type Device,
  type LanInterfaceDto,
  type ScanOutcomeDto,
} from "@/lib/tauri";

export function Dashboard() {
  const { toast } = useToast();
  const [iface, setIface] = useState<LanInterfaceDto | null>(null);
  const [devices, setDevices] = useState<Device[]>([]);
  const [profile, setProfile] = useState("normal");
  const [scanning, setScanning] = useState(false);
  const [lastOutcome, setLastOutcome] = useState<ScanOutcomeDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    try {
      const [ifaceRes, devicesRes] = await Promise.all([
        detectInterface(),
        listDevices().catch(() => [] as Device[]),
      ]);
      setIface(ifaceRes);
      setDevices(devicesRes);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  async function handleScan() {
    setScanning(true);
    setError(null);
    try {
      const outcome = await runDiscovery(profile);
      setLastOutcome(outcome);
      const devicesRes = await listDevices().catch(() => [] as Device[]);
      setDevices(devicesRes);
      toast(
        `Escaneo completado: ${outcome.hosts_alive} hosts vivos, ${outcome.hosts_new} nuevos.`,
        "success",
      );
    } catch (e) {
      const msg = String(e);
      setError(msg);
      toast(`Error: ${msg}`, "error");
    } finally {
      setScanning(false);
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Red activa</CardTitle>
          <CardDescription>
            Interfaz detectada automáticamente como default route.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-2">
          <Info label="Interfaz" value={iface?.name ?? "—"} />
          <Info label="IP / CIDR" value={iface ? `${iface.ip}/${iface.prefix_len}` : "—"} />
          <Info label="Gateway" value={iface?.gateway_ip ?? "—"} />
          <Info label="MAC" value={iface?.mac ?? "—"} />
          <Info
            label="DNS"
            value={iface ? iface.dns_servers.join(", ") || "—" : "—"}
          />
        </CardContent>
      </Card>

      <div className="grid gap-4 sm:grid-cols-3">
        <Stat label="Dispositivos" value={devices.length} />
        <Stat label="Hosts vivos (último scan)" value={lastOutcome?.hosts_alive ?? 0} />
        <Stat label="Nuevos (último scan)" value={lastOutcome?.hosts_new ?? 0} />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Escanear ahora</CardTitle>
          <CardDescription>
            Descubre los hosts de tu LAN con el perfil seleccionado.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap items-end gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-muted-foreground">Perfil</label>
            <ProfileSelect value={profile} onChange={setProfile} className="w-40" />
          </div>
          <Button onClick={handleScan} disabled={scanning}>
            {scanning ? "Escaneando…" : "Escanear ahora"}
          </Button>
        </CardContent>
      </Card>

      {error && (
        <p className="text-sm text-red-600">Error: {error}</p>
      )}
    </div>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <Card>
      <CardContent className="p-4">
        <div className="text-xs text-muted-foreground">{label}</div>
        <div className="mt-1 text-2xl font-bold">{value}</div>
      </CardContent>
    </Card>
  );
}