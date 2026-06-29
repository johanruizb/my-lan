// Catálogo de enmascaramiento para el modo censura (AC-1/AC-2).
//
// Una sola fuente de verdad para la lista de campos sensibles y las funciones
// de texto de mascara. La consume:
//   - `<MaskedValue>` (UI, blur + reveal-on-hover) via `isSensitive`/`maskValue`.
//   - Los exports CSV/JSON: el backend Rust enmascara en la frontera de
//     serializacion (`src-tauri/src/commands.rs` export_devices_cmd /
//     export_services_cmd) con un catalogo espejo que **DEBE MANTENERSE
//     SINCRONIZADO** con este archivo (mismos nombres de campo y mismo formato
//     de mascara parcial). Ver el comentario cruzado en el lado Rust.
//
// Estrategia por campo (spec):
//   - MAC (primary_mac/mac/gateway_mac): placeholder constante "••••", nunca
//     revelable (AC-5).
//   - IP/hostname/display_name/gateway_ip/dns_servers/cidr: mascara parcial
//     (e.g. "192.168.1.42" -> "192.168.*.*") en exports; en UI se usa blur.
//   - vendor/manufacturer/banner/product/version/port/notes: visibles.

/** Placeholder constante para MAC: nunca revela (AC-5). */
export function maskMac(): string {
    return "••••";
}

/**
 * Mascara parcial de IPv4/IPv6.
 * IPv4: cero los dos ultimos octetos -> "192.168.1.42" => "192.168.*.*".
 * IPv6: cero los ultimos 3 grupos -> "2001:db8::1:2:3" => "2001:db8::*:*:*".
 */
export function maskIp(ip: string): string {
    if (ip.includes(":")) {
        const groups = ip.split(":");
        const n = groups.length;
        if (n > 3) {
            return [...groups.slice(0, n - 3), "*", "*", "*"].join(":");
        }
        return "*".repeat(n).split("").join(":");
    }
    const octets = ip.split(".");
    if (octets.length === 4) {
        return [octets[0], octets[1], "*", "*"].join(".");
    }
    return "*";
}

/**
 * Mascara parcial de hostname: conserva la primera etiqueta, enmascara el
 * resto -> "router.lan" => "router.*". Si no hay punto, "*" completa.
 */
export function maskHostname(h: string): string {
    const idx = h.indexOf(".");
    if (idx === -1) return "*";
    return `${h.slice(0, idx)}.*`;
}

/**
 * Mascara de CIDR: enmascara la parte de direccion pero conserva el prefix-len
 * (es estructural, no un identificador) -> "192.168.1.0/24" => "192.168.*.0/24".
 */
export function maskCidr(cidr: string): string {
    const slash = cidr.indexOf("/");
    if (slash === -1) return maskIp(cidr);
    const addr = cidr.slice(0, slash);
    const prefix = cidr.slice(slash);
    return `${maskIp(addr)}${prefix}`;
}

/**
 * Mascara una lista de DNS (servidores): enmascara cada entrada como IP.
 */
export function maskDns(servers: string[]): string {
    return servers.map((s) => maskIp(s)).join(", ");
}

/** Catálogo de campos estrictamente identificadores (sensibles). */
const SENSITIVE_FIELDS = new Set<string>([
    "primary_ip",
    "primary_mac",
    "hostname",
    "display_name",
    "gateway_ip",
    "gateway_mac",
    "dns_servers",
    "cidr",
    "ip",
    "mac",
    "device_ip",
]);

/** true si el campo es un identificador estricto que debe enmascararse. */
export function isSensitive(field: string): boolean {
    return SENSITIVE_FIELDS.has(field);
}

/** Campos MAC: placeholder constante, nunca hover-revelable (AC-5). */
const MAC_FIELDS = new Set<string>(["primary_mac", "mac", "gateway_mac"]);

export function isMacField(field: string): boolean {
    return MAC_FIELDS.has(field);
}

/**
 * Dispatch por nombre de campo a la mascara de texto adecuada.
 * Usado por los exports (parcial-mask) y por `<MaskedValue>` para su estado
 * no-revelado (el contenido bajo el blur). Mantiene un solo catalogo.
 */
export function maskValue(field: string, value: string): string {
    if (!isSensitive(field)) return value;
    switch (field) {
        case "primary_mac":
        case "mac":
        case "gateway_mac":
            return maskMac();
        case "dns_servers":
            // dns_servers llega como string plano o separado por comas.
            return maskDns(value.split(",").map((s) => s.trim()));
        case "cidr":
            return maskCidr(value);
        case "hostname":
            return maskHostname(value);
        case "display_name":
            // display_name es texto libre; mascara parcial de hostname aplica
            // razonablemente (conserva primera etiqueta).
            return maskHostname(value);
        default:
            // ip / primary_ip / gateway_ip / device_ip
            return maskIp(value);
    }
}
