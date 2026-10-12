import { useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";

interface CopyButtonProps {
    value?: string | null;
    label: string;
    className?: string;
    iconClassName?: string;
}

export function CopyButton({
    value,
    label,
    className,
    iconClassName,
}: CopyButtonProps) {
    const [copied, setCopied] = useState(false);
    const { toast } = useToast();
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        return () => {
            if (timerRef.current) clearTimeout(timerRef.current);
        };
    }, []);

    if (!value || value === "—") return null;

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();
        let success = false;

        if (navigator?.clipboard?.writeText) {
            try {
                await navigator.clipboard.writeText(value);
                success = true;
            } catch {
                success = false;
            }
        }

        if (!success && typeof document !== "undefined") {
            try {
                const textarea = document.createElement("textarea");
                textarea.value = value;
                textarea.style.position = "fixed";
                textarea.style.opacity = "0";
                document.body.appendChild(textarea);
                textarea.focus();
                textarea.select();
                success = document.execCommand("copy");
                document.body.removeChild(textarea);
            } catch {
                success = false;
            }
        }

        if (success) {
            setCopied(true);
            toast(`${label} copiado al portapapeles.`, "default");
            if (timerRef.current) clearTimeout(timerRef.current);
            timerRef.current = setTimeout(() => setCopied(false), 1500);
        } else {
            toast(`No se pudo copiar ${label.toLowerCase()}.`, "error");
        }
    };

    return (
        <button
            type="button"
            onClick={handleCopy}
            title={`Copiar ${label}`}
            aria-label={`Copiar ${label}`}
            className={cn(
                "inline-flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                className,
            )}
        >
            {copied ? (
                <Check
                    className={cn("h-3 w-3 text-emerald-500", iconClassName)}
                    aria-hidden
                />
            ) : (
                <Copy className={cn("h-3 w-3", iconClassName)} aria-hidden />
            )}
        </button>
    );
}
