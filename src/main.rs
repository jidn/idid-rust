use crate::date_filter::DateFilter;
use crate::util_time::current_datetime;
use chrono::{DateTime, FixedOffset, NaiveDate};
use clap::{Parser, Subcommand};
use idid::write_to_tsv;
use std::env;
use std::path::PathBuf;
use std::process::Command;

mod date_filter;
mod date_parse;
mod pick;
mod time_parse;
mod util_time;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add accomplishment.
    #[command(arg_required_else_help = true)]
    Add {
        /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,

        /// Text to record
        #[arg(required = true)]
        text: Vec<String>,
    },
    /// Edit TSV file using $EDITOR.
    Edit,

    /// Start recording time.
    Start {
        /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,
    },

    /// Pick entries for processing by starting date.
    ///
    Pick {
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

        /// Pick entries inclusive to range
        #[arg(short = 'r', long, value_name = "DATE", num_args = 2)]
        range: Option<Vec<String>>,

        /// Exclude entries containing TEXT
        #[arg(short = 'x', long, value_name = "TEXT")]
        exclude: Option<Vec<String>>,
    },

    /// Show accomplishments.
    Show,

    /// Summarize accomplishments.
    Summary,
}

#[derive(Parser)]
#[command(version, about, long_about)] // read from Cargo.toml
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Use a custom TSV file instead of $ididTSV or $XDG_DATA_HOME
    #[arg(long, value_name = "FILE")]
    tsv: Option<PathBuf>,
}

fn ended_at(offset: Option<&str>) -> Result<DateTime<FixedOffset>, String> {
    if offset.is_some() {
        return time_parse::time_adjustment(offset);
    }
    Ok(current_datetime())
}

fn main() {
    let cli = Cli::parse();
    let tsv: String = idid::get_tsv_path(cli.tsv)
        .unwrap()
        .to_string_lossy()
        .to_string();

    match cli.command {
        Some(Commands::Add { offset, text }) => match ended_at(offset.as_deref()) {
            Ok(ended) => {
                write_to_tsv(&tsv, &ended, &text.join(" "));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
        Some(Commands::Start { offset }) => match ended_at(offset.as_deref()) {
            Ok(ended) => {
                write_to_tsv(&tsv, &ended, pick::START_RECORDING);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
        Some(Commands::Edit {}) => {
            // Get the value of the EDITOR environment variable
            let editor = match env::var("EDITOR") {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("EDITOR environment variable is not set");
                    std::process::exit(1);
                }
            };

            let mut command = Command::new(&editor);
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
        Some(Commands::Pick {
            dates,
            range,
            exclude,
        }) => {
            // Process dates and ranges using str_to_date
            let parsed_dates = date_parse::strings_to_dates(&dates).unwrap();
            let parsed_range = date_parse::strings_to_dates(&range).unwrap();
            let _filter = DateFilter::new(&parsed_dates, &parsed_range);

            #[cfg(debug_assertions)]
            println!(
                "Lines tsv={}, dates={:?}, range={:?}, exclude={:?}",
                tsv, parsed_dates, parsed_range, exclude
            );
        }
        Some(Commands::Show {}) => {
            #[cfg(debug_assertions)]
            println!("Show tsv={}", tsv);
        }
        Some(Commands::Summary {}) => {
            #[cfg(debug_assertions)]
            println!("Summary tsv={}", tsv);
        }
        None => {
            #[cfg(debug_assertions)]
            println!("Show current tsv={}", tsv);
        }
    }
}
