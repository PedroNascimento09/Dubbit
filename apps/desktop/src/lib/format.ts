const STAGE_LABELS: Record<string, string> = {
  prepare: "Preparing",
  model: "Downloading model",
  ingest: "Extracting audio",
  transcribe: "Recognising speech",
  translate: "Translating",
  subtitles: "Writing subtitles",
  done: "Ready",
};

export function stageLabel(stage: string | null): string {
  if (!stage) return "Waiting";
  return STAGE_LABELS[stage] ?? stage;
}

export function clockTime(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(rest).padStart(2, "0")}`;
  }
  return `${minutes}:${String(rest).padStart(2, "0")}`;
}

export function cueTime(seconds: number): string {
  const total = Math.max(0, Math.round(seconds * 1000));
  const minutes = Math.floor(total / 60000);
  const rest = Math.floor((total % 60000) / 1000);
  const millis = total % 1000;
  return `${String(minutes).padStart(2, "0")}:${String(rest).padStart(2, "0")}.${String(millis).padStart(3, "0")}`;
}

export function bytes(size: number): string {
  const megabytes = size / (1024 * 1024);
  if (megabytes >= 1024) return `${(megabytes / 1024).toFixed(1)} GB`;
  if (megabytes >= 1) return `${Math.round(megabytes)} MB`;
  return `${Math.max(1, Math.round(size / 1024))} KB`;
}

export function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

export function elapsed(job: {
  started_at: number | null;
  finished_at: number | null;
  elapsed_seconds: number;
}): string {
  if (!job.started_at) return "";
  const end = job.finished_at ? job.finished_at * 1000 : Date.now();
  const seconds = job.finished_at
    ? job.elapsed_seconds
    : Math.max(0, (end - job.started_at * 1000) / 1000);
  return clockTime(seconds);
}

export function languageName(
  languages: { code: string; name: string }[],
  code: string | null,
): string {
  if (!code) return "Detected automatically";
  return languages.find((language) => language.code === code)?.name ?? code.toUpperCase();
}
