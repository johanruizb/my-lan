import type { HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export function Card({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
    return (
        <div
            className={cn(
                "rounded-xl border border-border bg-card text-card-foreground shadow-sm",
                className,
            )}
            {...props}
        />
    );
}

export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
    variant?: "default" | "toolbar";
}

export function CardHeader({
    className,
    variant = "default",
    ...props
}: CardHeaderProps) {
    return (
        <div
            className={cn(
                "p-3",
                variant === "toolbar"
                    ? "flex flex-row items-center justify-between gap-4"
                    : "flex flex-col gap-1",
                className,
            )}
            {...props}
        />
    );
}

export interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {
    as?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
}

export function CardTitle({
    as: Comp = "h3",
    className,
    ...props
}: CardTitleProps) {
    return (
        <Comp
            className={cn("text-base font-semibold leading-none", className)}
            {...props}
        />
    );
}

export function CardDescription({
    className,
    ...props
}: HTMLAttributes<HTMLParagraphElement>) {
    return (
        <p
            className={cn("text-sm text-muted-foreground", className)}
            {...props}
        />
    );
}

export function CardContent({
    className,
    ...props
}: HTMLAttributes<HTMLDivElement>) {
    return <div className={cn("p-6 pt-0", className)} {...props} />;
}

export function CardFooter({
    className,
    ...props
}: HTMLAttributes<HTMLDivElement>) {
    return (
        <div
            className={cn("flex items-center p-6 pt-0", className)}
            {...props}
        />
    );
}
