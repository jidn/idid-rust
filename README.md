# idid

[![Build Status](https://github.com/jidn/idid-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/jidn/idid-rust/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/idid)](https://crates.io/crates/idid)

*idid* is a command-line tool for keeping track of time spent on various tasks
or accomplishments. It allows users to record, edit, and view their activities
in a simple, structured format.
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

```sh
$ cargo install idid
```

TODO: add package manager support.  Anyone interested?

### Build from repository

To build idid, follow these steps:

Clone the repository:

```sh
$ git clone https://github.com/jidn/idid-rust.git
```

Navigate to the project directory:

```sh
$ cd idid-rust
```

Build the project:

```sh
$ cargo build --release
```

Optionally, add the executable to your PATH:

```sh
$ export PATH="$PATH:/path/to/idid-rust/target/release/"
```



## Quick Start

`idid` revolves around the idea of recording what you just finished 
As you record your accomplishments, the duration is calculated from your previous entry.
Thus, your first accomplishment for the day is starting.
If you need to alter time, insert an entry, or edit the entry; just edit the TSV file everything adjusts accordingly.

### Start your day
Your first accomplishment of the day is just to start `idid` recording for the day.

```shell
$ idid start
Starting at 07:55 AM.  All right!
```

Wow, nice feedback.  If you prefer no response, just use the `--quiet` flag.

Altering the start time is easy. Just give it the number of minutes ago or the time.
For example, as I was coming in, Tim stopped me in the hall for about 10 minutes going over an item and it is now 8:05 am.

```shell
$ idid start -t 10 
Starting at 07:55 AM.  Keep it up.

$ idid start -t 7:55 
Starting at 07:55 AM.  Sensational.
```

### Additional accomplishments

As you finish a task, milestone, or item of note, record what you did.

```shell
$idid add cleared inbox
Mon 08:20 AM for 00:25  Well done!
```

Nice. I see the time recorded, duration in HH:MM format, and some positive feedback.

Later on, you forgot to record fixing an issue 10 minutes ago at 9:50.
To alter the time, use the `-t` option with either the number of minutes or the time.

```shell
$ idid add -t 10 fixed issue #42
Mon 09:50 AM for 01:30  Well done!
```
or

```shell
$ idid add -t 9:50 fixed issue #42
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

`idid` makes it easy to change your history by allowing you to use your favorite text editor to make changes.
If you are like me and the vi family of editors is your friend, the following will open the TSV file in your editor and place you at the bottom, or most recent entry.

```shell
$ idid edit
```

If you are using another text editor.  The file is usually found at `$XDG_DATA_HOME/idid/idid.tsv` or `~/.local/share/idid/idid.tsv`.

Now you can make changes.
+ Remove that double entry.
+ Add the accomplishment you forgot.
+ Fix the typos.

Things to remember.

+ The TSV must be in chronological order. The duration is dependent on it.
+ Blank lines and comments are not allowed.
+ Do not alter the start text. 

### Show your day

It would be nice to show a list of your accomplishments for today.

```shell
$ idid show today
```

Yes, I know the date looks a bit funny.  It is a format specified in (RFC 3339)[http://tools.ietf.org/html/rfc3339).
While "readable" may be debated, it has several benefits as it remains in chronological order when sorted, is strictly defined, and has common library support.

You can give any number of **`DATE`**s or use the `--range` with two dates to get all entries with the range.

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
+ **show**: Show all recorded accomplishments.
+ **sum**: Summarize accomplishments by day.

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

`idid` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.

