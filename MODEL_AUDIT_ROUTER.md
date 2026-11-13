# Independent Model Audit Router — EMU-OPS

**Revision date:** 2026-11-13

## Mandatory data and evidence policy
- Never replace real or maintainer-provided data with invented, generated, synthetic or placeholder records.
- Trace every dataset to its source, licence, collection date, previous Git revision and limitations before changing it.
- If a source cannot be verified, preserve the current data and mark it unknown/unverified. Never invent replacement records.
- Synthetic fixtures are permitted only when explicitly labelled and isolated from runtime/production data.
- Do not fabricate official records, identities, licence numbers, reports, citations, metrics, benchmark outputs, partnerships, deployments or contributors.
- Claim tests passed only with reproducible output on the exact commit reviewed.

## README preservation
Recover the last maintainer-authored README from Git history. Keep it intact. Never replace or rewrite the original. Any AI-written explanation belongs in a dated, clearly labelled addendum at the bottom only. Preserve original links, sections, examples and content.

## Independent audit roles
1. Data provenance and synthetic-data separation.
2. Code, architecture and API correctness.
3. Security, privacy, dependency and trust-boundary review.
4. Build, tests, CI and reproducibility verification.
5. README history, licensing, attribution and link review.
6. Recruiter/researcher-facing claims and evidence review.

## Review workflow
1. Record repo URL, default branch, exact HEAD SHA, fork/original status and relevant PR heads.
2. Classify changes as merged, open, closed-unmerged, reverted or unverified.
3. Inventory runtime data, fixtures, generated files, APIs and provenance.
4. Compare README with its pre-AI Git-history baseline.
5. Run documented install/build/lint/test commands on the exact reviewed commit.
6. Check edge cases, error paths, security, privacy, accessibility and performance.
7. Verify all public-facing claims and links.
8. Report evidence before changing code or data.

## Required finding format
Severity (**P0/P1/P2/P3**), exact path and line/symbol, reproduction steps, expected vs actual, evidence and commit SHA, confirmed/suspected/unverifiable status, minimal fix, regression test, and whether data or README history is affected.

## Merge gate
- [ ] No fabricated data presented as real or official.
- [ ] Real and user-provided data preserved; no silent substitutions.
- [ ] Synthetic fixtures clearly labelled and separated.
- [ ] Original README preserved; AI addendum at bottom only.
- [ ] Tests and CI checked on exact PR head; failures disclosed.
- [ ] No secrets, private personal details, classified or confidential information.
- [ ] Licensing and upstream attribution retained.
- [ ] Independent second review and maintainer approval completed.

Repository: **EMU-OPS**. Do not infer functionality from the name or roadmap. Do not merge, delete, force-push, replace datasets or rewrite the original README during an audit. The repository owner has final approval.
