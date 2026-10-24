import type { Environment, Job, Media, Project, Segment } from "./types";

const languages = (
  [
    ["en", "English"],
    ["pt", "Portuguese"],
    ["es", "Spanish"],
    ["fr", "French"],
    ["de", "German"],
    ["it", "Italian"],
    ["nl", "Dutch"],
    ["pl", "Polish"],
    ["ru", "Russian"],
    ["tr", "Turkish"],
    ["ar", "Arabic"],
    ["hi", "Hindi"],
    ["ja", "Japanese"],
    ["ko", "Korean"],
    ["zh", "Chinese"],
    ["uk", "Ukrainian"],
  ] as [string, string][]
).map(([code, name]) => ({ code, name }));

export const demoEnvironment: Environment = {
  version: "0.1.0",
  platform: "windows 11, 12 cores, 15 GB RAM",
  hardware: {
    os_name: "windows",
    os_version: "11",
    arch: "x86_64",
    cpu_cores: 12,
    ram_mb: 15360,
    gpus: [{ vendor: "nvidia", name: "GeForce RTX 3050", memory_mb: 4096 }],
  },
  profile: {
    name: "balanced",
    asr_model: "whisper-base-q5_1",
    threads: 6,
    ram_budget_mb: 2400,
    realtime_factor: 4.1,
    reason: "12 cores and 15 GB of memory",
  },
  tools: [
    {
      name: "ffmpeg",
      found: true,
      location: "C:\\Tools\\ffmpeg.exe",
      version: "ffmpeg version 7.1",
    },
    {
      name: "ffprobe",
      found: true,
      location: "C:\\Tools\\ffprobe.exe",
      version: "ffprobe version 7.1",
    },
    { name: "whisper-cli", found: true, location: "C:\\Tools\\whisper-cli.exe", version: null },
  ],
  models: [
    {
      id: "whisper-tiny-q5_1",
      file: "ggml-tiny-q5_1.bin",
      size: "30 MB",
      size_bytes: 31_000_000,
      ram_mb: 400,
      license: "MIT",
      quality: "fast",
      installed: true,
    },
    {
      id: "whisper-base-q5_1",
      file: "ggml-base-q5_1.bin",
      size: "57 MB",
      size_bytes: 59_707_625,
      ram_mb: 800,
      license: "MIT",
      quality: "balanced",
      installed: true,
    },
    {
      id: "whisper-small-q5_1",
      file: "ggml-small-q5_1.bin",
      size: "182 MB",
      size_bytes: 190_000_000,
      ram_mb: 1100,
      license: "MIT",
      quality: "maximum",
      installed: false,
    },
    {
      id: "whisper-medium-q5_0",
      file: "ggml-medium-q5_0.bin",
      size: "514 MB",
      size_bytes: 539_000_000,
      ram_mb: 2600,
      license: "MIT",
      quality: "maximum",
      installed: false,
    },
  ],
  languages,
  data_dir: "C:\\Users\\you\\AppData\\Local\\Dubbit",
  target_language: "pt",
  translator_ready: true,
  translator_label: "Local model, llama3.2",
  ready: true,
  blockers: [],
};

export const demoMedia: Media = {
  path: "C:\\Videos\\designing-for-everyone.mp4",
  duration: 742.5,
  size_bytes: 214_000_000,
  has_video: true,
  has_audio: true,
  video_codec: "h264",
  width: 1920,
  height: 1080,
  fps: 25,
  audio_codec: "aac",
  sample_rate: 48000,
  channels: 2,
};

