import type { HTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

// Chips/badges para tipo de dispositivo y flags (AC-9).
const badgeVariants = cva(
    "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium transition-colors",
    {
        variants: {
            variant: {
                default:
                    "border-transparent bg-primary text-primary-foreground",
                secondary:
                    "border-transparent bg-secondary text-secondary-foreground",
                outline: "border-border text-foreground",
                info: "border-primary/30 bg-primary/10 text-primary-strong",
                success:
                    "border-success/30 bg-success/10 text-success-strong",
                warning:
                    "border-warning/30 bg-warning/10 text-warning-strong",
                destructive:
                    "border-destructive/30 bg-destructive/10 text-destructive-strong",
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
