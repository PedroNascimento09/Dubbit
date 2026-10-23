# 0002: Apache-2.0 for all project code

- Status: Accepted
- Date: 2026-10-23

## Context

Dubbit is public and free. The licence determines who can adopt it, what a redistributor must preserve, and how much confidence a company needs before embedding it in a product. The community around offline media tools is unusually sensitive to copyleft licences, and a permissive licence removes that objection before it is raised.

The choice also has to survive contact with the dependencies. Media tooling is a mix of permissive and copyleft code, and a permissive project that links a copyleft library loses the freedom it advertised.

## Decision

All project code is licensed under Apache-2.0, with the official licence text in the repository root.

- The copyright holder and year are in the `NOTICE` file, which a redistributor must keep.
- Dependencies are limited to permissive licences: MIT, BSD, ISC, Apache-2.0 and the Unicode licence. Copyleft components are allowed only when they run as separate executables invoked over the command line, never linked into a crate. That rule covers media tools and any recognised codec whose licence is stronger.
- The name, logo and mascot are not licensed with the code. The trademark policy states what a fork may and may not call itself.
- Contributions use the Developer Certificate of Origin, signed off in each commit, rather than a contributor licence agreement.

Per-file SPDX headers are not used. The project forbids comments in code, tests and configuration, and a licence header is a comment. Licensing metadata lives in `REUSE.toml` and `NOTICE` instead, which keeps the repository machine-readable without reintroducing comment blocks.

## Consequences

- Anyone may use Dubbit, including in a closed product, provided the licence, the notice and the statement of changes are preserved.
- The patent grant gives adopters a reason to prefer this project over a permissive alternative without one.
- Companies can ship it without a legal review turning into a licence negotiation.
- The cost is accepted knowingly: a third party may close their version and sell it, and the project cannot stop them. Protection of the name comes from trademark policy and from the project's reputation, not from copyright.
- Forks must keep the attribution and must rename. Cases that ignore this are handled as licence breaches, not as a technical measure.
- Relicensing becomes impossible once outside contributions exist, so this decision is treated as final.
- Without per-file headers, a reader who copies one file into another project may lose the provenance. `REUSE.toml` covers every path in the repository and the contributing guide states the licence in plain words.

## Alternatives considered

- MIT: shortest and most popular, but no patent grant and no explicit trademark language.
- GPL-3.0: forces published changes, which is attractive in principle and costly in adoption.
- AGPL-3.0: also covers network use, and deters exactly the users this project wants.
- A non-commercial licence: not an open source licence, and it would end the project's claim to be one.
