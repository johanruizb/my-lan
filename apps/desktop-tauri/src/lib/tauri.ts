// Capa IPC tipada sobre `@tauri-apps/api`.
//
// CONVENCIÓN DE NOMBRES: los DTOs del backend Rust se serializan en **snake_case**
// (los tipos de `mylan-core` —`Device`, `Service`, `ServiceExportRow`,
// `ScanProgress`— derivan `Serialize` sin `rename_all`, y los DTOs propios del
// backend siguen la misma convención para mantener una sola forma en todo el
// frontend). Los tipos aquí espejan exactamente los nombres que emite serde.

import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type { UnlistenFn };

// --- Modelos espejo de mylan-core / mylan-db --------------------------------

export interface Device {
    id: string;
    network_id: string;
    primary_mac: string | null;
    primary_ip: string | null;
    hostname: string | null;
    display_name: string | null;
    vendor: string | null;
    manufacturer: string | null;
    model: string | null;
    device_type: string;
    os_family: string | null;
    confidence: number;
    first_seen_at: string;
    last_seen_at: string;
    is_trusted: boolean;
    is_hidden: boolean;
    is_online: boolean;
    notes: string | null;
}

export interface Service {
    id: string;
    device_id: string;
    protocol: string;
    port: number;
    service_name: string | null;
    product: string | null;
    version: string | null;
    banner: string | null;
    state: string;
    first_seen_at: string;
    last_seen_at: string;
}

export interface ServiceExportRow {
    device_id: string;
    device_ip: string | null;
    display_name: string | null;
    protocol: string;
    port: number;
    service_name: string | null;
    product: string | null;
    version: string | null;
    banner: string | null;
    state: string;
    first_seen_at: string;
    last_seen_at: string;
}

export interface ScanProgress {
    percent_done: number;
    ports_tested: number;
    ports_total: number;
    latest_open_port: number | null;
}

// --- DTOs propios del backend -----------------------------------------------

export interface LanInterfaceDto {
    name: string;
    ip: string;
    prefix_len: number;
    mac: string | null;
    gateway_ip: string | null;
    gateway_mac: string | null;
    dns_servers: string[];
    cidr: string;
    /** SSID Wi-Fi de la interfaz activa, `null` si es cableada o no detectable. */
    ssid: string | null;
}

// Nombre humano de una red persistido por CIDR. `source` distingue el origen
// (`"auto"` = SSID/CIDR detectado, `"user"` = etiqueta editada por el usuario)
// para que un re-escaneo no sobrescriba un nombre del usuario (override gana).
export interface NetworkNameDto {
    name: string;
    source: string;
}

export interface DeviceDetailDto {
    device: Device;
    services: Service[];
}

export interface ScanOutcomeDto {
    scan_id: string;
    network_id: string;
    hosts_alive: number;
    hosts_new: number;
    duration_ms: number;
}

// Resumen de un escaneo para el historial (AC-17 IPC `list_scans`).
// `scan_type` ("discovery" | "ports") distingue el camino; `target_ip` es la IP
// sondeada en un scan de puertos (null en descubrimiento); `open_ports` es el
// conteo de puertos abiertos (0 para descubrimiento). ADR-0001 #23.
export interface ScanSummaryDto {
    id: string;
    scan_type: string;
    target_ip: string | null;
    profile: string;
    status: string;
    started_at: string;
    finished_at: string | null;
    hosts_alive: number;
    hosts_new: number;
    open_ports: number;
}

export interface ServiceFiltersDto {
    device_id?: string | null;
    port?: number | null;
    protocol?: string | null;
    service?: string | null;
}

export interface ScanHeartbeat {
    scan_id: string;
    elapsed_ms: number;
    scan_timeout_ms: number;
}

export interface ScanCancelled {
    scan_id: string;
}

export interface ScanFinished {
    scan_id: string;
}

export interface ScanStarted {
    scan_id: string;
    ip: string | null;
    profile: string;
}

// Progreso de descubrimiento: IPs barridas / total del CIDR (AC-3).
export interface DiscoveryProgress {
    scan_id: string;
    swept: number;
    total: number;
}

// Host descubierto en vivo: el `Device` ya enriquecido tal cual quedó en la BD.
export interface ScanDevice {
    scan_id: string;
    device: Device;
}

