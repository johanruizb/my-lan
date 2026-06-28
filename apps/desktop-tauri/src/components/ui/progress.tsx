import { cn } from "@/lib/utils";

export interface ProgressProps {
  value: number; // 0..100
  className?: string;
}

export function Progress({ value, className }: ProgressProps) {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <div
      className={cn("h-2 w-full overflow-hidden rounded-full bg-muted", className)}
      role="progressbar"
      aria-valuenow={clamped}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        className="h-full bg-primary transition-[width] duration-150"
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
}