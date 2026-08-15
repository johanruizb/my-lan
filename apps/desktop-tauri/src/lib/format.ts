import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import "dayjs/locale/es";

dayjs.extend(relativeTime);
dayjs.locale("es");

// Helper unificado de timestamps (elimina drift Devices.fromNow vs Scans/DeviceDetail raw).
export interface FormatTimestampOpts {
    relative?: boolean;
}

// Formato absoluto locale-aware (Intl.DateTimeFormat en lugar de hardcoded).
const absoluteFormatter = new Intl.DateTimeFormat("es", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
});

export function formatTimestamp(
    value: string | number | Date | null | undefined,
    opts: FormatTimestampOpts = {},
): string {
    if (value === null || value === undefined) return "—";
    const d = dayjs(value);
    if (!d.isValid()) return "—";
    if (opts.relative) return d.fromNow();
    return absoluteFormatter.format(d.toDate());
}

export function formatRelative(
    value: string | number | Date | null | undefined,
): string {
    return formatTimestamp(value, { relative: true });
}
