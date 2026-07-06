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
        is_online: true,
        notes: null,
        ...overrides,
    };
}

// AC-18: fixtures variados para cubrir las 3 ramas de `deriveTrustState`
// (trusted/recognized/unknown) + offline. Valores numéricos-string para
// `confidence` (el backend serializa `Confidence(u8)` como número, no "high").
export const deviceFixtures: Device[] = [
    // (a) trusted + online + router.
    makeDevice({
        id: "dev-1",
        primary_ip: "192.168.1.1",
        hostname: "router.lan",
        display_name: "Router principal",
        device_type: "router",
        confidence: "90",
        is_trusted: true,
        is_online: true,
    }),
    // (b) recognized + online + phone.
    makeDevice({
        id: "dev-2",
        primary_ip: "192.168.1.50",
        hostname: "phone.lan",
        display_name: "Móvil de Johan",
        device_type: "phone",
        confidence: "60",
        is_trusted: false,
        is_online: true,
    }),
    // (c) unknown + offline + tablet.
    makeDevice({
        id: "dev-3",
        primary_ip: "192.168.1.99",
        hostname: "tablet.lan",
        display_name: "Tablet salón",
        device_type: "tablet",
        confidence: "30",
        is_trusted: false,
        is_online: false,
    }),
];

export const defaultSettings: Settings = {
    db_path: "/tmp/mylan.db",
    default_profile: "normal",
    theme: "light",
    censorship_enabled: true,
};