export interface Settings {
    db_path: string;
    default_profile: string;
    /** Tema de la UI: `"light"` | `"dark"` (AC-3). */
    theme: string;
    /** Modo censura: enmascara identificadores en UI y exports (AC-1/AC-2). */
    censorship_enabled: boolean;
    /** Búsqueda automática de actualizaciones al arrancar la app. */
    check_updates_on_startup?: boolean;
    /** Monitoreo continuo en segundo plano activo. */
    background_monitoring_enabled?: boolean;
    /** Intervalo de escaneo continuo en segundos (1m, 2m, 5m, 10m, 15m, 30m). */
    monitoring_interval_secs?: number;
    /** Minimizar a la bandeja del sistema en lugar de cerrar la app. */
    minimize_to_tray?: boolean;
    /** Autoarranque con el sistema operativo al iniciar sesión. */
    autostart_enabled?: boolean;
    /** Notificaciones nativas de escritorio activas. */
    notifications_enabled?: boolean;
    /** Notificar cuando se detecten dispositivos no confiables. */
    notify_untrusted_devices?: boolean;
    /** Notificar cambios de IP o MAC en dispositivos confiables. */
    notify_trusted_device_tampered?: boolean;
    /** Notificar caída o recuperación de Internet y Gateway. */
    notify_gateway_down?: boolean;
    /** Notificar puertos críticos abiertos recién detectados. */
    notify_critical_ports?: boolean;
    /** ID del workspace activo (Pilar 6 / Fase 10). */
    active_workspace_id?: string;
}

/** Estado en vivo del monitoreo continuo en segundo plano. */
export interface MonitoringStatusDto {
    enabled: boolean;
    paused: boolean;
    is_scanning: boolean;
    interval_secs: number;
    last_scan_at: string | null;
    next_scan_in_secs: number | null;
    current_network: string | null;
}

// --- Wrappers tipados de invoke ---------------------------------------------

export const detectInterface = (interfaceOverride?: string) =>
    invoke<LanInterfaceDto>("detect_interface_cmd", {
        interface: interfaceOverride ?? null,
    });

export const listDevices = (workspaceId?: string) =>
    invoke<Device[]>(
        "list_devices_cmd",
        workspaceId !== undefined ? { workspaceId } : {},
    );

export const getDevice = (ip: string) =>
    invoke<DeviceDetailDto>("get_device_cmd", { ip });

// Edición parcial de un dispositivo por `id` (UUID `string`, NO number).
// Los campos `null` significan "no tocar" (UPDATE parcial en backend).
export const updateDevice = (
    id: string,
    fields: { displayName?: string; isTrusted?: boolean; notes?: string },
) =>
    invoke<Device>("update_device_cmd", {
        id,
        displayName: fields.displayName ?? null,
        isTrusted: fields.isTrusted ?? null,
        notes: fields.notes ?? null,
    });

export const runDiscovery = (
    profile: string,
    scanId: string,
    interfaceOverride?: string,
) =>
    invoke<ScanOutcomeDto>("run_discovery_cmd", {
        profile,
        interface: interfaceOverride ?? null,
        scanId,
    });

export const scanPorts = (ip: string, profile: string, scanId: string) =>
    invoke<Service[]>("scan_ports_cmd", { ip, profile, scanId });

export const cancelScan = (scanId: string) =>
    invoke<boolean>("cancel_scan_cmd", { scanId });

// Notificación OS nativa al terminar un escaneo de puertos cuando la ventana
// NO está enfocada (AC-4/#24). El frontend comprueba `document.hidden` antes
// de invocar; el comando Rust emite la notificación vía
// `tauri-plugin-notification`. Errores silenciosos: el caller hace fallback al
// toast existente sin mostrar error al usuario.
export const notifyScanFinished = (title: string, body: string) =>
    invoke<void>("notify_scan_finished_cmd", { title, body });

export const listServices = (filters: ServiceFiltersDto = {}) =>
    invoke<ServiceExportRow[]>("list_services_cmd", { filters });

export const listScans = () => invoke<ScanSummaryDto[]>("list_scans_cmd");

export const exportDevices = (format: string, outputPath?: string) =>
    invoke<string>("export_devices_cmd", {
        format,
        outputPath: outputPath ?? null,
    });

export const exportServices = (format: string, outputPath?: string) =>
    invoke<string>("export_services_cmd", {
        format,
        outputPath: outputPath ?? null,
    });

export const dbPath = () => invoke<string>("db_path_cmd");

export const getSettings = () => invoke<Settings>("get_settings_cmd");

export const setSettings = (settings: Settings) =>
    invoke<void>("set_settings_cmd", { settings });

