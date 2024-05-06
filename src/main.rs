use chrono::{DateTime, Duration, FixedOffset};
use clap::{Args, Parser, Subcommand};
use rand::seq::SliceRandom;
use std::env;
use std::fs;
use std::path::PathBuf;

mod date_parse;
mod time_parse;
mod util_time;
use util_time::current_datetime;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start tracking time.
    Start {
        /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,

        /// Quiet response
        #[arg(short, long)]
        quiet: bool,
    },

    /// Add something you did.
    #[command(arg_required_else_help = true)]
    Add {
        /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,

        /// Quiet response
        #[arg(short, long)]
        quiet: bool,

        /// Text to record
        #[arg(required = true)]
        text: Vec<String>,
    },
    /// Edit TSV file using $EDITOR.
    Edit,

    /// See duration from today's last entry or lines of TSV
    Last {
        /// See the last LINES of TSV
        #[arg(value_name = "LINES", help = "Number of LINES.")]
        lines: Option<u32>,
    },

    /// Show DATEs or DATE range entries.
    Show {
        #[clap(flatten)]
        args: ArgsShow,

        /// Show duration in seconds
        #[arg(long)]
        seconds: bool,

        /// Json output
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct ArgsAdd {
    /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
    #[arg(short = 't', value_name = "WHEN")]
    offset: Option<String>,

    /// Quiet response
    #[arg(short, long)]
    quiet: bool,

    /// Text to record
    #[arg(required = true)]
    text: Vec<String>,
}

#[derive(Args, Debug)]
struct ArgsShow {
    /// DATE can be any of:
    ///
    ///  -  Days before today. 0 is today and 1 is yesterday; max 999.
    ///  -  Literal "today" or "yesterday".
    ///  -  YYYYMMDD (dashes optinal).
    ///  -  YYMMDD (leading 0, dashes optional) starting in the year 2000.
    ///  -  Last MMDD (leading 0, dashes optinal); within 364ish days.
    ///  -  Last week day with optional numeric suffix to add weeks.
    ///     "mon" is last Monday and "mon1" goes back an additional week.
    #[arg(value_name="DATE", num_args=0.., help="See --help for allowed formats.", verbatim_doc_comment)]
    dates: Option<Vec<String>>,

    /// Pick entries inclusive of range
    #[arg(short = 'r', long, value_name = "DATE", num_args = 2)]
    range: Option<Vec<String>>,
}

#[derive(Parser)]
#[command(version, about, long_about, author)] // read from Cargo.toml
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// TSV file instead of $ididTSV or $XDG_DATA_HOME/idid/idid.tsv
    #[arg(long, value_name = "FILE")]
    tsv: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let tsv: String = idid::get_tsv_path(&cli.tsv)
        .unwrap()
        .to_string_lossy()
        .to_string();

    match &cli.command {
        Some(Commands::Add {
            offset,
            quiet,
            text,
        }) => {
            command_add(&tsv, offset.as_deref(), quiet, text);
        }
        Some(Commands::Start { offset, quiet }) => {
            command_start(&tsv, offset, quiet);
        }
        Some(Commands::Edit {}) => {
            command_edit(&tsv);
        }
        Some(Commands::Last { lines }) => {
            command_last(&tsv, lines);
        }
        Some(Commands::Show {
            args,
            seconds,
            json,
        }) => {
            let filter = date_filter_from_date_args(args);
            if filter.is_empty() {
                eprintln!("Error: at least one of --dates or --range is required");
                std::process::exit(1);
            }

            for entry in idid::pick(tsv, &filter) {
                println!("{}", entry.serialize(seconds, *json));
            }
        }
        None => {
            #[cfg(debug_assertions)]
            println!("None: current tsv={}", tsv);
        }
    }
}