const demoText: [number, number, string, string][] = [
  [
    0.0,
    5.4,
    "Designing for everyone starts with admitting what we do not know.",
    "Desenhar para todos começa por admitir o que não sabemos.",
  ],
  [
    5.4,
    11.2,
    "Most software is built for the person who wrote it, in the room where it was written.",
    "A maior parte do software é feita para quem o escreveu, na sala onde foi escrito.",
  ],
  [
    11.2,
    17.6,
    "That is comfortable, and it is also the reason so many tools feel foreign to new users.",
    "Isso é confortável, e é também a razão pela qual tantas ferramentas parecem estranhas a quem chega.",
  ],
  [
    17.6,
    24.1,
    "We tested the same task with twelve people, and only three finished it without help.",
    "Testámos a mesma tarefa com doze pessoas, e só três a concluíram sem ajuda.",
  ],
  [
    24.1,
    30.8,
    "The rest were not careless. The interface never told them what would happen next.",
    "As outras não foram descuidadas. A interface nunca lhes disse o que iria acontecer a seguir.",
  ],
  [
    30.8,
    37.2,
    "So we removed a menu, then another, and put the single next step in plain words.",
    "Por isso retirámos um menu, depois outro, e escrevemos o único passo seguinte por palavras simples.",
  ],
  [
    37.2,
    43.9,
    "Nothing was lost. The work became easier to start, which is what a tool owes the person holding it.",
    "Nada se perdeu. O trabalho tornou-se mais fácil de começar, que é o que uma ferramenta deve a quem a usa.",
  ],
  [
    43.9,
    50.6,
    "If a screen needs a manual, the screen is the problem, not the manual.",
    "Se um ecrã precisa de um manual, o problema é o ecrã, não o manual.",
  ],
  [
    50.6,
    57.3,
    "Thank you for watching, and thank you for building things that other people can use.",
    "Obrigado por assistir, e obrigado por construir coisas que outras pessoas conseguem usar.",
  ],
];

const demoSegments: Segment[] = demoText.map(([start, end, text, translation], index) => ({
  index,
  start,
  end,
  text,
  translation,
}));

export const demoProject: Project = {
  dir: "C:\\Videos\\designing-for-everyone.dubbit",
  name: "designing-for-everyone.dubbit",
  source_path: demoMedia.path,
  source_language: "en",
  target_language: "pt",
  translate: true,
  duration: demoMedia.duration,
  formats: ["srt", "vtt"],
  segments: demoSegments,
  files: ["subtitles/en.srt", "subtitles/en.vtt", "subtitles/pt.srt", "subtitles/pt.vtt"],
  audio_path: null,
};

function now(): number {
  return Math.floor(Date.now() / 1000);
}

export const demoJobs: Job[] = [
  {
    id: 1,
    input: demoMedia.path,
    name: "designing-for-everyone.mp4",
    output: demoProject.dir,
    model: "whisper-base-q5_1",
    source_language: "en",
    target_language: "pt",
    translator: "auto",
    status: "done",
    stage: "done",
    detail: null,
    fraction: null,
    queued_at: now() - 640,
    started_at: now() - 630,
    finished_at: now() - 480,
    elapsed_seconds: 150,
    error: null,
    files: demoProject.files,
    segments: demoSegments.length,
  },
  {
    id: 2,
    input: "C:\\Videos\\field-notes.mp4",
    name: "field-notes.mp4",
    output: "C:\\Videos\\field-notes.dubbit",
    model: "whisper-base-q5_1",
    source_language: null,
    target_language: "es",
    translator: "auto",
    status: "queued",
    stage: null,
    detail: null,
    fraction: null,
    queued_at: now() - 20,
    started_at: null,
    finished_at: null,
    elapsed_seconds: 0,
    error: null,
    files: [],
    segments: 0,
  },
];

export const demoFiles: { path: string; media: Media }[] = [
  { path: demoMedia.path, media: demoMedia },
  {
    path: "C:\\Videos\\field-notes.mp4",
    media: {
      ...demoMedia,
      path: "C:\\Videos\\field-notes.mp4",
      duration: 318.2,
      size_bytes: 96_000_000,
    },
  },
];

export function isDemo(): boolean {
  return !("__TAURI_INTERNALS__" in window);
}
