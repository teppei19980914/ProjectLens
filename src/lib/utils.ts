import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/** Tailwindクラス名の合成・重複解決ヘルパー（shadcn/ui標準パターン）。 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
