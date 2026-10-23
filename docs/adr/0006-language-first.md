# 0006: English first for code, commits and technical documentation

- Status: Accepted
- Date: 2026-10-23

## Context

The tool solves a problem that is not specific to one country: teachers, podcasters and small studios all over the world need subtitles for media they already own. Contributions and bug reports will arrive in several languages, and a repository with mixed languages in identifiers and commit messages is hard to search and hard to review.

At the same time, the people who need the tool most are not necessarily comfortable reading English installation instructions.

## Decision

English is the working language of the project.

- Code, identifiers, tests, configuration, commit messages, issue templates and architecture records are English.
- Technical documentation and the primary README are English.
- User-facing text is written in English first, then translated. Translations live in separate files, one per language, and are reviewed by someone who reads that language.
- Error titles and hints in the product are short English sentences. Localising them is a later change that touches the presentation layer, not the error codes, which stay stable.
- Commit messages never carry anything but the change: no identification of other products, projects or companies.

## Consequences

- One vocabulary across code, commits and issues, which keeps review fast and search useful.
- The project is readable by the widest pool of contributors, which matters more for a small project than for a large one.
- Error codes decouple the message from the failure, so a translation never invalidates troubleshooting instructions.
- The cost is unequal: contributors write in a second language, and a clumsy sentence can obscure a good idea. Reviewers are expected to fix wording rather than reject the change.
- Translated guides are work that has to be scheduled before a release in a new language, not an automatic side effect.
- Native speakers of a language the project claims to support may find the English documentation a barrier until the translation ships, which is an honest reason to delay a language claim.

## Alternatives considered

- A bilingual repository from the start: doubles the maintenance of every document and splits discussions.
- Code in one language and documentation in another: confuses the vocabulary, because the terms in the guide stop matching the names in the code.
- Identifier translation per locale: breaks search and makes the codebase unreadable for anyone outside that locale.
