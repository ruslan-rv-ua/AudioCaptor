# AudioCaptor

Portable Windows audio recorder: Tauri v2, Rust backend, Svelte 5 + TypeScript frontend. Structure, build commands and the release procedure are in [DEVELOPMENT.md](DEVELOPMENT.md).

Accessibility for screen-reader users is the top requirement: every interface change keeps keyboard access, labels and spoken state changes (see "Accessibility" in DEVELOPMENT.md).

Backlog records (tasks, drafts, milestones, decisions, documents) are written in Ukrainian. Code, code comments and commit messages are written in English.

## Backlog first

Work found along the way goes into the backlog first, as a task or a draft placed in order; doing it waits for the owner's word. A draft is work that still needs an owner decision or an outside input.

## Branches: git-flow-next

The repository runs on git-flow-next; `git-flow overview` shows the setup.

- Start every branch with `git flow feature start <name>` (`git flow bugfix start <name>` for a bug task).
- The owner decides when a branch is finished; then run `git flow feature finish <name> --no-fetch --no-push`.
- `main` moves only when a release finishes: `git flow release start <version>`, then `git flow release finish <version> --notag`. The Release workflow creates the tag `v<version>`.
- Changes to `backlog/` and `CLAUDE.md` alone are committed straight to `develop`, with no branch.
- The owner works alone: branches are finished locally, and a pull request is opened only when the owner asks for one.
- Pushing and starting a GitHub workflow wait for the owner's go-ahead.
- Every GitHub release ships to Scoop users within a day (the Excavator workflow in `ruslan-rv-ua/scoop-bucket` picks up the newest one), so releases and `v*` tags are created only for real versions.
- Help is `git-flow <command> --help`; `git flow --help` fails.

## Checks

- Run `just check`: it runs `pnpm vite:build` (which generates `src/paraglide/` for `pnpm check`), `pnpm check`, clippy with `--all-targets -- -D warnings`, `cargo test` and `cargo fmt --check`, and stops at the first failure.
- Rust code uses the default rustfmt style; run `cargo fmt --manifest-path src-tauri/Cargo.toml` before committing.
- Pushes to `develop` start no CI run, so these local checks decide whether a task is done.

## Sessions: main and working

The owner coordinates in one **main session** and runs each backlog task in a **working session** started from a task card. A session started from a task card is a working session; any other session in this project is the main session.

Main session:

- Picks the next task with the owner by backlog order and dependencies, and answers small questions.
- Offers a task card on request, titled with the task ID and its labels, for example `TASK-12 · Fable · high`. The owner sets model and effort in the card.
- Only reads while a working session is active: both share this folder.
- Reviews a working session's report against the diff, the checks and the task's acceptance criteria before calling the task done.
- Archives a working session once the owner accepts its task and the branch is finished; it can be reopened from the Archived list.
- Closes a task once the owner accepts it, its branch is finished and the local checks pass. When the task changes CI, the build or dependencies, it first asks the owner for a CI run started by hand. A red CI run becomes a bug task.
- Does quick tasks itself, without a card: a handful of commands or one short file.
- Commits its backlog changes as it makes them, without asking.

Working session:

- Runs one at a time, in this folder, on its own branch named after the task: `git flow feature start task-12-flac-export`.
- Commits its work on that branch without asking; the commits stay local.
- Ends with a report: what changed, which checks ran, which acceptance criteria are met. Finishing the branch and pushing stay with the owner.

## Model and effort labels

Each task and draft carries one `model-*` and one `effort-*` label: the Claude model and reasoning effort recommended for it.

- Give both labels to every new task and draft.
- Use them when delegating to a subagent or a session, unless the owner chooses otherwise.
- When a task proves harder than its labels assume, say so and raise effort before model.

<!-- BACKLOG.MD GUIDELINES START -->
<!-- backlog.md-instructions-version: 1.57.4 -->
<CRITICAL_INSTRUCTION>

## Backlog.md Workflow

This project uses Backlog.md for task and project management.

**At the beginning of each conversation in this project, run `backlog instructions overview` before answering or taking action. Re-read it only if you have not read it yet in the current conversation.**

Use the overview to decide whether to search, read, create, or update Backlog tasks.

Before task lifecycle actions, read the matching detailed guide:
- `backlog instructions task-creation` before creating or splitting tasks
- `backlog instructions task-execution` before planning, changing status or assignee, adding a plan or implementation notes, or implementing task work
- `backlog instructions task-finalization` before checking acceptance criteria, writing final summaries, or moving tasks to terminal statuses

Use `backlog <command> --help` before running unfamiliar commands. Help shows options, fields, and examples.

Do not edit Backlog task, draft, document, decision, or milestone markdown files directly. Use the `backlog` CLI so metadata, relationships, and history stay consistent.

</CRITICAL_INSTRUCTION>
<!-- BACKLOG.MD GUIDELINES END -->
