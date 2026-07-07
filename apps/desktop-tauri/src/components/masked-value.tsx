import { useState } from "react";
import { cn } from "@/lib/utils";
import { useCensorship } from "@/components/censorship-provider";
import { isMacField, isSensitive, maskMac, maskValue } from "@/lib/censor";

// <MaskedValue> (AC-1/AC-5): renderiza un valor de campo aplicando el modo
// censura. Sigue los patrones de DeviceDetail.tsx / Devices.tsx para la clase
// `mono` via `cn`.
//
// Comportamiento:
//   - censorship OFF o value == null -> valor crudo (o "—"), sin cambios.
//   - Campo MAC-family -> placeholder constante "••••", NO hover-revelable
//     (AC-5: la MAC nunca se revela en la UI). aria-label "MAC oculta" para
//     que el lector de pantalla no anuncie los puntos.
//   - Otro campo sensible -> blur-sm + select-none sobre el valor real;
//     onMouseEnter/onFocus revela (quita el blur); onMouseLeave/onBlur re-enmascara.
//     aria-label fija al valor enmascarado (maskValue) para que el lector de
//     pantalla anuncie la versión censurada incluso cuando se revela visualmente.
//   - Campo no sensible -> pass-through (seguro envolver cualquier campo).

interface MaskedValueProps {
    field: string;
    value: string | null;
    mono?: boolean;
}

export function MaskedValue({ field, value, mono }: MaskedValueProps) {
    const { censorshipEnabled } = useCensorship();
    const [revealed, setRevealed] = useState(false);

    // Passthrough: censura apagada, valor nulo o campo no sensible.
    if (!censorshipEnabled || value == null || !isSensitive(field)) {
        return (
            <span className={cn(mono && "font-mono text-sm")}>
                {value ?? "—"}
            </span>
        );
    }

    // MAC-family: placeholder constante, nunca hover-revelable (AC-5).
    if (isMacField(field)) {
        return (
            <span
                className={cn(mono && "font-mono text-sm")}
                aria-label="MAC oculta"
            >
                {maskMac()}
            </span>
        );
    }

    // Otro campo sensible: blur + reveal-on-hover/focus.
    // Mientras esta blurred, select-none evita copia casual del DOM; al revelar
    // se permite seleccion (transitorio).
    // aria-label fija al valor enmascarado: el lector de pantalla anuncia la
    // versión censurada (maskValue) siempre, incluso cuando el valor real se
    // muestra visualmente al revelar.
    const masked = maskValue(field, value);
    return (
        <span
            className={cn(
                "transition-[filter] select-none",
                mono && "font-mono text-sm",
                !revealed && "blur-sm",
            )}
            onMouseEnter={() => setRevealed(true)}
            onMouseLeave={() => setRevealed(false)}
            onFocus={() => setRevealed(true)}
            onBlur={() => setRevealed(false)}
            tabIndex={0}
            aria-label={masked}
        >
            {revealed ? value : masked}
        </span>
    );
}
