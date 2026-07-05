import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

// Primitive label+control+helper+error alineados (elimina mt-5 hack x6).
// Reusa por F1.3 Empty CTAs layout, F4.2 helpers, AC-15.

export interface FormFieldProps {
    label: string;
    htmlFor?: string;
    children: ReactNode;
    helper?: string;
    error?: string;
    className?: string;
    required?: boolean;
}

export function FormField({
    label,
    htmlFor,
    children,
    helper,
    error,
    className,
    required,
}: FormFieldProps) {
    return (
        <div className={cn("flex flex-col gap-1.5", className)}>
            <label
                htmlFor={htmlFor}
                className="text-sm font-medium leading-none text-foreground"
            >
                {label}
                {required && <span className="ml-0.5 text-destructive">*</span>}
            </label>
            {children}
            {helper && !error && (
                <p
                    id={htmlFor ? `${htmlFor}-helper` : undefined}
                    className="text-xs text-muted-foreground"
                >
                    {helper}
                </p>
            )}
            {error && (
                <p className="text-xs text-destructive" role="alert">
                    {error}
                </p>
            )}
        </div>
    );
}
