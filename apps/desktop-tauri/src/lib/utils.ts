import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/** shadcn/ui className combiner (clsx + tailwind-merge). */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}