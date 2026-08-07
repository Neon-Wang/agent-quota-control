export type TranslateValues = Record<string, string | number | boolean | null | undefined>;

const SIMPLE_TOKEN = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g;
const PLURAL_PATTERN =
  /^\{(\w+),\s*plural,\s*((?:=0\s*\{[^{}]*\}|one\s*\{[^{}]*\}|other\s*\{[^{}]*\}\s*)+)\}$/;

export function formatMessage(
  template: string,
  values: TranslateValues = {},
): string {
  const pluralMatch = template.trim().match(PLURAL_PATTERN);
  if (pluralMatch) {
    return formatPlural(pluralMatch[1], pluralMatch[2], values);
  }

  return template.replace(SIMPLE_TOKEN, (_, key: string) => {
    const value = values[key];
    if (value == null) return "";
    return String(value);
  });
}

function formatPlural(
  countKey: string,
  body: string,
  values: TranslateValues,
): string {
  const count = Number(values[countKey] ?? 0);
  const clauses = new Map<string, string>();
  const clausePattern = /(=\d+|one|other)\s*\{([^{}]*)\}/g;
  let match: RegExpExecArray | null;
  while ((match = clausePattern.exec(body))) {
    clauses.set(match[1], match[2]);
  }

  let selected =
    clauses.get(`=${count}`) ??
    (count === 1 ? clauses.get("one") : undefined) ??
    clauses.get("other") ??
    "";

  selected = selected.replace(/#/g, String(count));
  return formatMessage(selected, values);
}

export function createFormatter(locale: string) {
  return {
    dateTime(
      value: Date | number | string,
      options?: Intl.DateTimeFormatOptions,
    ) {
      const date = value instanceof Date ? value : new Date(value);
      return new Intl.DateTimeFormat(locale, options).format(date);
    },
    number(value: number, options?: Intl.NumberFormatOptions) {
      return new Intl.NumberFormat(locale, options).format(value);
    },
    relativeTime(
      value: Date | number,
      options?: Intl.RelativeTimeFormatOptions,
    ) {
      const target =
        typeof value === "number" ? value : value.getTime();
      const deltaSec = Math.round((target - Date.now()) / 1000);
      const abs = Math.abs(deltaSec);
      const rtf = new Intl.RelativeTimeFormat(locale, {
        numeric: "auto",
        ...options,
      });
      if (abs < 60) return rtf.format(deltaSec, "second");
      if (abs < 3600) return rtf.format(Math.round(deltaSec / 60), "minute");
      if (abs < 86400) return rtf.format(Math.round(deltaSec / 3600), "hour");
      return rtf.format(Math.round(deltaSec / 86400), "day");
    },
  };
}
