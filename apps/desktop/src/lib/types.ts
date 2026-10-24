export type JobStatus = "queued" | "running" | "done" | "failed" | "stopped";

export type Stage =
  "prepare" | "model" | "ingest" | "transcribe" | "translate" | "subtitles" | "done";

export interface Language {
  code: string;
  name: string;
}

export interface Model {
  id: string;
  file: string;
  size: string;
  size_bytes: number;
  ram_mb: number;
  license: string;
  quality: string;
  installed: boolean;
}

export interface Tool {
  name: string;
  found: boolean;
  location: string | null;
  version: string | null;
}

export interface Profile {
  name: string;
  asr_model: string;
  threads: number;
  ram_budget_mb: number;
  realtime_factor: number;
  reason: string;
}

export interface Hardware {
  os_name: string;
  os_version: string;
  arch: string;
  cpu_cores: number;
  ram_mb: number;
  gpus: { vendor: string; name: string; memory_mb: number | null }[];
}

export interface Environment {
  version: string;
  platform: string;
  hardware: Hardware;
  profile: Profile;
  tools: Tool[];
  models: Model[];
  languages: Language[];
  data_dir: string;
  target_language: string;
  translator_ready: boolean;
  translator_label: string | null;
  ready: boolean;
  blockers: string[];
}

export interface Media {
  path: string;
  duration: number;
  size_bytes: number;
  has_video: boolean;
  has_audio: boolean;
  video_codec: string | null;
  width: number | null;
  height: number | null;
  fps: number | null;
  audio_codec: string | null;
  sample_rate: number | null;
  channels: number | null;
}

export interface DubbitError {
  code: string;
  title: string;
  detail: string | null;
  hint: string | null;
}

export interface Job {
  id: number;
  input: string;
  name: string;
  output: string;
  model: string;
  source_language: string | null;
  target_language: string | null;
  translator: string;
  status: JobStatus;
  stage: Stage | string | null;
  detail: string | null;
  fraction: number | null;
  queued_at: number;
  started_at: number | null;
  finished_at: number | null;
  elapsed_seconds: number;
  error: DubbitError | null;
  files: string[];
  segments: number;
}

export interface NewJob {
  input: string;
  model: string;
  source_language: string | null;
  target_language: string | null;
  translator: string;
}

export interface Segment {
  index: number;
  start: number;
  end: number;
  text: string;
  translation: string;
}

export interface Project {
  dir: string;
  name: string;
  source_path: string;
  source_language: string | null;
  target_language: string | null;
  translate: boolean;
  duration: number;
  formats: string[];
  segments: Segment[];
  files: string[];
  audio_path: string | null;
}

export interface PickedFile {
  path: string;
  media: Media;
}
