import { useEffect, useState } from "react";

import { clockTime } from "../lib/format";
import type { Job } from "../lib/types";

interface JobProgressProps {
  job: Job;
  onRetry: () => void;
  onOpen: () => void;
  onReveal: () => void;
}

const STAGE_ORDER = ["ingest", "transcribe", "translate", "subtitles"] as const;

const STAGE_TEXT: Record<string, { title: string; detail: string }> = {
  ingest: {
    title: "Extract audio",
    detail: "A mono track at 16 kHz, kept inside the project folder.",
  },
  transcribe: {
    title: "Recognise speech",
    detail: "Whisper runs on the processor, never on a server.",
  },
  translate: { title: "Translate", detail: "One line at a time, with your glossary applied." },
  subtitles: { title: "Write subtitles", detail: "SRT and VTT, in every language of the run." },
};

export function JobProgress({ job, onRetry, onOpen, onReveal }: JobProgressProps) {
  const [liveSeconds, setLiveSeconds] = useState(0);

  useEffect(() => {
    if (job.status !== "running" || !job.started_at) return;
    const started = job.started_at;
    const timer = window.setInterval(() => {
      setLiveSeconds(Math.max(0, Date.now() / 1000 - started));
    }, 1000);
    return () => window.clearInterval(timer);
  }, [job.status, job.started_at]);

  const order: string[] = job.target_language
    ? [...STAGE_ORDER]
    : STAGE_ORDER.filter((stage) => stage !== "translate");
  const current = job.stage ? order.indexOf(job.stage) : -1;
  const seconds = job.finished_at ? job.elapsed_seconds : liveSeconds;

  return (
    <div className="progress">
      <header className="stage-head">
        <div>
          <p className="eyebrow">
            {job.status === "running"
              ? "Working"
              : job.status === "queued"
                ? "In the queue"
                : job.status === "done"
                  ? "Finished"
                  : job.status === "stopped"
                    ? "Stopped"
                    : "Failed"}
          </p>
          <h1 className="stage-title">{job.name}</h1>
          <p className="stage-sub">
            {job.source_language ? job.source_language.toUpperCase() : "Detected language"}
            {job.target_language ? ` → ${job.target_language.toUpperCase()}` : ""}
            {seconds > 0 ? ` · ${clockTime(seconds)}` : ""}
          </p>
        </div>
        {job.status === "done" && (
          <div className="stage-actions">
            <button type="button" className="button ghost" onClick={onReveal}>
              Open folder
            </button>
            <button type="button" className="button primary" onClick={onOpen}>
              Review subtitles
            </button>
          </div>
        )}
      </header>

      {job.status === "running" && (
        <ol className="stages">
          {order.map((stage, index) => {
            const state =
              current > index || job.status === "done"
                ? "is-done"
                : current === index
                  ? "is-active"
                  : "is-pending";
            return (
              <li className={`stage-item ${state}`} key={stage}>
                <span className="stage-marker" aria-hidden="true">
                  {state === "is-done" ? (
                    <svg viewBox="0 0 16 16">
                      <path
                        d="M3.5 8.5l3 3 6-7"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="1.8"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                  ) : (
                    <span className="stage-pip" />
                  )}
                </span>
                <div className="stage-copy">
                  <span className="stage-name">{STAGE_TEXT[stage]?.title ?? stage}</span>
                  <span className="stage-detail">
                    {current === index && job.detail
                      ? job.detail
                      : (STAGE_TEXT[stage]?.detail ?? "")}
                  </span>
                </div>
              </li>
            );
          })}
        </ol>
      )}

      {job.status === "running" && current < 0 && (
        <div className="preparing">
          <span className="preparing-label">
            {job.stage === "model" ? "Downloading the speech model" : "Preparing the project"}
          </span>
          <p className="preparing-detail">
            {job.detail ?? "Reading the file and checking the disk."}
          </p>
          {job.fraction !== null && (
            <span className="meter">
              <span
                className="meter-fill"
                style={{ width: `${Math.round((job.fraction ?? 0) * 100)}%` }}
              />
            </span>
          )}
        </div>
      )}

      {job.status === "queued" && (
        <p className="queued-note">
          Waiting for the job above it. You can add more files; they run one at a time.
        </p>
      )}

      {job.status === "failed" && job.error && (
        <div className="alert is-error" role="alert">
          <span className="alert-code">{job.error.code}</span>
          <p className="alert-title">{job.error.title}</p>
          {job.error.detail && <p className="alert-detail">{job.error.detail}</p>}
          {job.error.hint && <p className="alert-hint">{job.error.hint}</p>}
          <button type="button" className="button ghost" onClick={onRetry}>
            Try again
          </button>
        </div>
      )}

      {job.status === "stopped" && (
        <div className="alert">
          <span className="alert-code">Stopped</span>
          <p className="alert-title">The run was stopped after the last finished stage.</p>
          <p className="alert-hint">
            Finishing stages are kept, so a retry continues where it stopped.
          </p>
          <button type="button" className="button ghost" onClick={onRetry}>
            Continue
          </button>
        </div>
      )}

      <footer className="progress-foot">
        <span>Project folder</span>
        <code className="path">{job.output}</code>
      </footer>
    </div>
  );
}
