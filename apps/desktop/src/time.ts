export function normalizeTimeInput(value: string, defaultTime?: string): string | null {
  const input = value.trim();
  if (input === "") {
    return "";
  }

  const defaultMinutes = defaultTime?.split(":")[1]?.padStart(2, "0") ?? "00";

  const completeTime = /^([01]?\d|2[0-3]):([0-9]|[0-5]\d)$/.exec(input);
  if (completeTime) {
    return `${completeTime[1].padStart(2, "0")}:${completeTime[2].padStart(2, "0")}`;
  }

  const hourOnly = /^([01]?\d|2[0-3])$/.exec(input);
  if (hourOnly) {
    return `${hourOnly[1].padStart(2, "0")}:${defaultMinutes}`;
  }

  return null;
}
