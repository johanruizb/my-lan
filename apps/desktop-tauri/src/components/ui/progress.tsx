import * as React from "react";
import * as ProgressPrimitive from "@radix-ui/react-progress";
import { cn } from "@/lib/utils";

// Radix Progress: roving focus, ARIA progressbar con valuenow/min/max, keyboard
// accesible por defecto. Micro-motion sutil en el ancho (AC-7).
export const Progress = React.forwardRef<
    React.ElementRef<typeof ProgressPrimitive.Root>,
    React.ComponentPropsWithoutRef<typeof ProgressPrimitive.Root> & {
        /** Indeterminado: barra animada cuando no hay valor conocido (AC-15 aria-busy). */
        indeterminate?: boolean;
    }
>(({ className, value, indeterminate, ...props }, ref) => (
    <ProgressPrimitive.Root
        ref={ref}
        className={cn(
            "relative h-2 w-full overflow-hidden rounded-full bg-muted",
            className,
        )}
        {...props}
    >
        <ProgressPrimitive.Indicator
            className="h-full flex-1 bg-primary transition-[width] duration-300 ease-out"
            style={
                indeterminate
                    ? {
                          width: "40%",
                          animation:
                              "progress-indeterminate 1.2s ease-in-out infinite",
                      }
                    : { transform: `translateX(-${100 - (value ?? 0)}%)` }
            }
        />
    </ProgressPrimitive.Root>
));
Progress.displayName = "Progress";
