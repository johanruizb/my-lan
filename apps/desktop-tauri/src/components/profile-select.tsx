import { Select } from "@/components/ui/select";

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
}: {
  value: string;
  onChange: (v: string) => void;
  className?: string;
}) {
  return (
    <Select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={className}
    >
      {PROFILES.map((p) => (
        <option key={p.value} value={p.value}>
          {p.label}
        </option>
      ))}
    </Select>
  );
}

/** Genera un id único para un scan (usado como clave de cancelación). */
export function newScanId(): string {
  return `scan-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}