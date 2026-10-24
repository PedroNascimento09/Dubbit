import { useCallback, useEffect, useMemo, useState } from "react";

import { Brand } from "./components/Brand";
import { JobProgress } from "./components/JobProgress";
import { NewJobForm } from "./components/NewJobForm";
import { QueueList } from "./components/QueueList";
import { ReviewEditor } from "./components/ReviewEditor";
import { desktop, type ModelProgress } from "./lib/bridge";
import type { Environment, Job, NewJob, Project } from "./lib/types";

type Theme = "dark" | "light";

function initialTheme(): Theme {
  const stored = window.localStorage.getItem("dubbit.theme");
  if (stored === "dark" || stored === "light") return stored;
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function readError(error: unknown): string {
  if (error && typeof error === "object" && "title" in error) {
    const view = error as { title?: string; detail?: string | null; hint?: string | null };
    return [view.title, view.detail, view.hint].filter(Boolean).join(" — ");
  }
  return error instanceof Error ? error.message : String(error);
}

export function App() {
  const [environment, setEnvironment] = useState<Environment | null>(null);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [project, setProject] = useState<Project | null>(null);
  const [fatal, setFatal] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [modelProgress, setModelProgress] = useState<ModelProgress | null>(null);
  const [theme, setTheme] = useState<Theme>(initialTheme);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("dubbit.theme", theme);
  }, [theme]);

  useEffect(() => {
    let alive = true;
    void desktop
      .loadEnvironment()
      .then((value) => {
        if (alive) setEnvironment(value);
      })
      .catch((error) => {
        if (alive) setFatal(readError(error));
      });
    void desktop.loadJobs().then((value) => {
      if (alive) setJobs(value);
    });
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void desktop
      .watchJobs((value) => setJobs(value))
      .then((unlisten) => {
        dispose = unlisten;
      });
    return () => dispose?.();
  }, []);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void desktop
      .watchModel((value) => setModelProgress(value))
      .then((unlisten) => {
        dispose = unlisten;
      });
    return () => dispose?.();
  }, []);

  const selected = useMemo(
    () => jobs.find((job) => job.id === selectedId) ?? null,
    [jobs, selectedId],
  );

  useEffect(() => {
    if (!selected || selected.status !== "done") return;
    if (project?.dir === selected.output) return;
    let alive = true;
    void desktop
      .openProject(selected.output)
      .then((value) => {
        if (alive) setProject(value);
      })
      .catch((error) => {
        if (alive) setFatal(readError(error));
      });
    return () => {
      alive = false;
    };
  }, [selected, project?.dir]);

  const create = useCallback(async (requests: NewJob[]) => {
    setBusy(true);
    try {
      const created = await desktop.enqueue(requests);
      const first = created[0];
      if (first) {
        setProject(null);
        setSelectedId(first.id);
      }
      setJobs(await desktop.loadJobs());
    } catch (error) {
      setFatal(readError(error));
    } finally {
      setBusy(false);
    }
  }, []);

  const action = useCallback(
    async (id: number, name: string) => {
      try {
        const next = await desktop.jobAction(id, name);
        setJobs(next);
        if (selectedId !== null && !next.some((job) => job.id === selectedId)) {
          setSelectedId(null);
          setProject(null);
        }
      } catch (error) {
        setFatal(readError(error));
      }
    },
    [selectedId],
  );

  const installModel = useCallback(async (id: string) => {
    try {
      await desktop.installModel(id);
      const refreshed = await desktop.loadEnvironment();
      setEnvironment(refreshed);
    } catch (error) {
      setFatal(readError(error));
    }
  }, []);

  const reveal = useCallback((path: string) => {
    void desktop.reveal(path);
  }, []);

  return (
    <div className="app">
      <aside className="rail">
        <Brand version={environment?.version ?? "0.1.0"} demo={desktop.isDemo()} />

        {environment ? (
          <NewJobForm
            environment={environment}
            busy={busy}
            modelProgress={modelProgress}
            onCreate={create}
            onInstallModel={installModel}
          />
        ) : (
          <div className="rail-loading">
            <span className="skeleton-line" />
            <span className="skeleton-block" />
            <span className="skeleton-line short" />
            <span className="skeleton-block" />
          </div>
        )}

        <QueueList
          jobs={jobs}
          selectedId={selectedId}
          onSelect={(job) => {
            setSelectedId(job.id);
            if (job.status !== "done" || project?.dir !== job.output) setProject(null);
          }}
          onAction={(id, name) => void action(id, name)}
        />

        <footer className="rail-foot">
          <div className="machine">
            <span className="machine-line">
              {environment?.platform ?? "Reading this computer"}
            </span>
            <span className="machine-line muted">
              {environment
                ? `${environment.profile.name} profile · ${environment.profile.asr_model.replace("whisper-", "")}`
                : " "}
            </span>
          </div>
          <button
            type="button"
            className="theme-switch"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} appearance`}
            title={`Switch to ${theme === "dark" ? "light" : "dark"} appearance`}
          >
            {theme === "dark" ? (
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <circle cx="8" cy="8" r="3.2" fill="currentColor" />
                <path
                  d="M8 1.4v1.6M8 13v1.6M1.4 8h1.6M13 8h1.6M3.3 3.3l1.1 1.1M11.6 11.6l1.1 1.1M12.7 3.3l-1.1 1.1M4.4 11.6l-1.1 1.1"
                  stroke="currentColor"
                  strokeWidth="1.4"
                  strokeLinecap="round"
                />
              </svg>
            ) : (
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <path
                  d="M13.2 9.6A5.4 5.4 0 0 1 6.4 2.8a5.6 5.6 0 1 0 6.8 6.8z"
                  fill="currentColor"
                />
              </svg>
            )}
          </button>
        </footer>
      </aside>

      <main className="stage">
        {fatal && (
          <div className="alert is-error" role="alert">
            <span className="alert-code">Something went wrong</span>
            <p className="alert-title">{fatal}</p>
            <button type="button" className="button ghost" onClick={() => setFatal(null)}>
              Dismiss
            </button>
          </div>
        )}

        {project && selected ? (
          <ReviewEditor
            project={project}
            onSaved={setProject}
            onReveal={() => reveal(project.dir)}
          />
        ) : selected ? (
          <JobProgress
            job={selected}
            onRetry={() => void action(selected.id, "retry")}
            onOpen={() => {
              void desktop.openProject(selected.output).then(setProject);
            }}
            onReveal={() => reveal(selected.output)}
          />
        ) : (
          <section className="empty">
            <div className="empty-art" aria-hidden="true">
              <span className="wave-bar" style={{ height: "22%" }} />
              <span className="wave-bar" style={{ height: "48%" }} />
              <span className="wave-bar is-tall" style={{ height: "78%" }} />
              <span className="wave-bar" style={{ height: "56%" }} />
              <span className="wave-bar" style={{ height: "32%" }} />
              <span className="wave-bar" style={{ height: "18%" }} />
            </div>
            <p className="eyebrow">Subtitles, locally</p>
            <h1 className="empty-title">
              Your videos get subtitles.
              <br />
              Nothing leaves this computer.
            </h1>
            <p className="empty-text">
              Drop a file, choose the language, and Dubbit writes the subtitles with the
              processor you already have. No account, no queue on someone else's server.
            </p>
            <ol className="steps">
              <li className="step">
                <span className="step-number">1</span>
                <div>
                  <span className="step-title">Add a video</span>
                  <span className="step-text">
                    MP4, MKV, MOV or audio. Long files are fine.
                  </span>
                </div>
              </li>
              <li className="step">
                <span className="step-number">2</span>
                <div>
                  <span className="step-title">Let it listen</span>
                  <span className="step-text">
                    Speech is recognised on the processor, stage by stage.
                  </span>
                </div>
              </li>
              <li className="step">
                <span className="step-number">3</span>
                <div>
                  <span className="step-title">Review and export</span>
                  <span className="step-text">
                    Fix any line, then save SRT and VTT for your editor.
                  </span>
                </div>
              </li>
            </ol>
            {environment && (
              <p className="empty-foot">
                {environment.profile.name} profile · {environment.hardware.cpu_cores} cores ·{" "}
                {Math.round(environment.hardware.ram_mb / 1024)} GB RAM
                {environment.translator_label ? ` · ${environment.translator_label}` : ""}
              </p>
            )}
          </section>
        )}
      </main>
    </div>
  );
}