// Nombre de la red por CIDR (`networkId` == `Network.id` == CIDR). El backend
// aplica la precedencia de override en `set`: una etiqueta de usuario no se
// pisa por la auto-detección posterior del SSID.
export const getNetworkName = (networkId: string) =>
    invoke<NetworkNameDto>("get_network_name_cmd", { networkId });

export const setNetworkName = (networkId: string, label: string) =>
    invoke<void>("set_network_name_cmd", { networkId, label });

// Versión unificada de la app, leída en runtime desde `tauri.conf.json`
// (misma fuente para el pie de la sidebar y la pantalla /about).
export const getAppVersion = () => getVersion();

// --- Wrappers tipados de listen ---------------------------------------------

export function onScanProgress(
    cb: (p: ScanProgress) => void,
): Promise<UnlistenFn> {
    return listen<ScanProgress>("scan:progress", (e) => cb(e.payload));
}

export function onScanHeartbeat(
    cb: (h: ScanHeartbeat) => void,
): Promise<UnlistenFn> {
    return listen<ScanHeartbeat>("scan:heartbeat", (e) => cb(e.payload));
}

export function onScanCancelled(
    cb: (c: ScanCancelled) => void,
): Promise<UnlistenFn> {
    return listen<ScanCancelled>("scan:cancelled", (e) => cb(e.payload));
}

export function onScanFinished(
    cb: (f: ScanFinished) => void,
): Promise<UnlistenFn> {
    return listen<ScanFinished>("scan:finished", (e) => cb(e.payload));
}

export function onScanStarted(
    cb: (s: ScanStarted) => void,
): Promise<UnlistenFn> {
    return listen<ScanStarted>("scan:started", (e) => cb(e.payload));
}

export function onDiscoveryProgress(
    cb: (p: DiscoveryProgress) => void,
): Promise<UnlistenFn> {
    return listen<DiscoveryProgress>("scan:discovery_progress", (e) =>
        cb(e.payload),
    );
}

export function onScanDevice(cb: (d: ScanDevice) => void): Promise<UnlistenFn> {
    return listen<ScanDevice>("scan:device", (e) => cb(e.payload));
}

export function onDbImported(cb: () => void): Promise<UnlistenFn> {
    return listen("db:imported", () => cb());
}

// Evento `censorship:fresh`: emitido por `lib.rs` cuando el archivo de ajustes
// no existía antes de esta versión (install nuevo). Mueve el `listen()` al seam
// para que `vi.mock("@/lib/tauri")` cubra el 100% de componentes (AC-14).
// La lógica de timing (race listener/timeout, cleanup) queda en el consumidor.
export function onCensorshipFresh(cb: () => void): Promise<UnlistenFn> {
    return listen("censorship:fresh", () => cb());
}

// --- Monitoreo continuo 24/7 (Fase 5: Desktop Watch) -------------------------

export const getMonitoringStatus = () =>
    invoke<MonitoringStatusDto>("get_monitoring_status_cmd");

export const toggleMonitoringPause = () =>
    invoke<boolean>("toggle_monitoring_pause_cmd");

export const triggerBackgroundScan = () =>
    invoke<void>("trigger_background_scan_cmd");

export function onMonitoringStatus(
    cb: (status: MonitoringStatusDto) => void,
): Promise<UnlistenFn> {
    return listen<MonitoringStatusDto>("monitoring:status", (e) => cb(e.payload));
}

// --- Alertas y notificaciones nativas de escritorio (Fase 6: Desktop Alerts) -

export type AlertCategory =
    | "device_new_untrusted"
    | "device_identity_tampered"
    | "gateway_down"
    | "gateway_recovered"
    | "port_critical_opened";

export interface AlertItem {
    category: AlertCategory;
    title: string;
    body: string;
    device_id: string | null;
}

export interface AlertSummary {
    total: number;
    title: string;
    body: string;
    alerts: AlertItem[];
}

export function onAlertSingle(cb: (alert: AlertItem) => void): Promise<UnlistenFn> {
    return listen<AlertItem>("alert:single", (e) => cb(e.payload));
}

export function onAlertSummary(cb: (summary: AlertSummary) => void): Promise<UnlistenFn> {
    return listen<AlertSummary>("alert:summary", (e) => cb(e.payload));
}

// --- Suite de Herramientas de Red (Fase 7: Network Tools) --------------------

export interface PingPacket {
    seq: number;
    target: string;
    reachable: boolean;
    latency_ms: number | null;
    jitter_ms: number;
    method: "icmp" | "tcp_connect";
}

