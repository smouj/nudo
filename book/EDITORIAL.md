# Editorial contract

## Page contract

Every substantive chapter must contain:

1. one H1 title;
2. an explicit status;
3. a one-sentence summary;
4. enough context to explain why the subject exists;
5. links to normative sources when the subject is language-defining;
6. a clear distinction between implemented behaviour and proposed behaviour.

## Status vocabulary

Use only:

- **Implemented** - observable in the current toolchain.
- **Specified** - normative or sufficiently settled, but not necessarily implemented.
- **Proposed** - design direction; may change through a NEP.
- **Planned** - scheduled in the roadmap, with no implementation claim.
- **Open** - deliberately unresolved.
- **Historical** - describes past project state or decisions.

## Typography and layout

- H1 starts a major chapter/leaf in PDF output.
- Avoid H4 unless the section is genuinely deep enough to require it.
- Code examples shorter than one printed page should not be split.
- Tables must have meaningful headers and should not be used for prose layout.
- Figures and diagrams require captions.
- Avoid orphan headings and one-line carryovers.
- Keep paragraphs compact: one idea per paragraph, usually 2-5 sentences.

## Terminology

- `NUDO` for the project/brand.
- `Nudo` in ordinary prose where sentence casing is required.
- `nudo` for the binary, commands and filenames.
- `.nudo` for source files.

## Truthfulness

Never present roadmap syntax as implemented syntax. Never claim provider,
security, sandbox, agent or verification behaviour that the runtime does not yet
implement. The book should age well because uncertainty is recorded rather than
smoothed over.
