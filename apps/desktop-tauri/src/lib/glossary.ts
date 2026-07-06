// Catálogo unificado término→{traducción ES (opcional), tooltip explicativo del concepto}.
// Cobertura exhaustiva trace Lane 3 (16 términos). Fuente única (consolida F1.1+F4.1).
// Perfiles (quick/normal/deep/iot/router) → profiles.ts (F0.6), NO aquí.

export interface GlossaryEntry {
    term: string;
    translation?: string;
    tooltip: string;
}

export const GLOSSARY: Record<string, GlossaryEntry> = {
    cidr: {
        term: "CIDR",
        tooltip:
            "Notación que indica tu red y su tamaño. Por ejemplo, 192.168.1.0/24 significa la red 192.168.1.0 con hasta 254 dispositivos.",
    },
    mac: {
        term: "MAC",
        tooltip:
            "Identificador único de fábrica de la tarjeta de red de un dispositivo. No cambia aunque cambie la IP.",
    },
    ip: {
        term: "IP",
        tooltip:
            "Dirección única que identifica un dispositivo en la red, como un número de teléfono para conectarlo.",
    },
    gateway: {
        term: "Gateway",
        translation: "Puerta de enlace",
        tooltip:
            "El dispositivo (normalmente tu router) que conecta tu red a Internet. Todo el tráfico sale por aquí.",
    },
    dns: {
        term: "DNS",
        tooltip:
            "Servicio que traduce nombres web (ej. google.com) a direcciones IP. Como una agenda de teléfonos para Internet.",
    },
    hostname: {
        term: "Hostname",
        translation: "Nombre del equipo",
        tooltip:
            "El nombre que un dispositivo usa para identificarse en la red, cuando está disponible.",
    },
    vendor: {
        term: "Vendor",
        translation: "Fabricante",
        tooltip:
            "El fabricante de la tarjeta de red del dispositivo, deducido de su MAC.",
    },
    banner: {
        term: "Banner",
        tooltip:
            "Texto que un servicio abierto devuelve al conectarse. Suele incluir nombre y versión del software.",
    },
    confianza: {
        term: "Confianza",
        tooltip:
            "Qué tan seguro estamos de la clasificación del tipo de dispositivo. Más alto = más seguro.",
    },
    prefix_len: {
        term: "prefix_len",
        tooltip:
            "Tamaño de la red (cuántos dispositivos caben). /24 = 254 dispositivos, /16 = 65 mil.",
    },
    default_route: {
        term: "default route",
        translation: "ruta por defecto",
        tooltip:
            "La ruta por donde sale todo el tráfico que no es local. Normalmente tu router.",
    },
    ssid: {
        term: "SSID",
        tooltip: "El nombre de tu red WiFi, el que ves al conectar equipos.",
    },
    protocolo: {
        term: "Protocolo",
        tooltip: "El lenguaje de comunicación del servicio (tcp o udp).",
    },
    puerto: {
        term: "Puerto",
        tooltip:
            "Número (1-65535) que identifica un servicio concreto en un dispositivo. Ej: 80 = web, 443 = web segura.",
    },
    servicio: {
        term: "Servicio",
        tooltip:
            "Un programa escuchando en un puerto del dispositivo (web, SSH, impresora, etc.).",
    },
    sondeando: {
        term: "Sondeando",
        translation: "Explorando",
        tooltip:
            "Probando uno a uno los hosts de la red para ver cuáles responden.",
    },
    hosts: {
        term: "hosts",
        translation: "dispositivos",
        tooltip:
            "Equipos conectados a la red (tu PC, móvil, TV, impresora, etc.).",
    },
};

export function getGlossary(key: string): GlossaryEntry | undefined {
    return GLOSSARY[key.toLowerCase()];
}
