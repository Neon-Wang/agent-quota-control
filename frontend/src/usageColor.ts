/** Shared utilization color language for meters, badges accents, and trend charts. */

export type UsageTone = "ok" | "warn" | "danger";

/** Match progress meters: green <70, amber ≥70, red ≥90. */
export function usageTone(utilization: number): UsageTone {
  if (utilization >= 90) return "danger";
  if (utilization >= 70) return "warn";
  return "ok";
}

export function meterFillClass(utilization: number): string {
  const tone = usageTone(utilization);
  if (tone === "danger") return "meter-fill danger";
  if (tone === "warn") return "meter-fill warn";
  return "meter-fill ok-fill";
}

export function usageToneClass(utilization: number): string {
  return `usage-tone-${usageTone(utilization)}`;
}

/** Vertical stroke/area stops as % of the 0→100 utilization axis. */
export const USAGE_GRADIENT_STOPS: Array<{ offset: number; tone: UsageTone }> = [
  { offset: 0, tone: "ok" },
  { offset: 62, tone: "ok" },
  { offset: 70, tone: "warn" },
  { offset: 88, tone: "warn" },
  { offset: 90, tone: "danger" },
  { offset: 100, tone: "danger" },
];
