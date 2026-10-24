import { clockTime, elapsed, stageLabel } from "../lib/format";
import type { Job } from "../lib/types";

interface QueueListProps {
  jobs: Job[];
  selectedId: number | null;
  onSelect: (job: Job) => void;
  onAction: (id: number, action: string) => void;
}

function statusText(job: Job): string {
  switch (job.status) {
    case "queued":
      return "Waiting";
    case "running":
      return job.fraction !== null && job.fraction !== undefined
        ? `${stageLabel(job.stage)} ${Math.round(job.fraction * 100)}%`
        : stageLabel(job.stage);
    case "failed":
      return job.error?.title ?? "Failed";
    case "stopped":
      return "Stopped";
    case "done":
      return `${job.segments} lines · ${clockTime(job.elapsed_seconds)}`;
  }
}

export function QueueList({ jobs, selectedId, onSelect, onAction }: QueueListProps) {
  const finished = jobs.filter(
    (job) => job.status === "done" || job.status === "failed" || job.status === "stopped",
  ).length;

  return (
    <section className="queue" aria-label="Queue">
      <div className="section-head">
        <h2 className="section-title">Queue</h2>
        {finished > 0 && (
          <button type="button" className="link-button" onClick={() => onAction(0, "clear")}>
            Clear finished
          </button>
        )}
      </div>

      {jobs.length === 0 ? (
        <p className="queue-empty">Nothing queued yet. Add a file above to begin.</p>
      ) : (
        <ul className="job-list">
          {jobs.map((job) => (
            <li key={job.id}>
              <div
                className={`job-row${selectedId === job.id ? " is-active" : ""}`}
                role="button"
                tabIndex={0}
                onClick={() => onSelect(job)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(job);
                  }
                }}
              >
                <span className={`status-dot is-${job.status}`} aria-hidden="true" />
                <div className="job-body">
                  <span className="job-name" title={job.input}>
                    {job.name}
                  </span>
                  <span className="job-meta">
                    {statusText(job)}
                    {job.status === "running" && elapsed(job) ? ` · ${elapsed(job)}` : ""}
                  </span>
                  {job.status === "running" && job.fraction !== null && (
                    <span className="job-bar">
                      <span
                        className="job-bar-fill"
                        style={{ width: `${Math.round((job.fraction ?? 0) * 100)}%` }}
                      />
                    </span>
                  )}
                </div>
                <div className="job-actions">
                  {job.status === "running" && (
                    <button
                      type="button"
                      className="icon-button"
                      aria-label={`Stop ${job.name}`}
                      title="Stop after the current stage"
                      onClick={(event) => {
                        event.stopPropagation();
                        onAction(job.id, "stop");
                      }}
                    >
                      <svg viewBox="0 0 16 16" aria-hidden="true">
                        <rect x="4" y="4" width="8" height="8" rx="1.5" fill="currentColor" />
                      </svg>
                    </button>
                  )}
                  {(job.status === "failed" || job.status === "stopped") && (
                    <button
                      type="button"
                      className="icon-button"
                      aria-label={`Retry ${job.name}`}
                      title="Try again, keeping finished stages"
                      onClick={(event) => {
                        event.stopPropagation();
                        onAction(job.id, "retry");
                      }}
                    >
                      <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path
                          d="M13 8a5 5 0 1 1-1.6-3.7M13 3v3.5H9.5"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="1.5"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                      </svg>
                    </button>
                  )}
                  {job.status !== "running" && (
                    <button
                      type="button"
                      className="icon-button"
                      aria-label={`Remove ${job.name}`}
                      title="Remove from the queue"
                      onClick={(event) => {
                        event.stopPropagation();
                        onAction(job.id, "remove");
                      }}
                    >
                      <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path
                          d="M4 4l8 8M12 4l-8 8"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="1.5"
                          strokeLinecap="round"
                        />
                      </svg>
                    </button>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
