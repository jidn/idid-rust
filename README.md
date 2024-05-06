# idid

[![Build Status](https://github.com/jidn/idid-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/jidn/idid-rust/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/idid)](https://crates.io/crates/idid)
[![Unit tests](https://github.com/jidn/idid-rust/actions/workflows/unit-tests.yml/badge.svg)](https://github.com/jidn/idid-rust/actions/workflows/unit-tests.yml)


**Idid** is a command-line tool for tracking time spent and kept in a simple, structured format.

Why another time tracker?  Simply, the others didn't meet my needs.  I wanted:

- Easy command line.
- Simple data structure.
- Simple to edit and modify.
- Geared for my needs, not any manger or corprate desires.
- Focus on what I actually did, not what I planned on doing; these are not the same. 

**Easy command line**.
There are only five commands: start, add, edit, last, and show.
Of thoses you could be happy only knowing the first three.
Some of the most popular well over 15 different commands, too many.

**Simple data structure**.
It is a two column tab-separated-value (TSV) file. 
The timestamp and my text description of what I did.
No start time or duration is needed as it can be infered from the previous row.

**Simple to edit and modify**.
This tool does not attempt to make changes.
The possibilities are vast, so you edit the TSV file directly.
You want to insert a task? 
Just add a row and because start and duration is calculated from previous rows you don't need to modify an others.

**Geared for my needs**.
I wanted to accurately track interuptions; people dropping by needing help, mentoring, or solving problems.
I also was curious about the actual amount of time I spent on various activities and discovered it didn't match what I remembered in a day or two.

**Focus on what I did**.
Most of the alternate *solutions* start with the premis, "I am going to start THIS."
What I quickly discovered is planning rarely survives the actual encounter.
So I needed to record what I did, and that required a quick change, I could quickly jot down what I was doing with work-in-progress "+WIP".

## Evolution of `idid`

This started as a simple bash script, adding features as I needed.
It worked well.  
Eventually, it moved to Python. However, setting it up on other machines took up more time as I needed it in a user virtual environment and not the global python.
I was learning Rust and here was an opportunity to migrate to an app with years of entries.



-----

**Table of Contents**

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
    - [Commands](#commands)
    - [Options](#options)
- [Contributing](#contributing)
- [License](#license)

## Installation

There are a number of ways to install `idid` either repository, cargo, or Arch Linux makepkg.
Read the [INSTALL.md](INSTALL.md) file for details.

## Quick Start

`idid` revolves around the idea of recording what you just did, hence the name `idid`.
As you record your activity, the duration is calculated from your previous entry.
Thus, your first activity for the day is starting.

### Start your day
Your first accomplishment of the day is just to start `idid` recording for the day.

```sh
idid start
Starting at 07:55 AM.  All right!
```

Wow, nice feedback.  If you prefer no response, just use the `--quiet` flag.

Altering the start time is easy. Just give it the number of minutes ago or the time.
For example, as I was coming in, Tim stopped me in the hall for about 10 minutes going over an item and it is now 8:05 am.

```sh
idid start -t 10 
Starting at 07:55 AM.  Keep it up.

idid start -t 7:55 
Starting at 07:55 AM.  Sensational.
```

### Additional activity

As you finish a task, milestone, or item of note, record what you did.

```
did add cleared inbox
Mon 08:20 AM for 00:25  Well done!
```

Nice. I see the time recorded, duration in HH:MM format, and some positive feedback.

Later on, you forgot to record fixing an issue 10 minutes ago at 9:50.
To alter the time, use the `-t` option with either the number of minutes or the time.

```
idid add -t 10 fixed issue #42
Mon 09:50 AM for 01:30  Well done!
```

or

```sh
idid add -t 9:50 fixed issue #42
Mon 09:50  01:30  Well done!
```

Remember you are typing in your shell so some characters will cause problems.
The most common issues are single quotes, semi-colons, and ampersands.
You will have to quote them or use natural language. 

### Noncontiguous: lunch and extended breaks

Lunch or extended breaks may not be something you want to track.
For some reason, those to whom I report don't want that time included.
Add an entry before leaving about what you have done up to that point with `idid add 'project poodles work-in-progress (WIP)'` or something similar.
Now use `idid start` after returning.

### Edit your history

If you need to alter time, insert an entry, or edit an entry; just edit the TSV file and everything adjusts accordingly.
The `edit` sub-command allows you to quickly use the vi family of editors open the TSV file and place you at the file's end.

```shell
idid edit
```

If you are using another text editor.  The file is usually found at `$XDG_DATA_HOME/idid/idid.tsv` or `~/.local/share/idid/idid.tsv`.

Now you can make changes.
+ Remove that double entry.
+ Add the accomplishment you forgot.
+ Fix the typos.

Things to remember.

+ The TSV must be in chronological order. The duration depends on it.
+ Blank lines and comments are not allowed.
+ Do not alter the day's start text. 

### Show your day

It would be nice to show a list of your accomplishments for today.

```shell
$ idid show today
```

Yes, I know the date looks a bit funny.  It is a format specified in (RFC 3339)[http://tools.ietf.org/html/rfc3339).
While "readable" may be debated, it has several benefits as it remains in chronological order when sorted, is strictly defined, and has common library support.
You can give any number of **`DATE`**s or use the `--range` with two dates to get all entries with the range.

This consistant output format allows you to create additional tools to transforms the information for reporting, invoicing, or whatever your mind dreams up.  See [group-by-day.sh](scripts/group-by-day.sh) as an example.

#### DATE formats

The word `today` is a special `DATE`, as is `yesterday`.
You can also use the number of days in the past, to `idid show 0` is the same as `idid show today`.
While `DATE` as a number is difficult to use, any number less than a thousand is valid.

A much easier format is the two-digit month and day as **`MM-DD`** or **`MMDD`**; the dash is optional.
As long as the date is within the last 264ish days, you don't need to specify the year.
When you need the year, use **`YY-MM-DD`** or **`YYYY-MM-DD`** will give you the exact date.

You can also use the abbreviated day-of-the-week (DOW) i.e. 'mon', 'tue', ..., 'sun'. 
Just remember that if today is Monday, then 'mon' is last Monday not today.
If you want to add additional weeks, append a number to the DOW.
The Monday one week before the last Monday is `mon1`.

If today were Monday, April 1, 2024, then Sunday, March 31, 2024, could be represented by any of the following:
+ yesterday
+ 1
+ 03-31 or 0331
+ 2024-03-31
+ sun

I know. It seems a bit excessive. But I use them, so use the ones that work best for your needs.
If you need a quick reminder, execute `idid show --help`. 
The long form `--help` shows the many date formats.

## Usage

The idid tool provides several commands and options for managing your accomplishments. Here's a brief overview:

### Commands

+ **add**: Add a new accomplishment.
+ **edit**: Edit the TSV (Tab-Separated Values) file using your default editor.
+ **last**: See the duration from today's last entry or display a specific number of lines from the TSV file.
+ **start**: Start recording time for the day.
+ **show**: Show selected accomplishments.

### Options

* `--tsv <FILE>`: Specify a custom TSV file instead of the default location.
* `-h, --help`: Display help information.
* `-V, --version`: Display the version of **idid**.

For detailed usage instructions and examples, run `idid --help` or `idid <command> --help`.

## Contributing

Contributions to idid-rust are welcome! If you'd like to contribute, please follow these steps:

+ Fork the repository.
+ Create your feature branch (`git checkout -b feature/my-feature`).
+ Commit your changes (`git commit -am 'Add new feature'`).
+ Push to the branch (`git push origin feature/my-feature`).
+ Create a new Pull Request.

## License

`idid` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) [license](LICENSE-MIT).

