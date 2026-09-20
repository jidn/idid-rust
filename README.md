# idid

[![Build](https://github.com/jidn/idid-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/jidn/idid-rust/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/idid)](https://crates.io/crates/idid)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

`idid` is a small command-line time tracker for recording what you actually
did.

It keeps your history in a plain text TSV file. Each record contains a
timestamp and a description. Durations are calculated from neighboring
records, so the file stays easy to read, edit, copy, and process with other
command-line tools.

## Contents

- [Why use it?](#why-use-it)
- [How it works](#how-it-works)
- [Install](#install)
- [Quick start](#quick-start)
- [Recording time](#recording-time)
- [Common workflows](#common-workflows)
- [Reviewing your history](#reviewing-your-history)
- [Edit the data directly](#edit-the-data-directly)
- [Command summary](#command-summary)
- [Useful scripts](#useful-scripts)
- [Contributing](#contributing)
- [License](#license)

## Why use it?

`idid` is designed for people who want to record work quickly without turning
their day into a project-management exercise.

- Record an activity with one short command.
- Track interruptions, meetings, breaks, and work in progress.
- Edit your history with a normal text editor.
- Keep your data in a simple, portable file.
- Filter and transform output with tools such as `grep`, `sort`, and `awk`.

## How it works

`idid` stores one timestamp and one description per line. It calculates an
activity’s duration from the timestamp on the following line:

```text
08:00  *~*~*--------------------
09:15  reviewed the project notes
10:00  answered email
```

In this example, `reviewed the project notes` lasted 45 minutes. The start
marker begins a tracking period but is not itself an activity.

This model keeps the data simple: add, remove, or correct a line and the
calculated durations adjust automatically.

## Install

With Rust and Cargo installed:

```sh
cargo install idid
```

To build the current repository instead:

```sh
git clone https://github.com/jidn/idid-rust.git
cd idid-rust
cargo install --path .
```

See [INSTALL.md](INSTALL.md) for release builds and Arch Linux installation.

## Quick start

The shortest useful workflow is:

```sh
idid start
idid add reviewed the project notes
idid show
```

The first record of a work period establishes its starting point. Each later
activity lasts until the next record. If you stop tracking for lunch or for a
long break, use `start` when you return to begin a new period.

## Recording time

### Start a period

`start` writes a special marker to the TSV file:

```sh
idid start
idid start --quiet
```

Use `-t` when the period began earlier:

```sh
idid start -t 10       # ten minutes ago
idid start -t 8:00am
idid start -t 13:15
```

### Add an activity

The text after `add` becomes the activity description. Multiple words are
joined into one description:

```sh
idid add prepared the meeting agenda
```

You can also enter an activity that happened earlier:

```sh
idid add -t 10 fixed issue 42
idid add -t 9:50 fixed issue 42
```

The time argument accepts a number of minutes ago, a 24-hour time, or a time
with `am` or `pm`.

Use `--quiet` when another program or shell script should receive no progress
message:

```sh
idid add --quiet sent the status update
```

Quote activity text when your shell would otherwise interpret special
characters such as `&`, `;`, redirects, or quotes:

```sh
idid add 'reviewed the build & deployment scripts'
```

### Tag activities

The description is deliberately free-form. A useful convention is to put a
project or context tag at the beginning:

```sh
idid add +acme emailed the next steps
idid add +lunch at Costco
```

Then filter the output:

```sh
idid show | grep '+acme'
```

## Common workflows

### Resume after a break

End the current period with an activity, then start a new period when you
return:

```sh
idid add finished the morning work
idid start
```

If you want to record the break itself, add it as an ordinary activity with a
tag such as `+personal`.

### Correct a missed entry

Use `-t` to record when an activity actually happened:

```sh
idid add -t 15 fixed the test failure
```

For larger corrections or inserted records, use `idid edit` and update the
TSV directly.

### Use short aliases

If `add` is part of your regular workflow, a short alias can make recording
nearly instant:

```sh
alias tt='idid add'
tt reviewed the deployment notes
```

## Reviewing your history

### See the most recent activity

With no argument, `last` shows how long ago the latest record was written:

```sh
idid last
00:25
```

Give it a number to print that many raw TSV lines, newest first:

```sh
idid last 3
```

### Show entries

`show` displays matching activities newest first. By default, it shows today:

```sh
idid show
```

Show a particular day:

```sh
idid show yesterday
idid show 1
idid show 2024-04-01
```

Show several days or an inclusive range:

```sh
idid show mon tue wed
idid show --range mon fri
```

Add a total to the normal human-readable output:

```sh
idid show --total
```

The output is intended to be useful to both people and scripts. A normal
entry looks like this:

```text
2024-04-01T15:02:24-05:00	00:10	emailed status update
```

The timestamp uses RFC 3339. The duration is shown as `HH:MM`.

For scripts that need seconds instead of `HH:MM`:

```sh
idid show --seconds
```

For structured processing, request one JSON object per matching entry:

```sh
idid show --json
idid show --json --seconds
```

The JSON output is newline-delimited: each output line is a separate JSON
object.

### Date formats

The date arguments accepted by `show` include:

- `today` and `yesterday`;
- a number from `0` to `999`, meaning that many days ago;
- `MM-DD` or `MMDD` for a recent date without a year;
- `YY-MM-DD`, `YYMMDD`, `YYYY-MM-DD`, or `YYYYMMDD`;
- a weekday such as `mon`, `tue`, or `fri`;
- a weekday followed by a number, such as `mon1`, for an earlier week.

Weekdays refer to the most recent occurrence before today. For example, if
today is Monday, `mon` means the previous Monday and `mon1` means the Monday
before that.

Run this whenever you need a compact reminder of the accepted formats:

```sh
idid show --help
```

## Edit the data directly

Use `edit` to open the TSV file in the editor named by `$EDITOR`:

```sh
idid edit
```

The data file is selected in this order:

| Source                         | Purpose                                                            |
| ------------------------------ | ------------------------------------------------------------------ |
| `--tsv FILE`                   | Use one specific file. The file must already exist.                |
| `ididTSV`                      | Set a preferred file path.                                         |
| `$XDG_DATA_HOME/idid/idid.tsv` | Use the default file, creating its directory and file when needed. |

For example, to use a separate file for a project:

```sh
touch project.tsv
idid --tsv project.tsv start
idid --tsv project.tsv add investigated the deployment issue
```

> [!TIP]
> If you use that file often, a short alias keeps the workflow quick:
>
> ```sh
> alias pa='idid --tsv project.tsv add'
> pa investigated the deployment issue
> ```

> [!IMPORTANT]
> Keep records in chronological order, use one record per line, and do not add
> blank lines or comments. Do not change the special start marker.

The special start marker is:

```text
*~*~*--------------------
```

Because durations are calculated from neighboring timestamps, inserting,
removing, or correcting a record automatically changes the calculated results.

## Command summary

```text
idid start [--quiet] [-t WHEN]
idid add [--quiet] [-t WHEN] TEXT...
idid edit
idid last [LINES]
idid show [DATE...] [--range DATE DATE] [--total] [--seconds] [--json]
```

All commands also accept `--tsv FILE` to select a particular history file.
Use `idid COMMAND --help` for the complete command-specific help.

## Useful scripts

The repository includes shell examples for processing `show` output:

- [`scripts/group-by-day.sh`](scripts/group-by-day.sh) groups durations by day.
- [`scripts/total-duration.sh`](scripts/total-duration.sh) adds durations.

For example:

```sh
idid show mon fri | grep -v lunch | ./scripts/group-by-day.sh
```

## Contributing

Changes are welcome. Build and verify the project with:

```sh
just verify
```

The repository’s [`justfile`](justfile) is the source of truth for these
development checks. Individual recipes are available for `build`, `fmt`,
`check`, `test`, `lint`, and `shell-check`.

## License

`idid` is distributed under the terms of the [MIT license](LICENSE-MIT).
