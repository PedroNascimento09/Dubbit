import { useEffect, useMemo, useRef, useState } from "react";

import { desktop, type ModelProgress } from "../lib/bridge";
import { baseName, bytes, clockTime } from "../lib/format";
import type { Environment, NewJob, PickedFile } from "../lib/types";

interface NewJobFormProps {
  environment: Environment;
  busy: boolean;
  modelProgress: ModelProgress | null;
  onCreate: (requests: NewJob[]) => Promise<void>;
  onInstallModel: (id: string) => Promise<void>;
}

function modelLabel(id: string, quality: string): string {
  const name = id
    .replace("whisper-", "")
    .replace(/-q5_[01]$/, "")
    .replace(/-q8_0$/, "");
  const title = name.charAt(0).toUpperCase() + name.slice(1);
  const rank = quality.charAt(0).toUpperCase() + quality.slice(1);
  return `${rank} · ${title}`;
}

export function NewJobForm({
  environment,
  busy,
  modelProgress,
  onCreate,
  onInstallModel,
}: NewJobFormProps) {
  const [files, setFiles] = useState<PickedFile[]>([]);
  const [source, setSource] = useState("");
  const [target, setTarget] = useState(
    environment.translator_ready ? environment.target_language : "",
  );
  const [model, setModel] = useState(environment.profile.asr_model);
  const [dragging, setDragging] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const add = useMemo(
    () =>
      async (paths: string[]): Promise<void> => {
        const known = new Set(files.map((file) => file.path));
        const incoming = paths.filter((path) => !known.has(path));
        if (incoming.length === 0) return;
        const inspected: PickedFile[] = [];
        for (const path of incoming) {
          try {
            inspected.push({ path, media: await desktop.inspect(path) });
          } catch (error) {
            setNotice(error instanceof Error ? error.message : String(error));
          }
        }
        if (mounted.current && inspected.length > 0) {
          setFiles((current) => [...current, ...inspected]);
          setNotice(null);
        }
      },
    [files],
  );

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void desktop
      .watchDrop((paths) => {
        void add(paths);
      })
      .then((unlisten) => {
        dispose = unlisten;
      });
    return () => dispose?.();
  }, [add]);

  const selectedModel = environment.models.find((entry) => entry.id === model);
  const downloading = modelProgress && modelProgress.id === model ? modelProgress : null;
  const sameLanguage =
    source !== "" && target !== "" && source.split("-")[0] === target.split("-")[0];
  const duration = files.reduce((total, file) => total + file.media.duration, 0);
  const blocked = !environment.ready;

  const submit = async (): Promise<void> => {
    if (files.length === 0 || busy || blocked) return;
    const requests: NewJob[] = files.map((file) => ({
      input: file.path,
      model,
      source_language: source === "" ? null : source,
      target_language: target === "" ? null : target,
      translator: "auto",
    }));
    await onCreate(requests);
    if (mounted.current) setFiles([]);
  };

  const install = async (): Promise<void> => {
    if (!selectedModel || selectedModel.installed) return;
    setInstalling(selectedModel.id);
    try {
      await onInstallModel(selectedModel.id);
    } finally {
      if (mounted.current) setInstalling(null);
    }
  };

  return (
    <section className="new-job" aria-label="New subtitles">
      <div className="section-head">
        <h2 className="section-title">New subtitles</h2>
        {files.length > 0 && (
          <span className="section-note">
            {files.length} {files.length === 1 ? "file" : "files"} · {clockTime(duration)}
          </span>
        )}
      </div>

      {blocked && (
        <div className="alert" role="alert">
          <span className="alert-code">Missing tools</span>
          <p className="alert-title">
            Dubbit needs {environment.blockers.join(" and ")} to read your videos.
          </p>
          <p className="alert-hint">
            Install them and reopen the app. The command line guide explains how.
          </p>
        </div>
      )}

      <div
        className={`dropzone${dragging ? " is-active" : ""}`}
        role="button"
        tabIndex={0}
        onClick={() => {
          void desktop.pickMedia().then((paths) => add(paths));
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            void desktop.pickMedia().then((paths) => add(paths));
          }
        }}
        onDragEnter={() => setDragging(true)}
        onDragLeave={() => setDragging(false)}
        onDragOver={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDrop={(event) => {
          event.preventDefault();
          setDragging(false);
        }}
      >
        <svg className="dropzone-icon" viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M12 16V4m0 0L7.5 8.5M12 4l4.5 4.5M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.6"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
        <span className="dropzone-title">Drop a video or audio file</span>
        <span className="dropzone-hint">or click to choose · MP4, MKV, MOV, MP3, WAV</span>
      </div>

      {files.length > 0 && (
        <ul className="file-list">
          {files.map((file) => (
            <li className="file-chip" key={file.path}>
              <div className="file-chip-body">
                <span className="file-chip-name" title={file.path}>
                  {baseName(file.path)}
                </span>
                <span className="file-chip-meta">
                  {clockTime(file.media.duration)} · {bytes(file.media.size_bytes)}
                  {file.media.width && file.media.height
                    ? ` · ${file.media.width}×${file.media.height}`
                    : ""}
                </span>
              </div>
              <button
                type="button"
                className="icon-button"
                aria-label={`Remove ${baseName(file.path)}`}
                onClick={() =>
                  setFiles((current) => current.filter((entry) => entry.path !== file.path))
                }
              >
                <svg viewBox="0 0 16 16" aria-hidden="true">
                  <path
                    d="M4 4l8 8M12 4l-8 8"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.6"
                    strokeLinecap="round"
                  />
                </svg>
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="fields">
        <label className="field">
          <span className="field-label">Spoken language</span>
          <select
            className="field-select"
            value={source}
            onChange={(event) => setSource(event.target.value)}
          >
            <option value="">Detect automatically</option>
            {environment.languages.map((language) => (
              <option key={language.code} value={language.code}>
                {language.name}
              </option>
            ))}
          </select>
        </label>

        <label className="field">
          <span className="field-label">Subtitles in</span>
          <select
            className="field-select"
            value={target}
            disabled={!environment.translator_ready}
            onChange={(event) => setTarget(event.target.value)}
          >
            <option value="">Same language only</option>
            {environment.languages.map((language) => (
              <option key={language.code} value={language.code}>
                {language.name}
              </option>
            ))}
          </select>
          {!environment.translator_ready && (
            <span className="field-hint">
              Translation needs a local model. Until one is set up, Dubbit writes subtitles in
              the spoken language.
            </span>
          )}
          {sameLanguage && (
            <span className="field-hint is-warning">
              Choose a different language from the spoken one.
            </span>
          )}
        </label>
      </div>

      <label className="field">
        <span className="field-label">Speech model</span>
        <select
          className="field-select"
          value={model}
          onChange={(event) => setModel(event.target.value)}
        >
          {environment.models.map((entry) => (
            <option key={entry.id} value={entry.id}>
              {modelLabel(entry.id, entry.quality)} · {entry.size}
              {entry.installed ? "" : " · download"}
            </option>
          ))}
        </select>
        {selectedModel && !selectedModel.installed && !downloading && (
          <span className="field-hint">
            Downloads once, {selectedModel.size}. {selectedModel.ram_mb} MB of memory while
            running.{" "}
            <button
              type="button"
              className="link-button"
              disabled={installing !== null}
              onClick={() => void install()}
            >
              Download now
            </button>
          </span>
        )}
        {downloading && (
          <span className="field-hint">
            Downloading {modelProgress?.detail}
            <span className="inline-bar">
              <span
                className="inline-bar-fill"
                style={{ width: `${Math.round((downloading.fraction ?? 0) * 100)}%` }}
              />
            </span>
          </span>
        )}
      </label>

      {notice && (
        <div className="alert" role="alert">
          <span className="alert-code">Could not read the file</span>
          <p className="alert-title">{notice}</p>
        </div>
      )}

      <button
        type="button"
        className="button primary block"
        disabled={files.length === 0 || busy || blocked || sameLanguage}
        onClick={() => void submit()}
      >
        {busy ? "Adding to the queue" : "Create subtitles"}
      </button>
      <p className="form-footnote">
        Runs on this computer. No account, no upload, nothing leaves the machine.
      </p>
    </section>
  );
}
