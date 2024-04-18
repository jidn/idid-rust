use chrono::{DateTime, Duration, FixedOffset, NaiveDate};
use clap::{Args, Parser, Subcommand};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
// use std::process;

mod date_filter;
mod date_parse;
mod entry;
mod time_parse;
mod tsv;
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

    /// See duration from today's last entry or lines of TSV
    Last {
        /// See lines of TSV
        #[arg(short = 'n', long, value_name = "LINES", default_value_t = 0)]
        lines: u32,
    },

    /// Start recording time.
    Start {
        /// WHEN minutes ago or time, ie "8am", "13:15", "4:55pm"
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,
    },

    /// Show accomplishments.
    Show {
        #[clap(flatten)]
        args: DateArgs,

        /// Show duration in seconds
        #[arg(short, long)]
        seconds: bool,

        /// Json output
        #[arg(long)]
        json: bool,
    },

    /// Sum accomplishments by day
    Sum {
        #[clap(flatten)]
        args: DateArgs,
        /// Show total after summing daily entries
        #[arg(short, long)]
        total: bool,
    },
}

#[derive(Args, Debug)]
struct DateArgs {
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

    /// Show duration instead of ending timestamp
    #[arg(short, long)]
    duration: bool,
}

#[derive(Parser)]
#[command(version, about, long_about)] // read from Cargo.toml
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// TSV file instead of $ididTSV or $XDG_DATA_HOME/idid/idid.tsv
    #[arg(long, value_name = "FILE")]
    tsv: Option<PathBuf>,
}

/// Process dates and ranges using str_to_date
fn date_filter_from_date_args(args: &DateArgs) -> date_filter::DateFilter {
    let parsed_dates = date_parse::strings_to_dates(&args.dates).unwrap();
    let parsed_range = date_parse::strings_to_dates(&args.range).unwrap();
    date_filter::DateFilter::new(&parsed_range, &parsed_dates)
}

fn main() {
    let cli = Cli::parse();
    let tsv: String = tsv::get_tsv_path(&cli.tsv)
        .unwrap()
        .to_string_lossy()
        .to_string();

    match &cli.command {
        Some(Commands::Add { offset, text }) => match ended_at(offset.as_deref()) {
            Ok(ended) => {
                if text.len() == 0 {
                    eprintln!("Error: missing text");
                    std::process::exit(1);
                }
                tsv::write_to_tsv(&tsv, &ended, &text.join(" "));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
        Some(Commands::Start { offset }) => match ended_at(offset.as_deref()) {
            Ok(ended) => {
                tsv::write_to_tsv(&tsv, &ended, entry::START_RECORDING);
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
        Some(Commands::Last { lines }) => {
            let file = fs::File::open(&tsv).expect("Failed to open TSV file");
            let mut reverse_buffer = rev_lines::RevLines::new(file);
            if lines.is_some() {
                for _ in 0..lines.unwrap() {
                    if let Some(Ok(line)) = reverse_buffer.next() {
                        println!("{}", line);
                    } else {
                        break;
                    }
                }
            } else {
                match reverse_buffer.next().expect("empty TSV") {
                    Ok(tsv_line) => {
                        let (timestamp, _) = entry::Entry::from_tsv(&tsv_line).unwrap();
                        let now = util_time::current_datetime();
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
                    Err(e) => panic!("{}", e),
                }
                // let tuple_vec: Result<Vec<_>, _> = reverse_buffer
                //     .take(2)
                //     .map(|item| item.map_err(|e| e.to_string())) // Handle error conversion
                //     .collect();
                //
                // // Check if the collection was successful
                // match tuple_vec {
                //     Ok(vec) => {
                //         println!("{:?}", vec);
                //     }
                //     Err(err) => eprintln!("Error collecting tuples: {}", err),
                // }
            }
        }
        Some(Commands::Show {
            args,
            seconds,
            json,
        }) => {
            let filter = date_filter_from_date_args(&args);
            if filter.is_empty() {
                eprintln!("Error: at least one of --dates or --range is required");
                std::process::exit(1);
            }

            for entry in entry::pick(tsv, &filter) {
                println!("{}", entry.serialize(&seconds, *json));
            }
        }
        Some(Commands::Sum { args, total }) => {
            let filter = date_filter_from_date_args(&args);
            let mut total_duration = Duration::zero();

            if filter.is_empty() {
                eprintln!("Error: at least one of --dates or --range is required");
                std::process::exit(1);
            }

            // Sum all entry durations by day
            let mut daily_durations: BTreeMap<NaiveDate, Duration> = BTreeMap::new();
            for entry in entry::pick(tsv, &filter) {
                let date = entry.begin.date_naive();
                let duration = entry.cease - entry.begin;

                let day_duration = daily_durations.entry(date).or_insert(Duration::zero());
                *day_duration = *day_duration + duration;
                total_duration = total_duration + duration;
            }

            // Print all the durations by day
            for (date, duration) in &daily_durations {
                println!(
                    "{}  {:>4}:{:>02}",
                    date.format("%Y-%m-%d %a"),
                    duration.num_hours(),
                    duration.num_minutes() % 60
                );
            }

            if *total && total_duration.num_minutes() > 0 {
                println!(
                    "{:>14}  {:>4}:{:>02}",
                    "Total",
                    total_duration.num_hours(),
                    total_duration.num_minutes() % 60,
                );
            }
        }
        None => {
            #[cfg(debug_assertions)]
            println!("None: current tsv={}", tsv);
        }
    }
}

fn ended_at(offset: Option<&str>) -> Result<DateTime<FixedOffset>, String> {
    if offset.is_some() {
        return time_parse::time_adjustment(offset);
    }
    Ok(util_time::current_datetime())
}
