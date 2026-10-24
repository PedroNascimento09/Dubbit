# 0008: Translation stays behind a process boundary until an in-process runtime is proven

- Status: Accepted
- Date: 2026-10-24

## Context

Dubbit can translate subtitles through three backends: passthrough, a local model server that answers an HTTP request, and an external command that reads text on standard input. None of them ships with the product, so a new user gets subtitles in the spoken language only.

Measurement 4 in the benchmarks gives the quality of the OPUS-MT weights that would be shipped: chrF between 62 and 72 on real text for the five launch languages, each way against English. The same measurement found a cost that a decision has to face.

| Property | Measured |
| --- | --- |
| Weights on disk, per direction | 83 MB for English to Portuguese, 165 MB for English to German |
| Memory added while loaded | 350 MB and 426 MB |
| Time to the first translated line | 3.99 s and 3.56 s, mostly model load |
| Steady cost | 0.08 to 0.09 s per line |
| Quantised weights at int8 | Spanish to English repeats itself forever |

The light profile already spends 400 to 1500 MB on recognition. Adding 350 to 426 MB per language is affordable on the 8 GB machine the plan targets, but it changes what "light" means, and one of the five languages needs float32 to be correct.

## Decision

Translation keeps the existing process boundary for this milestone. No translation runtime is linked into `dubbit-core`, and no weights are bundled in the installer.

- The three backends stay as they are; a user who wants translation configures a local model server or a command.
- The interface keeps disabling the target language when no translator is configured, and it keeps saying that translated lines are a first draft.
- The v0.3 promise is not weakened to "subtitles" as a result, but the one-click promise of "translate on your PC" is not advertised until an in-process runtime exists. Saying otherwise would be selling an integration that is not there.
- The measurement scripts in `benchmarks/` stay the reference for any candidate runtime: quality per pair, memory, time to first line, and correctness per package.

The gate for reversing this decision is one spike with four answers, all measured on this machine: an in-process runtime that loads one pair from disk, the packaging and licence of that runtime with the Apache-2.0 product, the memory it adds, and the correctness of all five languages under the quantisation it would use.

## Consequences

- The product stays small and the licence boundary stays simple: no native inference library is linked yet, and no weight licence has to be cleared for distribution.
- A user who wants translation today has to install and run something else. That is a real gap against the product promise and the reason this record exists rather than an assumption.
- The int8 finding travels with the decision: whatever runtime is chosen must be tested per package, not per family, because one of five languages was wrong under a setting that looked like a pure speed trade.
- If the spike fails, the fallback is to keep the boundary and ship a documented path to a local server, which costs the promise but not the product.

## Alternatives considered

- Link CTranslate2 bindings into the core now: measured cost is acceptable, but it adds a native build to every platform and a licence surface before M4 installation is proven on a clean machine.
- Link ONNX Runtime and convert the weights: same packaging risk, plus a conversion step that has no measured equivalent yet.
- Bundle one pair and treat the rest as downloads: smallest first-run cost, but it hides the per-language weights problem instead of solving it.
