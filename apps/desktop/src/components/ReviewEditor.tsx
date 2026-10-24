import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { desktop } from "../lib/bridge";
import { baseName, cueTime } from "../lib/format";
import type { Project, Segment } from "../lib/types";

interface ReviewEditorProps {
  project: Project;
  onSaved: (project: Project) => void;
  onReveal: () => void;
}

export function ReviewEditor({ project, onSaved, onReveal }: ReviewEditorProps) {
  const [segments, setSegments] = useState<Segment[]>(project.segments);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [playing, setPlaying] = useState<number | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const audioSrc = useMemo(() => desktop.audioUrl(project.audio_path), [project.audio_path]);

  const save = useCallback(async () => {
    if (saving) return;
    setSaving(true);
    setMessage(null);
    try {
      const saved = await desktop.saveReview(project.dir, segments);
      onSaved(saved);
      setDirty(false);
      setMessage("Saved and exported");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }, [onSaved, project.dir, saving, segments]);

  useEffect(() => {
    const handler = (event: KeyboardEvent): void => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        void save();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [save]);

  const update = (index: number, value: string): void => {
    setSegments((current) =>
      current.map((segment) =>
        segment.index === index
          ? project.translate
            ? { ...segment, translation: value }
            : { ...segment, text: value }
          : segment,
      ),
    );
    setDirty(true);
  };

  const play = (segment: Segment): void => {
    const audio = audioRef.current;
    if (!audio) return;
    if (playing === segment.index) {
      audio.pause();
      setPlaying(null);
      return;
    }
    audio.currentTime = segment.start;
    void audio.play();
    setPlaying(segment.index);
  };

  const visible = query.trim()
    ? segments.filter((segment) => {
        const needle = query.trim().toLowerCase();
        return (
          segment.text.toLowerCase().includes(needle) ||
          segment.translation.toLowerCase().includes(needle)
        );
      })
    : segments;

  const translated = segments.filter((segment) => segment.translation.trim() !== "").length;

  return (
    <div className="editor">
      <header className="stage-head">
        <div>
          <p className="eyebrow">Review</p>
          <h1 className="stage-title">{baseName(project.source_path)}</h1>
          <p className="stage-sub">
            {project.source_language?.toUpperCase() ?? "Detected"}
            {project.target_language ? ` → ${project.target_language.toUpperCase()}` : ""}
            {" · "}
            {segments.length} lines
            {project.translate ? ` · ${translated} translated` : ""}
            {dirty ? " · unsaved changes" : ""}
            {message ? ` · ${message}` : ""}
          </p>
        </div>
        <div className="stage-actions">
          <label className="search">
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <circle
                cx="7"
                cy="7"
                r="4.2"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              />
              <path
                d="M10.5 10.5L14 14"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
              />
            </svg>
            <input
              type="search"
              value={query}
              placeholder="Find a line"
              onChange={(event) => setQuery(event.target.value)}
              aria-label="Find a line"
            />
          </label>
          <button type="button" className="button ghost" onClick={onReveal}>
            Open folder
          </button>
          <button
            type="button"
            className="button primary"
            disabled={saving}
            onClick={() => void save()}
          >
            {saving ? "Saving" : "Save and export"}
          </button>
        </div>
      </header>

      {visible.length === 0 ? (
        <p className="editor-empty">No line matches “{query}”.</p>
      ) : (
        <ol className="segments">
          {visible.map((segment) => (
            <li
              className={`segment${playing === segment.index ? " is-playing" : ""}`}
              key={segment.index}
            >
              <div className="segment-side">
                <button
                  type="button"
                  className="segment-play"
                  disabled={!audioSrc}
                  title={audioSrc ? "Play this line" : "Audio is available in the desktop app"}
                  aria-label={`Play line ${segment.index + 1}`}
                  onClick={() => play(segment)}
                >
                  {playing === segment.index ? (
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                      <rect x="4.5" y="4" width="2.4" height="8" rx="1" fill="currentColor" />
                      <rect x="9.1" y="4" width="2.4" height="8" rx="1" fill="currentColor" />
                    </svg>
                  ) : (
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                      <path d="M5.5 3.8l7 4.2-7 4.2z" fill="currentColor" />
                    </svg>
                  )}
                </button>
                <span className="segment-time">
                  {cueTime(segment.start)}
                  <span className="segment-time-end">{cueTime(segment.end)}</span>
                </span>
              </div>
              <div className="segment-body">
                <span className="segment-index">
                  {String(segment.index + 1).padStart(3, "0")}
                </span>
                <textarea
                  className="segment-input"
                  value={project.translate ? segment.translation : segment.text}
                  placeholder={project.translate ? "Add a translation" : "Type the line"}
                  spellCheck
                  rows={Math.max(
                    1,
                    Math.ceil(
                      (project.translate ? segment.translation : segment.text).length / 64,
                    ),
                  )}
                  onChange={(event) => update(segment.index, event.target.value)}
                />
                {project.translate && <p className="segment-source">{segment.text}</p>}
              </div>
            </li>
          ))}
        </ol>
      )}

      <audio
        ref={audioRef}
        src={audioSrc ?? undefined}
        preload="metadata"
        onTimeUpdate={(event) => {
          if (playing === null) return;
          const segment = segments.find((entry) => entry.index === playing);
          if (segment && event.currentTarget.currentTime >= segment.end) {
            event.currentTarget.pause();
            setPlaying(null);
          }
        }}
        onEnded={() => setPlaying(null)}
      />

      <footer className="progress-foot">
        <span>Exported files</span>
        <span className="file-path-list">
          {project.files.map((file) => (
            <code className="path" key={file}>
              {file}
            </code>
          ))}
        </span>
      </footer>
    </div>
  );
}
