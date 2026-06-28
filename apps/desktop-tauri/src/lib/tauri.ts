// Capa IPC tipada sobre `@tauri-apps/api`.
//
// CONVENCIÓN DE NOMBRES: los DTOs del backend Rust se serializan en **snake_case**
// (los tipos de `mylan-core` —`Device`, `Service`, `ServiceExportRow`,
// `ScanProgress`— derivan `Serialize` sin `rename_all`, y los DTOs propios del
// backend siguen la misma convención para mantener una sola forma en todo el
// frontend). Los tipos aquí espejan exactamente los nombres que emite serde.

import { invoke } from "@tauri-apps/api/core";
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
  confidence: string;
  first_seen_at: string;
  last_seen_at: string;
  is_trusted: boolean;
  is_hidden: boolean;
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

export interface Settings {
  db_path: string;
  default_profile: string;
}

// --- Wrappers tipados de invoke ---------------------------------------------

export const detectInterface = (interfaceOverride?: string) =>
  invoke<LanInterfaceDto>("detect_interface_cmd", {
    interface: interfaceOverride ?? null,
  });

export const listDevices = () => invoke<Device[]>("list_devices_cmd");

export const getDevice = (ip: string) =>
  invoke<DeviceDetailDto>("get_device_cmd", { ip });

export const runDiscovery = (profile: string, interfaceOverride?: string) =>
  invoke<ScanOutcomeDto>("run_discovery_cmd", {
    profile,
    interface: interfaceOverride ?? null,
  });

export const scanPorts = (ip: string, profile: string, scanId: string) =>
  invoke<Service[]>("scan_ports_cmd", { ip, profile, scanId });

export const cancelScan = (scanId: string) =>
  invoke<boolean>("cancel_scan_cmd", { scanId });

export const listServices = (filters: ServiceFiltersDto = {}) =>
  invoke<ServiceExportRow[]>("list_services_cmd", { filters });

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

// --- Wrappers tipados de listen ---------------------------------------------

export function onScanProgress(cb: (p: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>("scan:progress", (e) => cb(e.payload));
}

export function onScanHeartbeat(cb: (h: ScanHeartbeat) => void): Promise<UnlistenFn> {
  return listen<ScanHeartbeat>("scan:heartbeat", (e) => cb(e.payload));
}

export function onScanCancelled(cb: (c: ScanCancelled) => void): Promise<UnlistenFn> {
  return listen<ScanCancelled>("scan:cancelled", (e) => cb(e.payload));
}

export function onScanFinished(cb: (f: ScanFinished) => void): Promise<UnlistenFn> {
  return listen<ScanFinished>("scan:finished", (e) => cb(e.payload));
}

export function onScanStarted(cb: (s: ScanStarted) => void): Promise<UnlistenFn> {
  return listen<ScanStarted>("scan:started", (e) => cb(e.payload));
}

export function onDbImported(cb: () => void): Promise<UnlistenFn> {
  return listen("db:imported", () => cb());
}