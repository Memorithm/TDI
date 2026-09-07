# TDI Release Policy

GitHub Releases are immutable public milestones for TDI research artefacts. They are not a substitute for the repository's scientific status documents.

## Current tagged milestones

| Tag | Release status | Scientific role |
| --- | --- | --- |
| `tdi-2-continuous-v0.1.0` | published | validated TDI-2 continuous milestone |
| `tdi-3-preregistered-results` | publication managed by CI | completed preregistered inter-width result; TDI-3A and TDI-3B failed |
| `tdi-4-preregistered-results` | publication managed by CI | completed preregistered target-geometry result; TDI-4A and TDI-4B failed |
| `tdi-5.3-preregistered-results` | published | tagged TDI-5.3 preregistered result milestone |

## Rules

1. A Release must point to an existing historical tag or to a newly created tag that identifies the exact frozen scientific milestone.
2. Do not create a release tag at current `main` merely because an older experiment lacks a tag.
3. Release notes must preserve the frozen verdict, including failed, equivalent, inconclusive or blocked outcomes.
4. When a frozen evidence bundle is attached, it must be built from the tagged tree, not reconstructed from a later branch.
5. Attached evidence bundles receive a SHA-256 checksum.
6. Active programmes, preregistrations without completed evidence, identifiability blockers and unauthorised holdouts must not be represented as completed result releases.
7. Future TDI-5.x, TDI-6.x, TDI-7.x, TDI-8.x, TDI-9.x and TDI-10.x releases require recovery or creation of the exact milestone tag only when the repository evidence supports that tag.

## Automation

`.github/workflows/publish-historical-releases.yml` publishes the missing TDI-3 and TDI-4 GitHub Release pages idempotently from their existing annotated tags. Existing releases are left untouched.

The next release-audit phase is to map completed but untagged TDI-5.x and TDI-6.x experiments to their frozen scientific commits before creating any additional tags or releases.
