import * as React from "react";
import { cn } from "@/lib/utils";

// Switch accesible (rol ARIA `switch`) sin dependencia extra. Soporta
// focus-visible, teclado (Space/Enter via onClick nativo de button) y
// animación de deslizamiento. API compatible con Radix Switch
// (`checked`/`onCheckedChange`) para permitir migración directa si se añade
// @radix-ui/react-switch después. Reusa por T17 Settings (censura) y futuros
// toggles binarios accesibles (p. ej. confiable en DeviceDetail).
export interface SwitchProps extends Omit<
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    "onChange" | "type" | "role"
> {
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
}

export const Switch = React.forwardRef<HTMLButtonElement, SwitchProps>(
    ({ checked, onCheckedChange, className, disabled, ...props }, ref) => (
        <button
            ref={ref}
            type="button"
            role="switch"
            aria-checked={checked}
            disabled={disabled}
            onClick={() => onCheckedChange(!checked)}
            className={cn(
                "peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                checked ? "bg-primary" : "bg-input",
                className,
            )}
            {...props}
        >
            <span
                aria-hidden
                className={cn(
                    "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform",
                    checked ? "translate-x-4" : "translate-x-0",
                )}
            />
        </button>
    ),
);
Switch.displayName = "Switch";
