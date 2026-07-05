import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import "dayjs/locale/es";

dayjs.extend(relativeTime);
dayjs.locale("es");

// Helper unificado de timestamps (elimina drift Devices.fromNow vs Scans/DeviceDetail raw).
export interface FormatTimestampOpts {
    relative?: boolean;
}

export function formatTimestamp(
    value: string | number | Date | null | undefined,
    opts: FormatTimestampOpts = {},
): string {
    if (value === null || value === undefined || value === "") return "—";
    const d = dayjs(value);
    if (!d.isValid()) return "—";
    return opts.relative ? d.fromNow() : d.format("YYYY-MM-DD HH:mm");
}

export function formatRelative(
    value: string | number | Date | null | undefined,
): string {
    return formatTimestamp(value, { relative: true });
}
