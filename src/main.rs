use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process::Command;

// mod lib;
// mod idid;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add accomplishment.
    #[command(arg_required_else_help = true)]
    Add {
        /// Either WHEN minutes ago or at HH:MM.
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,

        /// Text to record
        #[arg(required = true)]
        text: Vec<String>,
        // println!(idid::get_current_timestamp());
    },
    /// Edit TSV file using $EDITOR.
    Edit,

    /// Start recording time.
    Start {
        /// Either WHEN minutes ago or HH:MM.
        #[arg(short = 't', value_name = "WHEN")]
        offset: Option<String>,
    },

    // The following need selectors and filters
    // Date can be any of:
    //     Number of days before today; zero is today and one is yesterday.
    //     Literal text of 'today' or 'yesterday'.
    //     ISO 8601 date format 'YYYY-MM-DD' or 'MM-DD' the last 364ish days.
    //         Dashes are optional.
    //     The locale abbreviated, last day-of-the-week. A suffix adds weeks.
    //         'mon' and 'mon1' is last Monday and 'mon2' is two Mondays ago.
    // --date DATE[,DATE]...    multiple allowed
    // --range DATE DATE        multiple allowed
    //
    // Filters
    // -x --exclude string
    // -i --include string   or -f --find string
    /// Lines for processing.
    Lines,

    /// Show accomplishments.
    Show,

    /// Summarize accomplishments.
    Summary,
}

#[derive(Parser)]
#[command(version, long_about = None)] // read from Cargo.toml
#[command(about = "Add, edit, and show accomplishments.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Use a custom TSV file instead of $ididTSV or $XDG_DATA_HOME
    #[arg(long, value_name = "FILE")]
    tsv: Option<PathBuf>,
}

fn get_string(option_str: Option<String>) -> String {
    option_str.unwrap_or(String::from("NONE"))
}

fn main() {
    let cli = Cli::parse();
    let tsv: String = idid::get_tsv_path(cli.tsv)
        .unwrap()
        .to_string_lossy()
        .to_string();
    match cli.command {
        Some(Commands::Add { offset, text }) => {
            println!(
                "Add offset={}, timestamp={}, tsv={}, text='{}'",
                get_string(offset),
                "get_current_timestamp",
                // idid::get_current_timestamp(),
                tsv,
                text.join(" "),
            );
        }
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
            // match std::env::var("EDITOR") {
            //     Ok(value) => {
            //         println!("editor= '{}'", value);
            //         // let mut args = vec![value.clone()];
            //         // // path could have spaces, /Users/Clinton James/AppData/
            //         // args.push(format!("\"{tsv}\""));
            //         // if value.ends_with("vi") || value.ends_with("vim") {
            //         //     // Open at the end of the file.
            //         //     args.push(String::from("+$"));
            //         // }
            //         // println!("cmdline= '{}'", args.join(" "));
            //
            //         let mut cmd = Command::new(&value).arg(&tsv);
            //         if value.ends_with("vi") || value.ends_with("vim") {
            //             cmd.arg("+$");
            //         }
            //
            //         cmd.status().expect("failed to execute process");
            //     }
            //     Err(e) => {
            //         eprintln!("$EDITOR not found: {e}");
            //         std::process::exit(1);
            //     }
            // }
        }
        Some(Commands::Start { offset }) => {
            println!("Start t={}, tsv={}", get_string(offset), tsv);
        }
        Some(Commands::Lines {}) => {
            println!("Lines tsv={}", tsv);
        }
        Some(Commands::Show {}) => {
            println!("Show tsv={}", tsv);
        }
        Some(Commands::Summary {}) => {
            println!("Summary tsv={}", tsv);
        }
        None => {
            println!("Show current timing tsv={}", tsv);
        }
    }
}
