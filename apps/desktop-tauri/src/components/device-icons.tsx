import {
    Router,
    Smartphone,
    Laptop,
    Monitor,
    Tv,
    Printer,
    Camera,
    HardDrive,
    Gamepad2,
    Cpu,
    HelpCircle,
    type LucideIcon,
} from "lucide-react";

// Mapeo de device_type (snake_case de mylan-core::DeviceType) a icono lucide
// (AC-5). Los valores son los que emite serde: router, phone, laptop, desktop,
// tv, printer, camera, nas, console, iot, unknown.
const DEVICE_ICON: Record<string, LucideIcon> = {
    router: Router,
    phone: Smartphone,
    laptop: Laptop,
    desktop: Monitor,
    tv: Tv,
    printer: Printer,
    camera: Camera,
    nas: HardDrive,
    console: Gamepad2,
    iot: Cpu,
    unknown: HelpCircle,
};

const DEVICE_LABEL: Record<string, string> = {
    router: "Router",
    phone: "Teléfono",
    laptop: "Portátil",
    desktop: "Escritorio",
    tv: "TV",
    printer: "Impresora",
    camera: "Cámara",
    nas: "NAS",
    console: "Consola",
    iot: "IoT",
    unknown: "Desconocido",
};

/** Devuelve el icono lucide para un device_type; fallback a HelpCircle. */
export function deviceIcon(type: string): LucideIcon {
    return DEVICE_ICON[type] ?? HelpCircle;
}

/** Devuelve una etiqueta legible (español) para un device_type. */
export function deviceLabel(type: string): string {
    return DEVICE_LABEL[type] ?? "Desconocido";
}
