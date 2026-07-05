// Metadata hardcode perfiles (cero backend, estática).
// Perfil: descripción ES + tradeoff velocidad/profundidad + cuándo-usar.
// Reusa por F1.2 ProfileSelect.

export interface ProfileMeta {
    value: string;
    label: string;
    description: string;
    tradeoff: string;
    whenToUse: string;
}

export const PROFILES: ProfileMeta[] = [
    {
        value: "quick",
        label: "Rápido",
        description:
            "Descubrimiento veloz: ping TCP a los puertos más comunes.",
        tradeoff:
            "Más rápido, pero puede perder dispositivos que solo responden a otros puertos.",
        whenToUse: "Cuando quieres ver rápido qué hay en la red.",
    },
    {
        value: "normal",
        label: "Normal",
        description:
            "Equilibrio entre velocidad y cobertura: ARP + ICMP + puertos comunes.",
        tradeoff: "Cobertura buena en tiempo razonable.",
        whenToUse: "Uso diario para revisar tu red.",
    },
    {
        value: "deep",
        label: "Profundo",
        description:
            "Cobertura máxima: todos los métodos + escaneo de puertos extenso.",
        tradeoff: "Más lento, pero encuentra más dispositivos y servicios.",
        whenToUse: "Cuando necesitas un inventario completo y detallado.",
    },
    {
        value: "iot",
        label: "IoT",
        description:
            "Optimizado para dispositivos IoT (cámaras, sensores, bombillas).",
        tradeoff: "Prioriza puertos típicos de IoT sobre otros.",
        whenToUse: "Cuando buscas dispositivos del hogar inteligente.",
    },
    {
        value: "router",
        label: "Router",
        description: "Optimizado para encontrar routers y gateways.",
        tradeoff: "Prioriza puertos de gestión de red.",
        whenToUse: "Cuando quieres localizar y revisar tu router.",
    },
];

export function getProfile(value: string): ProfileMeta | undefined {
    return PROFILES.find((p) => p.value === value);
}
