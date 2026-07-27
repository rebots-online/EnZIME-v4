# Cold-Start Audit

Run before creating application source.

## A. Pak integrity

- [ ] Pak ZIP filename matches the release manifest.
- [ ] ZIP SHA-256 matches the operator-provided value.
- [ ] `PAK-MANIFEST.sha256` verifies every included file.
- [ ] No archive member escapes the extraction directory.
- [ ] No file named as a symlink points outside the extracted tree.

## B. Closed-world reference check

- [ ] Every path referenced by `KICKOFF-PROMPT.md` exists in the pak.
- [ ] Every path referenced by `ARCHITECTURE.md` is either an implementation
      output path or exists in the pak/vendor tree.
- [ ] No instruction requires a previous repository.
- [ ] No instruction requires a prior conversation.
- [ ] No instruction depends on `/home/robin`, `~`, or another absolute
      workstation path.
- [ ] No secret is embedded in the pak.

## C. Vendor check

- [ ] `VENDOR-MANIFEST.md` exists.
- [ ] Every required vendor directory exists.
- [ ] Every locked revision is recorded.
- [ ] Every vendor tree digest verifies.
- [ ] Every licence is present and compatible.
- [ ] Vendor trees are clean and contain no build outputs.

## D. Contract check

- [ ] Architecture has sections 1 through 25.
- [ ] Binding invariants A-01 through A-10 exist.
- [ ] SQL schema names match §16.4.
- [ ] Rust trait names match §16.3.
- [ ] Command registry has no duplicate command.
- [ ] Event registry has no duplicate event.
- [ ] React component names match §16.6.
- [ ] Routes match §16.7.
- [ ] Entity-to-UI rows use declared commands, tables and stores.

## E. Checklist reset

- [ ] Every executable checklist task begins `[ ]`.
- [ ] No task begins `[/]`, `[X]` or `✅`.
- [ ] Historical completion evidence is absent.
- [ ] Each task names allowed files.
- [ ] Each task includes Do, Verify and Accept.
- [ ] Each task is executable from its own text plus the pak.

## F. Historical failure guards

- [ ] No bulk donor-repository copy is planned.
- [ ] No historical `lib.rs` module forest is copied.
- [ ] First compile occurs before feature breadth.
- [ ] Android and desktop compile graphs are tested separately.
- [ ] No task claims success from prose alone.
- [ ] No screen is considered generated without `get_screen` evidence.
- [ ] No UI prompt may create a backend entity.
- [ ] No implementation placeholder is permitted in release code.
- [ ] No entitlement bypass is permitted.
- [ ] No checklist marker is advanced before its own acceptance evidence.

## Audit result

```text
Result: PASS | FAIL
Auditor:
Date:
Pak SHA-256:
Vendor manifest SHA-256:
Failed rows:
```

On FAIL, correct the pak.
Do not compensate in application code.

