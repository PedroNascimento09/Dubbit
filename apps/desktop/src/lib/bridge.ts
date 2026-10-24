import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

import { demoEnvironment, demoFiles, demoJobs, demoMedia, demoProject, isDemo } from "./demo";
import type { Environment, Job, Media, NewJob, Project, Segment } from "./types";

export interface ModelProgress {
  id: string;
  fraction: number;
  detail: string;
}

type Listener<T> = (value: T) => void;

const demoState: { jobs: Job[] } = { jobs: demoJobs.map((job) => ({ ...job })) };
let demoNextId = demoJobs.length + 1;
const demoJobListeners = new Set<Listener<Job[]>>();
const demoModelListeners = new Set<Listener<ModelProgress>>();

function demoEmitJobs(): void {
  const snapshot = demoState.jobs.map((job) => ({ ...job }));
  demoJobListeners.forEach((listener) => listener(snapshot));
}

function demoProgress(job: Job): void {
  const jobId = job.id;
  const stages: [string, number][] = [
    ["ingest", 1400],
    ["transcribe", 3200],
    ["translate", 1800],
    ["subtitles", 900],
  ];
  let cursor = 0;
  const advance = (): void => {
    const current = demoState.jobs.find((entry) => entry.id === jobId);
    if (!current || current.status !== "running") return;
    if (cursor >= stages.length) {
      current.status = "done";
      current.stage = "done";
      current.finished_at = Math.floor(Date.now() / 1000);
      current.elapsed_seconds = 42;
      current.files = demoProject.files;
      current.segments = demoProject.segments.length;
      demoEmitJobs();
      return;
    }
    const [stage, delay] = stages[cursor]!;
    cursor += 1;
    current.stage = stage;
    current.started_at = current.started_at ?? Math.floor(Date.now() / 1000);
    demoEmitJobs();
    window.setTimeout(advance, delay);
  };
  window.setTimeout(advance, 600);
}

export const desktop = {
  isDemo,

  async loadEnvironment(): Promise<Environment> {
    if (isDemo()) return demoEnvironment;
    return invoke<Environment>("environment");
  },

  async loadJobs(): Promise<Job[]> {
    if (isDemo()) return demoState.jobs.map((job) => ({ ...job }));
    return invoke<Job[]>("jobs");
  },

  async inspect(path: string): Promise<Media> {
    if (isDemo()) {
      return demoFiles.find((file) => file.path === path)?.media ?? { ...demoMedia, path };
    }
    return invoke<Media>("inspect", { path });
  },

  async enqueue(requests: NewJob[]): Promise<Job[]> {
    if (isDemo()) {
      const created: Job[] = requests.map((request, offset) => ({
        id: demoNextId + offset,
        input: request.input,
        name: request.input.split(/[\\/]/).pop() ?? request.input,
        output: demoProject.dir,
        model: request.model,
        source_language: request.source_language,
        target_language: request.target_language,
        translator: request.translator,
        status: "queued",
        stage: null,
        detail: null,
        fraction: null,
        queued_at: Math.floor(Date.now() / 1000),
        started_at: null,
        finished_at: null,
        elapsed_seconds: 0,
        error: null,
        files: [],
        segments: 0,
      }));
      demoNextId += created.length;
      demoState.jobs.push(...created);
      const running = created[0];
      if (running) {
        running.status = "running";
        running.stage = "prepare";
        running.started_at = Math.floor(Date.now() / 1000);
        demoProgress(running);
      }
      demoEmitJobs();
      return created;
    }
    return invoke<Job[]>("enqueue", { requests });
  },

  async jobAction(id: number, action: string): Promise<Job[]> {
    if (isDemo()) {
      if (action === "remove" || action === "clear") {
        demoState.jobs =
          action === "remove"
            ? demoState.jobs.filter((job) => job.id !== id)
            : demoState.jobs.filter(
                (job) => job.status === "running" || job.status === "queued",
              );
      }
      if (action === "retry") {
        const job = demoState.jobs.find((entry) => entry.id === id);
        if (job) {
          job.status = "running";
          job.stage = "prepare";
          job.error = null;
          job.started_at = Math.floor(Date.now() / 1000);
          demoProgress(job);
        }
      }
      demoEmitJobs();
      return demoState.jobs.map((job) => ({ ...job }));
    }
    return invoke<Job[]>("job_action", { id, action });
  },

  async openProject(dir: string): Promise<Project> {
    if (isDemo()) return demoProject;
    return invoke<Project>("open_project", { dir });
  },

  async saveReview(dir: string, segments: Segment[]): Promise<Project> {
    if (isDemo()) {
      demoProject.segments = segments;
      await new Promise((resolve) => window.setTimeout(resolve, 350));
      return demoProject;
    }
    return invoke<Project>("save_review", { dir, segments });
  },

  async installModel(id: string): Promise<void> {
    if (isDemo()) {
      const model = demoEnvironment.models.find((entry) => entry.id === id);
      if (!model) return;
      for (let step = 0; step <= 100; step += 10) {
        await new Promise((resolve) => window.setTimeout(resolve, 140));
        demoModelListeners.forEach((listener) =>
          listener({
            id,
            fraction: step / 100,
            detail: `${Math.round((model.size_bytes * step) / 100 / 1024 / 1024)} MB of ${model.size}`,
          }),
        );
      }
      model.installed = true;
      return;
    }
    await invoke<void>("install_model", { id });
  },

  async pickMedia(): Promise<string[]> {
    if (isDemo()) return [demoFiles[0]!.path];
    const picked = await open({
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Video and audio",
          extensions: [
            "mp4",
            "mkv",
            "mov",
            "webm",
            "avi",
            "m4v",
            "mp3",
            "wav",
            "m4a",
            "flac",
            "ogg",
            "opus",
          ],
        },
      ],
    });
    if (!picked) return [];
    return Array.isArray(picked) ? picked : [picked];
  },

  async reveal(path: string): Promise<void> {
    if (isDemo()) return;
    await revealItemInDir(path);
  },

  audioUrl(path: string | null): string | null {
    if (!path) return null;
    if (isDemo()) return null;
    return convertFileSrc(path);
  },

  async watchJobs(listener: Listener<Job[]>): Promise<() => void> {
    if (isDemo()) {
      demoJobListeners.add(listener);
      return () => demoJobListeners.delete(listener);
    }
    return listen<Job[]>("jobs", (event) => listener(event.payload));
  },

  async watchModel(listener: Listener<ModelProgress>): Promise<() => void> {
    if (isDemo()) {
      demoModelListeners.add(listener);
      return () => demoModelListeners.delete(listener);
    }
    return listen<ModelProgress>("model", (event) => listener(event.payload));
  },

  async watchDrop(listener: (paths: string[]) => void): Promise<() => void> {
    if (isDemo()) {
      const handler = (event: DragEvent): void => {
        event.preventDefault();
        const files = Array.from(event.dataTransfer?.files ?? []);
        const paths = files.map(
          (file) =>
            demoFiles.find((entry) => entry.path.endsWith(file.name))?.path ?? demoMedia.path,
        );
        if (paths.length > 0) listener(paths);
      };
      window.addEventListener("drop", handler);
      const prevent = (event: DragEvent): void => event.preventDefault();
      window.addEventListener("dragover", prevent);
      return () => {
        window.removeEventListener("drop", handler);
        window.removeEventListener("dragover", prevent);
      };
    }
    return getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") listener(event.payload.paths);
    });
  },
};
