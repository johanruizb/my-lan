import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { getProfile, PROFILES } from "@/lib/profiles";

// ProfileSelect: selector de perfiles con descripción visible (AC-2).
// Mantiene value/onChange API. Descripciones desde profiles.ts (F0.6, hardcode TS).

export function ProfileSelect({
    value,
    onChange,
    className,
    disabled,
    id,
}: {
    value: string;
    onChange: (v: string) => void;
    className?: string;
    disabled?: boolean;
    id?: string;
}) {
    const selected = getProfile(value);
    return (
        <Select value={value} onValueChange={onChange} disabled={disabled}>
            <SelectTrigger
                className={className}
                aria-label="Perfil de escaneo"
                id={id}
            >
                <SelectValue placeholder="Perfil">
                    <span className="truncate">
                        {selected ? selected.label : value}
                    </span>
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                {PROFILES.map((p) => (
                    <SelectItem key={p.value} value={p.value}>
                        <div className="flex flex-col gap-0.5">
                            <span>{p.label}</span>
                            <span className="text-xs text-muted-foreground">
                                {p.description}
                            </span>
                        </div>
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}

/** Genera un id único para un scan (usado como clave de cancelación). */
export function newScanId(): string {
    return `scan-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}
