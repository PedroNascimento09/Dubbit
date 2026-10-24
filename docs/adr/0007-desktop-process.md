# 0007: The window is a thin shell over the core, with one queue in the app process

- Status: Accepted
- Date: 2026-10-24

## Context

Milestone M3 needs a window a non-technical person can use: add a file, choose a language, watch it work, fix a line, export. The engine already exists as the `dubbit-core` library with a stable API and a project folder on disk.

Three forces shape the choice. The window must not fork the pipeline logic, or the command line and the window drift. A run takes minutes, so the interface has to stay responsive and survive being closed. And the installable size matters, because a subtitle tool that weighs more than the media it processes is a hard sell.

## Decision

The desktop application is a Tauri 2 shell in `apps/desktop/src-tauri`, with a React and TypeScript interface in `apps/desktop/src`.

- The core runs in the same process, behind Tauri commands. There is no second copy of the pipeline and no local HTTP server to keep in sync.
- The interface never touches the disk, the network or a process. It calls commands and listens to two events: `jobs` for the whole queue after a change, and `model` for download progress.
- The queue lives in the application data folder as `queue.json` and is owned by a single worker thread. Jobs run one at a time. A job that was running when the process died returns to the queue and resumes from its last finished checkpoint.
- Each stage is checkpointed by the core, so closing the window costs at most the current stage.
- Stopping a job sets a cancellation flag that the pipeline checks between stages. The recogniser is a separate process and is not killed mid-file; the window says so instead of pretending the stop is instant.
- The review editor saves through the core: it writes the segment checkpoints, rewrites every subtitle format, and only then reports success.
- The audio track is played from the project folder through the asset protocol, with the folder allowed per project rather than globally.
- The interface keeps a demo mode, used when it is opened outside the shell. It serves deterministic fixtures so layout and interaction can be checked in a browser without a build of the shell, and it never writes files.
- A render failure is caught at the root and offers a reload, because an empty window loses the user's trust faster than an ugly error.

## Consequences

- One binary, one process, one log. A crash takes the window and the current stage together, and the checkpoint recovers the run.
- The interface stays replaceable: any future client can call the same core, and the command line remains the reference for behaviour.
- Adding a feature to the pipeline means adding it to the core, not to the window. The window only decides how to ask.
- The queue is per user, not per project. Two windows on one machine share it, which is intended.
- The window depends on a system webview: WebView2 on Windows, WebKitGTK on Linux. A machine without one cannot run it, which is a packaging question for M4.
- A demo mode has to keep matching the real types. The Rust views are the source of truth, and a mismatch shows up as a TypeScript error at build time.

## Alternatives considered

- A separate local server with the interface in a browser: more moving parts, a port to secure, and the window loses the ability to close a job cleanly.
- An immediate-mode Rust interface: smallest binary, but weak text editing and typography for a review editor, which is the part that sells the product.
- Bundling the interface build into the core crate: fewer crates, but the core would carry a user interface it does not need.