export interface PingStreamSummary {
    target: string;
    transmitted: number;
    received: number;
    packet_loss_percent: number;
    min_rtt_ms: number | null;
    avg_rtt_ms: number | null;
    max_rtt_ms: number | null;
    jitter_ms: number;
}

export interface TraceHop {
    hop_number: number;
    ip: string | null;
    hostname: string | null;
    latency_ms: number | null;
    state: string;
}

export interface TraceHopEvent {
    session_id: string;
    hop: TraceHop;
}

export interface TraceDoneEvent {
    session_id: string;
    hops: TraceHop[];
}

export interface DnsRecord {
    name: string;
    record_type: string;
    value: string;
    ttl: number;
}

export interface PortCheckResult {
    target: string;
    port: number;
    state: string; // "open" | "closed" | "filtered"
    latency_ms: number | null;
    banner: string | null;
}

export const startToolPing = (params: {
    target: string;
    sessionId: string;
    intervalMs?: number;
    timeoutMs?: number;
}) =>
    invoke<void>("start_tool_ping_cmd", {
        target: params.target,
        sessionId: params.sessionId,
        intervalMs: params.intervalMs ?? null,
        timeoutMs: params.timeoutMs ?? null,
    });

export const startToolTraceroute = (params: {
    target: string;
    sessionId: string;
    maxHops?: number;
    timeoutMs?: number;
}) =>
    invoke<void>("start_tool_traceroute_cmd", {
        target: params.target,
        sessionId: params.sessionId,
        maxHops: params.maxHops ?? null,
        timeoutMs: params.timeoutMs ?? null,
    });

export const cancelTool = (sessionId: string) =>
    invoke<boolean>("cancel_tool_cmd", { sessionId });

export const dnsLookup = (name: string, recordType: string, provider?: string) =>
    invoke<DnsRecord[]>("dns_lookup_cmd", {
        name,
        recordType,
        provider: provider ?? null,
    });

export const sendWol = (mac: string, broadcastIp?: string) =>
    invoke<void>("send_wol_cmd", {
        mac,
        broadcastIp: broadcastIp ?? null,
    });

export const checkPort = (target: string, port: number, timeoutMs?: number) =>
    invoke<PortCheckResult>("check_port_cmd", {
        target,
        port,
        timeoutMs: timeoutMs ?? null,
    });

export function onToolPingPacket(
    cb: (payload: { session_id: string; packet: PingPacket }) => void,
): Promise<UnlistenFn> {
    return listen<{ session_id: string; packet: PingPacket }>("tool:ping_packet", (e) =>
        cb(e.payload),
    );
}

export function onToolPingSummary(
    cb: (payload: { session_id: string; summary: PingStreamSummary }) => void,
): Promise<UnlistenFn> {
    return listen<{ session_id: string; summary: PingStreamSummary }>("tool:ping_summary", (e) =>
        cb(e.payload),
    );
}

export function onToolTraceHop(
    cb: (payload: TraceHopEvent) => void,
): Promise<UnlistenFn> {
    return listen<TraceHopEvent>("tool:trace_hop", (e) => cb(e.payload));
}

export function onToolTraceDone(
    cb: (payload: TraceDoneEvent) => void,
): Promise<UnlistenFn> {
    return listen<TraceDoneEvent>("tool:trace_done", (e) => cb(e.payload));
}

// --- Seguridad y Auditoría de Riesgos (§4) ---------------------------------

export type RiskSeverity = "low" | "medium" | "high" | "critical";
export type FindingStatus = "open" | "acknowledged" | "ignored" | "resolved";

export interface SecurityFinding {
    id: string;
    device_id: string | null;
    network_id: string;
    rule_id: string;
    severity: RiskSeverity;
    title: string;
    description: string;
    evidence: string | null;
    remediation_advice: string | null;
    status: FindingStatus;
    first_seen_at: string;
    last_seen_at: string;
    resolved_at: string | null;
}

export interface SecurityAuditReport {
    network_id: string;
    score: number;
    total_findings: number;
    critical_count: number;
    high_count: number;
    medium_count: number;
    low_count: number;
    findings: SecurityFinding[];
}

export const getSecurityFindings = (params?: { networkId?: string; deviceId?: string }) =>
    invoke<SecurityFinding[]>("get_security_findings_cmd", {
        networkId: params?.networkId ?? null,
        deviceId: params?.deviceId ?? null,
    });

export const updateFindingStatus = (findingId: string, status: FindingStatus) =>
    invoke<boolean>("update_finding_status_cmd", {
        findingId,
        status,
    });