/// Add a line to the end of the TSV
fn command_add(tsv: &str, offset: Option<&str>, quiet: &bool, text: &[String]) {
    match offset_from_current_or_current(offset) {
        Ok(ended) => {
            if text.is_empty() {
                eprintln!("Error: missing text");
                std::process::exit(1);
            }
            let timestamp = get_last_entry_timestamp(tsv).expect("Bad last entry");
            idid::write_to_tsv(tsv, &ended, Some(&text.join(" ")));
            let duration = current_datetime() - timestamp;
            if duration > Duration::hours(12) {
                println!(
                    "WARNING: elapsed time from last is {:>2}:{:>02}",
                    duration.num_hours(),
                    duration.num_minutes() % 60
                );
            } else if !quiet {
                if offset.is_some() {
                    // Show offset time for visual confirmation
                    print!("{} for ", ended.format("%a %I:%M %p"));
                }
                // Show duration
                print!(
                    "{}:{:>02}  ",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                );
                praise();
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(2);
        }
    }
}

fn command_edit(tsv: &str) {
    // Get the value of the EDITOR environment variable
    let editor = match env::var("EDITOR") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("EDITOR environment variable is not set");
            std::process::exit(1);
        }
    };

    let mut command = std::process::Command::new(&editor);
    command.arg(tsv);

    // Check if the editor is a vi variant
    if editor.ends_with("vi") || editor.ends_with("vim") {
        // Open at the end of the file
        command.arg("+$");
    }

    let status = command.status().expect("Failed to start editor process");
    if !status.success() {
        eprintln!("Editor process failed with error code: {:?}", status.code());
    }
}

fn command_start(tsv: &str, offset: &Option<String>, quiet: &bool) {
    match offset_from_current_or_current(offset.as_deref()) {
        Ok(ended) => {
            idid::write_to_tsv(tsv, &ended, None);
            if !quiet {
                print!("Starting at {}.  ", ended.time().format("%I:%M %p"));
                praise();
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(2);
        }
    }
}

fn command_last(tsv: &str, lines: &Option<u32>) {
    if lines.is_some() {
        let file = fs::File::open(tsv).expect("Failed to open TSV file");
        let mut reverse_buffer = rev_lines::RevLines::new(file);
        for _ in 0..lines.unwrap() {
            if let Some(Ok(line)) = reverse_buffer.next() {
                println!("{}", line);
            } else {
                break;
            }
        }
    } else {
        let timestamp = get_last_entry_timestamp(tsv).expect("Invalid TSV");
        let now = current_datetime();
        if now.date_naive() == timestamp.date_naive() {
            let elapsed = now - timestamp;
            println!(
                "Elapsed: {:>2}:{:>02}",
                elapsed.num_hours(),
                elapsed.num_minutes() % 60
            );
        } else {
            #[cfg(debug_assertions)]
            println!("Last not today but {}", timestamp.date_naive());
        }
    }
}

/// Get either an offset from current time or the current time.
fn offset_from_current_or_current(offset: Option<&str>) -> Result<DateTime<FixedOffset>, String> {
    if offset.is_some() {
        return time_parse::time_adjustment(offset);
    }
    Ok(current_datetime())
}

fn get_last_entry_timestamp(tsv: &str) -> Result<DateTime<FixedOffset>, String> {
    let file = fs::File::open(tsv).expect("Failed to open TSV file");
    let mut reverse_buffer = rev_lines::RevLines::new(file);
    match reverse_buffer.next().expect("empty TSV") {
        Ok(tsv_line) => {
            let (timestamp, _) = idid::Entry::from_tsv(&tsv_line)?;
            Ok(timestamp)
        }
        Err(e) => panic!("{}", e),
    }
}

/// Process dates and ranges using str_to_date
fn date_filter_from_date_args(args: &ArgsShow) -> idid::DateFilter {
    let parsed_dates = date_parse::strings_to_dates(&args.dates).expect("Unable to parse dates.");
    let parsed_range = date_parse::strings_to_dates(&args.range).expect("Unable to parse range.");
    idid::DateFilter::new(&parsed_range, &parsed_dates)
}

fn praise() {
    let praises = [
        "All right",
        "Brilliant",
        "Excellent",
        "Fantastic",
        "Good going",
        "Good job",
        "Great work",
        "Impressive",
        "Keep it up",
        "Kudos",
        "Nailed it",
        "Nice going",
        "Nice",
        "Outstanding",
        "Phenomenal",
        "Respect",
        "Sensational",
        "Simply superb",
        "Smashing",
        "Stellar",
        "Thank you",
        "Way to go",
        "Well done",
        "Wonderfull",
    ];
    let say = praises.choose(&mut rand::thread_rng()).unwrap();

    let punct = [".", "!", "!!"]
        .choose(&mut rand::thread_rng())
        .unwrap_or(&".");
    println!("{}{}", say, punct);
}
