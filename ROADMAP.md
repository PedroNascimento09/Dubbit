# Roadmap

Dubbit ships in small milestones, each one usable and tested before the next starts. Dates are not promised; a milestone is done when its acceptance criterion is met and verified.

| Milestone | Deliverable | Acceptance criterion |
| --- | --- | --- |
| M0 | Research notes, licence audit, name validation and architecture decision records | The licence audit covers every dependency and model, the name is checked, and the decisions are written down |
| M1 | Core crate and command line client producing subtitles end to end on CPU | A short clip is subtitled in one run on a machine with no GPU |
| M2 | Model manager with resumable, verified downloads, hardware detection and stage checkpoints | Killing the process mid-run and running again continues from the last finished stage, and every hash matches |
| M3 | Desktop shell with a review editor and a processing queue | A non-technical user subtitles a video without reading documentation |
| M4 | Signed installers for Windows and Linux | A clean virtual machine installs Dubbit and produces subtitles with no manual steps |
| M4b | First public release, v0.3, subtitles only | Clean install and generated subtitles on an 8 GB machine with no GPU, zero network requests while processing, release checklist complete |
| M5 | Timing fit, background separation, glossary and subtitle polish | Quality metrics for timing and translation are published and reproducible |
| M6 | AMD, Intel and Apple acceleration with fallback to CPU | A compatibility matrix generated from tests, with four honest states |
| M7 | Public release 1.0 with dubbing | Full release gates pass on Windows, Linux and macOS |

## Known gaps

- A Windows installer and a portable Windows package exist and bundle the media tools at pinned, verified versions, but they have been installed on one machine only, never on a clean virtual machine. The Linux build is defined in the release workflow and has not run yet. There is no public release.
- The desktop window is a development preview: the queue, the review editor and the save path work.
- macOS is untested. Only Windows and Linux are on the tested path.
- The installer bundles FFmpeg and the speech recognition executable; the command line build still expects both on the machine.
- One model family covers recognition, with four sizes and no alternative engines.
- Translation now has first automatic numbers for English and Portuguese only, on two public corpora, and no reader review. No other pair has a number, and the product still reaches a model through the local server or the command backend rather than shipping one.

## How the scope is cut

If the schedule tightens, features are dropped from the top of this list down, keeping installation, the review editor, timing fit and background audio: lip sync, voice cloning, on-screen text localisation, plugins and style profiles, local language model translation, speaker separation, AMD and Intel acceleration, export to video editors.

Every cut is recorded here with its reason.
