use types::{ConfigsCollection, LogGetter};
mod log_getter;
mod s3_cli;
mod types;
use clap::*;
use libc;
use serde_json;
use types::Environment;

#[derive(Parser, Debug)]
#[command(author, version, about="Downloading air logs from s3", long_about = None)]
struct Args {
    #[clap(
        short = 'e',
        long = "env",
        help = "environment where logs supposed to be (prod/stage/dev)"
    )]
    environment: Environment,

    #[clap(short = 'd', long = "device", help = "device id")]
    device: String,

    #[clap(
        short = 'f',
        long = "folder",
        help = "folder (date) in format 2000-12-31. Leave empty to list all"
    )]
    folder: Option<String>,

    #[clap(short = 'l', long = "last", help = "download N last folders")]
    last: Option<usize>,

    #[clap(
        short = 'o',
        long = "output",
        help = "folder to save to, getting from configs by default"
    )]
    out_folder: Option<String>,

    #[clap(flatten = true, long = "list", help = "list present folders")]
    list: bool,
}

fn load_configs() -> ConfigsCollection {
    let configs_path =
        std::env::var("AIRLOGS_CONFIGS").unwrap_or("airlog_configs.json".to_string());
    if !std::path::Path::new(&configs_path).exists() {
        panic!("Configs didn't found. Put airlog_configs.json in current directory or specify path with AIRLOGS_CONFIGS env variable")
    }
    serde_json::from_str(
        std::fs::read_to_string(&configs_path)
            .expect("Failed to read configs file, probably it is corrupted")
            .as_str(),
    )
    .expect("Failed to deserialize configs. Please check if all fields are there")
}

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}

fn main() {
    reset_sigpipe();
    let config_list = load_configs();
    let args = Args::parse_from(std::env::args());
    let mut needed_configs = config_list.get_config(&args.environment).clone();
    let out_path = args.out_folder.unwrap_or(needed_configs.output_path);
    needed_configs.output_path = out_path.clone();
    let getter = log_getter::LogGetterImpl::<s3_cli::AWSCliOperations>::new(needed_configs);
    if !args.list {
        let mut folders_to_download = Vec::new();
        if args.folder.is_some() {
            folders_to_download.push(args.folder.clone().unwrap());
        }
        if args.folder.is_none() || args.last.is_some() {
            let last_folder_number = args.last.unwrap_or(1);
            let present_folders = getter.list_folders(&args.device).unwrap_or(Vec::new());
            present_folders
                .into_iter()
                .take(last_folder_number)
                .for_each(|f| folders_to_download.push(f));
        }
        for folder_to_download in folders_to_download {
            println!("Downloading folder \"{}\", please wait", folder_to_download);
            match getter.download_folder(&args.device, &folder_to_download) {
                Ok(dest) => println!("Done! You can find files at \n{}/{}", out_path, dest),
                Err(e) => println!(
                    "Failed to download folder, probably device or date not found. Error : {}",
                    e
                ),
            }
        }
    } else {
        match getter.list_folders(&args.device) {
            Ok(items) => items.into_iter().for_each(|i| println!("{}", i)),
            Err(e) => println!(
                "Failed to list folder, probably device not found. Error : {}",
                e
            ),
        }
    }
}
