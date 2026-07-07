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
export interface ScanSummaryDto {
    id: string;
    profile: string;
    status: string;
    started_at: string;
    finished_at: string | null;
    hosts_alive: number;
    hosts_new: number;
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
}

// --- Wrappers tipados de invoke ---------------------------------------------

export const detectInterface = (interfaceOverride?: string) =>
    invoke<LanInterfaceDto>("detect_interface_cmd", {
        interface: interfaceOverride ?? null,
    });

export const listDevices = () => invoke<Device[]>("list_devices_cmd");

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
