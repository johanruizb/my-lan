import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";

const PROFILES: { value: string; label: string }[] = [
    { value: "quick", label: "Quick" },
    { value: "normal", label: "Normal" },
    { value: "deep", label: "Deep" },
    { value: "iot", label: "IoT" },
    { value: "router", label: "Router" },
];

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
    return (
        <Select value={value} onValueChange={onChange} disabled={disabled}>
            <SelectTrigger
                className={className}
                aria-label="Perfil de escaneo"
                id={id}
            >
                <SelectValue placeholder="Perfil" />
            </SelectTrigger>
            <SelectContent>
                {PROFILES.map((p) => (
                    <SelectItem key={p.value} value={p.value}>
                        {p.label}
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
