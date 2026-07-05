import { HelpCircle } from "lucide-react";
import {
    Tooltip,
    TooltipTrigger,
    TooltipContent,
} from "@/components/ui/tooltip";
import { getGlossary } from "@/lib/glossary";
import { cn } from "@/lib/utils";

// Wrapper Radix Tooltip (consume TooltipProvider de App.tsx:525).
// Usa glossary.ts (F0.3) para término→tooltip explicativo del concepto.
// aria-describedby apunta a tooltip content (a11y para lectores de pantalla).
// Reusa por F1.1 (tooltips jerga en 4 screens).

export interface InfoTooltipProps {
    term: string;
    glossaryKey?: string;
    className?: string;
    side?: "top" | "right" | "bottom" | "left";
}

export function InfoTooltip({
    term,
    glossaryKey,
    className,
    side = "top",
}: InfoTooltipProps) {
    const entry = getGlossary(glossaryKey ?? term);
    const text = entry?.tooltip ?? term;
    const tipId = `tt-${(glossaryKey ?? term).toLowerCase().replace(/\s+/g, "-")}`;
    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <button
                    type="button"
                    className={cn(
                        "inline-flex items-center text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-sm",
                        className,
                    )}
                    aria-describedby={tipId}
                    aria-label={`Más información: ${term}`}
                >
                    <HelpCircle className="h-3.5 w-3.5" aria-hidden />
                </button>
            </TooltipTrigger>
            <TooltipContent id={tipId} side={side}>
                <span className="block max-w-xs text-left">{text}</span>
            </TooltipContent>
        </Tooltip>
    );
}