export const getSecurityScore = (networkId?: string) =>
    invoke<number>("get_security_score_cmd", {
        networkId: networkId ?? null,
    });

export const runSecurityAudit = (networkId?: string) =>
    invoke<SecurityAuditReport>("run_security_audit_cmd", {
        networkId: networkId ?? null,
    });

// Phase 9: Speed Test & Network Health
export interface NetworkHealthSample {
    id: string;
    network_id: string;
    target_type: "gateway" | "dns" | "internet" | string;
    target_ip: string;
    rtt_ms: number | null;
    jitter_ms: number;
    loss_pct: number;
    sample_count: number;
    probes_failed: number;
    method: string;
    created_at: string;
}

export interface SpeedTestResult {
    id: string;
    network_id: string | null;
    server_name: string;
    server_host: string;
    server_city: string | null;
    server_country: string | null;
    download_mbps: number;
    upload_mbps: number;
    latency_ms: number;
    jitter_ms: number;
    bufferbloat_ms: number | null;
    retransmits: number | null;
    protocol: string;
    created_at: string;
}

export interface SpeedTestProgress {
    phase: "init" | "latency" | "download" | "upload" | "finished" | "error" | string;
    progress_pct: number;
    current_mbps: number;
    bytes_transferred: number;
    total_bytes: number;
    latency_ms: number | null;
    jitter_ms: number | null;
    server_info: string | null;
    error_message: string | null;
}

export interface SpeedTestProgressEvent {
    session_id: string;
    progress: SpeedTestProgress;
}

export interface SpeedTestDoneEvent {
    session_id: string;
    result: SpeedTestResult | null;
    error: string | null;
}

export const startSpeedTest = (sessionId: string, networkId?: string) =>
    invoke<void>("start_speed_test_cmd", {
        sessionId,
        networkId: networkId ?? null,
    });

export const getRecentSpeedTests = (params?: { networkId?: string; limit?: number }) =>
    invoke<SpeedTestResult[]>("get_recent_speed_tests_cmd", {
        networkId: params?.networkId ?? null,
        limit: params?.limit ?? null,
    });

export const getNetworkHealthSamples = (networkId?: string, limit?: number) =>
    invoke<NetworkHealthSample[]>("get_network_health_samples_cmd", {
        networkId: networkId ?? null,
        limit: limit ?? null,
    });

export const runHealthProbe = (networkId?: string) =>
    invoke<NetworkHealthSample[]>("run_health_probe_cmd", {
        networkId: networkId ?? null,
    });

export const onSpeedTestProgress = (
    cb: (event: SpeedTestProgressEvent) => void,
) => listen<SpeedTestProgressEvent>("tool:speedtest_progress", (e) => cb(e.payload));

export const onSpeedTestDone = (
    cb: (event: SpeedTestDoneEvent) => void,
) => listen<SpeedTestDoneEvent>("tool:speedtest_done", (e) => cb(e.payload));

// --- Tipos e IPC de Workspaces (Pilar 6 / Fase 10) ---------------------------

export interface Workspace {
    id: string;
    name: string;
    description: string | null;
    icon: string;
    color: string;
    created_at: string;
    updated_at: string;
}

export interface WorkspaceStats {
    device_count: number;
    network_count: number;
    online_device_count: number;
    open_findings_count: number;
}

export interface CreateWorkspacePayload {
    id?: string;
    name: string;
    description?: string;
    icon?: string;
    color?: string;
}

export interface UpdateWorkspacePayload {
    id: string;
    name: string;
    description?: string;
    icon: string;
    color: string;
}

export const listWorkspaces = () => invoke<Workspace[]>("list_workspaces_cmd");

export const getWorkspace = (id: string) =>
    invoke<Workspace | null>("get_workspace_cmd", { id });

export const createWorkspace = (payload: CreateWorkspacePayload) =>
    invoke<Workspace>("create_workspace_cmd", { payload });

export const updateWorkspace = (payload: UpdateWorkspacePayload) =>
    invoke<Workspace>("update_workspace_cmd", { payload });

export const deleteWorkspace = (id: string) =>
    invoke<void>("delete_workspace_cmd", { id });

export const getWorkspaceStats = (id: string) =>
    invoke<WorkspaceStats>("get_workspace_stats_cmd", { id });

export const assignNetworkWorkspace = (networkId: string, workspaceId: string) =>
    invoke<void>("assign_network_workspace_cmd", { networkId, workspaceId });

export const detectWorkspaceForNetwork = (networkId: string) =>
    invoke<string>("detect_workspace_for_network_cmd", { networkId });
