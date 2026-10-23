use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "dubbit",
    version,
    about = "Local subtitling for video and audio.",
    long_about = "Dubbit transcribes and translates video and audio on your machine. Nothing is uploaded."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, global = true, help = "Print machine readable output")]
    pub json: bool,

    #[arg(long, global = true, help = "Print more detail")]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Produce subtitles for a video or audio file")]
    Dub(DubArgs),
    #[command(about = "Manage offline models")]
    Models {
        #[command(subcommand)]
        command: ModelsCommand,
    },
    #[command(about = "Check this machine and write a report")]
    Doctor(DoctorArgs),
    #[command(about = "List supported languages")]
    Languages,
    #[command(about = "Read or write settings")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    #[command(about = "Show the state of a project folder")]
    Project {
        #[arg(value_name = "DIRECTORY")]
        directory: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct DubArgs {
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "DIRECTORY", help = "Folder for the project")]
    pub output: Option<PathBuf>,

    #[arg(long = "to", value_name = "LANGUAGE", help = "Target language")]
    pub target: Option<String>,

    #[arg(long = "from", value_name = "LANGUAGE", help = "Source language")]
    pub source: Option<String>,

    #[arg(long, value_name = "MODEL", help = "Recognition model id")]
    pub model: Option<String>,

    #[arg(long, value_name = "PROFILE", help = "Hardware profile")]
    pub profile: Option<String>,

    #[arg(long, value_enum, value_name = "ENGINE", help = "Translation backend")]
    pub translator: Option<TranslatorArg>,

    #[arg(
        long,
        value_name = "FILE",
        help = "Glossary CSV with source,target rows"
    )]
    pub glossary: Option<PathBuf>,

    #[arg(
        long = "format",
        value_name = "FORMAT",
        value_delimiter = ',',
        help = "Subtitle formats: srt, vtt"
    )]
    pub formats: Vec<String>,

    #[arg(long, help = "Transcribe only, without translation")]
    pub no_translate: bool,

    #[arg(long, help = "Continue an interrupted project")]
    pub resume: bool,

    #[arg(long, value_name = "COUNT", help = "Worker threads")]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TranslatorArg {
    Auto,
    Passthrough,
    Ollama,
    Command,
}

impl From<TranslatorArg> for dubbit_core::config::TranslatorChoice {
    fn from(value: TranslatorArg) -> Self {
        match value {
            TranslatorArg::Auto => Self::Auto,
            TranslatorArg::Passthrough => Self::Passthrough,
            TranslatorArg::Ollama => Self::Ollama,
            TranslatorArg::Command => Self::Command,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ModelsCommand {
    #[command(about = "List models from the manifest")]
    List,
    #[command(about = "Download and verify a model")]
    Install {
        #[arg(value_name = "MODEL")]
        id: String,
    },
    #[command(about = "Delete a downloaded model")]
    Remove {
        #[arg(value_name = "MODEL")]
        id: String,
    },
    #[command(about = "Verify a downloaded model")]
    Verify {
        #[arg(value_name = "MODEL")]
        id: Option<String>,
    },
    #[command(about = "Print the path of a model")]
    Path {
        #[arg(value_name = "MODEL")]
        id: String,
    },
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(long, value_name = "FILE", help = "Write the report to a file")]
    pub report: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    #[command(about = "Print the current settings")]
    Show,
    #[command(about = "Change one setting")]
    Set {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(value_name = "VALUE")]
        value: String,
    },
    #[command(about = "Print the settings file path")]
    Path,
}
