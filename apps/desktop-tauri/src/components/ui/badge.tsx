import type { HTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

// Chips/badges para tipo de dispositivo y flags (AC-9).
const badgeVariants = cva(
    "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium transition-colors",
    {
        variants: {
            variant: {
                default:
                    "border-transparent bg-primary text-primary-foreground",
                secondary:
                    "border-transparent bg-secondary text-secondary-foreground",
                outline: "border-border text-foreground",
                success:
                    "border-transparent bg-success text-success-foreground",
                warning:
                    "border-transparent bg-warning text-warning-foreground",
                destructive:
                    "border-transparent bg-destructive text-destructive-foreground",
            },
        },
        defaultVariants: { variant: "default" },
    },
);

export interface BadgeProps
    extends
        HTMLAttributes<HTMLSpanElement>,
        VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
    return (
        <span
            className={cn(badgeVariants({ variant }), className)}
            {...props}
        />
    );
}

export { badgeVariants };
