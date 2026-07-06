// AC-19: Tests de contrato de `lib/tauri.ts` con `mockIPC` oficial.
// Valida que cada wrapper invoque el command name correcto con el payload
// esperado (detecta typos en command names snake_case del backend Rust).
// `mockIPC` intercepta `__TAURI_INTERNALS__.invoke`; `clearMocks` limpia
// entre tests para evitar sangrado de estado (riesgo R3).

import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
    cancelScan,
    dbPath,
    detectInterface,
    exportDevices,
    exportServices,
    getDevice,
    getNetworkName,
    getSettings,
    listDevices,
    listScans,
    listServices,
    runDiscovery,
    scanPorts,
    setNetworkName,
    setSettings,
} from "@/lib/tauri";
import { defaultSettings } from "./fixtures";

// Handler tipado: registra cada (cmd, payload) y devuelve respuestas válidas.
type Call = { cmd: string; payload: unknown };
let calls: Call[];
let handler: ReturnType<typeof vi.fn>;

beforeEach(() => {
    calls = [];
    handler = vi.fn((cmd: string, payload?: unknown) => {
        calls.push({ cmd, payload });
        switch (cmd) {
            case "list_devices_cmd":
                return [];
            case "get_device_cmd":
                return { device: null, services: [] };
            case "detect_interface_cmd":
                return null;
            case "run_discovery_cmd":
                return {
                    scan_id: "s1",
                    network_id: "192.168.1.0/24",
                    hosts_alive: 0,
                    hosts_new: 0,
                    duration_ms: 0,
                };
            case "scan_ports_cmd":
                return [];
            case "cancel_scan_cmd":
                return true;
            case "list_services_cmd":
                return [];
            case "list_scans_cmd":
                return [];
            case "export_devices_cmd":
            case "export_services_cmd":
                return "ok";
            case "db_path_cmd":
                return "/tmp/mylan.db";
            case "get_settings_cmd":
                return defaultSettings;
            case "set_settings_cmd":
                return null;
            case "get_network_name_cmd":
                return { name: "auto-net", source: "auto" };
            case "set_network_name_cmd":
                return null;
            default:
                return null;
        }
    });
    mockIPC(handler);
});

afterEach(() => {
    clearMocks();
});

describe("Contrato lib/tauri.ts — command names (AC-19)", () => {
    it("listDevices invoca list_devices_cmd", async () => {
        await listDevices();
        expect(handler).toHaveBeenCalledWith("list_devices_cmd", {});
    });

    it("getDevice invoca get_device_cmd con {ip}", async () => {
        await getDevice("192.168.1.42");
        expect(handler).toHaveBeenCalledWith("get_device_cmd", {
            ip: "192.168.1.42",
        });
    });

    it("detectInterface invoca detect_interface_cmd con {interface: null} por defecto", async () => {
        await detectInterface();
        expect(handler).toHaveBeenCalledWith("detect_interface_cmd", {
            interface: null,
        });
    });

    it("detectInterface pasa el override de interfaz", async () => {
        await detectInterface("eth0");
        expect(handler).toHaveBeenCalledWith("detect_interface_cmd", {
            interface: "eth0",
        });
    });

    it("runDiscovery invoca run_discovery_cmd con profile, interface y scanId", async () => {
        await runDiscovery("normal", "scan-1");
        expect(handler).toHaveBeenCalledWith("run_discovery_cmd", {
            profile: "normal",
            interface: null,
            scanId: "scan-1",
        });
    });

    it("runDiscovery pasa el override de interfaz", async () => {
        await runDiscovery("normal", "scan-2", "wlan0");
        expect(handler).toHaveBeenCalledWith("run_discovery_cmd", {
            profile: "normal",
            interface: "wlan0",
            scanId: "scan-2",
        });
    });

    it("scanPorts invoca scan_ports_cmd con ip, profile, scanId", async () => {
        await scanPorts("192.168.1.42", "normal", "scan-1");
        expect(handler).toHaveBeenCalledWith("scan_ports_cmd", {
            ip: "192.168.1.42",
            profile: "normal",
            scanId: "scan-1",
        });
    });

    it("cancelScan invoca cancel_scan_cmd con {scanId}", async () => {
        await cancelScan("scan-1");
        expect(handler).toHaveBeenCalledWith("cancel_scan_cmd", {
            scanId: "scan-1",
        });
    });

    it("listServices invoca list_services_cmd con {filters: {}}", async () => {
        await listServices();
        expect(handler).toHaveBeenCalledWith("list_services_cmd", {
            filters: {},
        });
    });

    it("listScans invoca list_scans_cmd", async () => {
        await listScans();
        expect(handler).toHaveBeenCalledWith("list_scans_cmd", {});
    });

    it("exportDevices invoca export_devices_cmd con format y outputPath null", async () => {
        await exportDevices("csv");
        expect(handler).toHaveBeenCalledWith("export_devices_cmd", {
            format: "csv",
            outputPath: null,
        });
    });

    it("exportServices invoca export_services_cmd con format y outputPath", async () => {
        await exportServices("json", "/tmp/out.json");
        expect(handler).toHaveBeenCalledWith("export_services_cmd", {
            format: "json",
            outputPath: "/tmp/out.json",
        });
    });

    it("dbPath invoca db_path_cmd", async () => {
        await dbPath();
        expect(handler).toHaveBeenCalledWith("db_path_cmd", {});
    });

    it("getSettings invoca get_settings_cmd", async () => {
        await getSettings();
        expect(handler).toHaveBeenCalledWith("get_settings_cmd", {});
    });

    it("setSettings invoca set_settings_cmd con {settings}", async () => {
        await setSettings(defaultSettings);
        expect(handler).toHaveBeenCalledWith("set_settings_cmd", {
            settings: defaultSettings,
        });
    });

    it("getNetworkName invoca get_network_name_cmd con {networkId}", async () => {
        await getNetworkName("192.168.1.0/24");
        expect(handler).toHaveBeenCalledWith("get_network_name_cmd", {
            networkId: "192.168.1.0/24",
        });
    });

    it("setNetworkName invoca set_network_name_cmd con {networkId, label}", async () => {
        await setNetworkName("192.168.1.0/24", "mi-red");
        expect(handler).toHaveBeenCalledWith("set_network_name_cmd", {
            networkId: "192.168.1.0/24",
            label: "mi-red",
        });
    });
});