# Agent skill setup

Conductor's repo-specific Matt Pocock skill settings live in this directory. The skills themselves are installed for Codex at the user level under ~/.codex/skills; they are not vendored into the product repository.

As of 2026-09-29, all 38 installed skill trees were compared recursively with the upstream mattpocock/skills main branch at commit c55ee46073ed923f86ce59a5eb3b6d895095d1b7. Every skill was present and its files matched byte-for-byte. The upstream source and its installation guidance are at [github.com/mattpocock/skills](https://github.com/mattpocock/skills).

The install is user-level and can change independently of this repository. Recheck the installed files against the current upstream revision before claiming they are still the latest. Repository-specific workflow policy takes precedence over a skill's default orchestration; see [the local-first workflow](../development-workflow.md).
