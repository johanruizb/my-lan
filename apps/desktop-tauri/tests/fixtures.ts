import type { Device, Settings } from "@/lib/tauri";

// Fixtures compartidas por tests de componentes, hooks y screens (AC-22).
// Datos deterministas; sin MACs/IPs reales (regla pre-push-safety).

export function makeDevice(overrides: Partial<Device> = {}): Device {
    return {
        id: "dev-1",
        network_id: "192.168.1.0/24",
        primary_mac: "aa:bb:cc:dd:ee:ff",
        primary_ip: "192.168.1.42",
        hostname: "router.lan",
        display_name: "router.lan",
        vendor: "ExampleVendor",
        manufacturer: null,
        model: null,
        device_type: "router",
        os_family: null,
        confidence: "high",
        first_seen_at: "2025-01-01T00:00:00Z",
        last_seen_at: "2025-01-02T00:00:00Z",
        is_trusted: false,
        is_hidden: false,
        notes: null,
        ...overrides,
    };
}

export const deviceFixtures: Device[] = [
    makeDevice({
        id: "dev-1",
        primary_ip: "192.168.1.10",
        hostname: "router.lan",
        display_name: "router.lan",
    }),
    makeDevice({
        id: "dev-2",
        primary_ip: "192.168.1.11",
        hostname: "laptop.lan",
        display_name: "laptop.lan",
    }),
    makeDevice({
        id: "dev-3",
        primary_ip: "192.168.1.12",
        hostname: "phone.lan",
        display_name: "phone.lan",
    }),
];

export const defaultSettings: Settings = {
    db_path: "/tmp/mylan.db",
    default_profile: "normal",
    theme: "light",
    censorship_enabled: true,
};