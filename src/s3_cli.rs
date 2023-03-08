use crate::types::*;
use chrono::{NaiveDateTime, Utc};
use std::env;
use std::path::Path;
use std::process::Command;

pub struct AWSCliOperations {
    configs: Configs,
}

impl AWSCliOperations {
    fn output_to_string(&self, output: std::process::Output) -> String {
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .trim()
            .to_string()
    }

    fn get_output(&self, command: &str) -> String {
        let shell = match env::consts::OS {
            "windows" => "powershell",
            "linux" => "bash",
            _ => "sh",
        };
        self.output_to_string(Command::new(shell).arg("-c").arg(command).output().unwrap())
    }

    fn get_path(&self, path: &str) -> String {
        format!(
            "s3://{}/{}",
            self.configs.base_path,
            path.replace("//", "/")
        )
    }
}

impl LogOperations for AWSCliOperations {
    fn new(configs: Configs) -> AWSCliOperations {
        let future_self = AWSCliOperations { configs };
        if let Err(e) = future_self.ls("") {
            panic!(
                "Failed to do ls on logs base path {}, error : {}. Possible reasons are :\n
                1) You need to relogin with aws sso login --profile {}
                2) Base path is invalid
                3) No internet connection
                4) Profile doesn't exist or doesn't have permissions",
                &future_self.configs.base_path, e, &future_self.configs.profile
            );
        }
        future_self
    }

    fn ls(&self, path: &str) -> LogResult<Vec<FileIndex>> {
        let ls_command = format!(
            "aws s3 ls {} --profile {}",
            self.get_path(path),
            self.configs.profile
        );
        // println!("{}", ls_command);
        let output = self.get_output(ls_command.as_str());
        // println!("{}", output);
        fn parse_line(l: &str) -> LogResult<FileIndex> {
            let after_split: Vec<&str> = l.split_whitespace().collect();
            match after_split.len() {
                2 => LogResult::Ok(FileIndex::Prefix(Prefix {
                    name: after_split[1].to_string(),
                })),
                4 => {
                    let date_time = NaiveDateTime::parse_from_str(
                        format!("{} {}", after_split[0], after_split[1]).as_str(),
                        "%Y-%m-%d %H:%M:%S",
                    );
                    if date_time.is_err() {
                        return LogResult::Err("Failed to parse date".to_string());
                    }
                    let size = after_split[2].parse();
                    if size.is_err() {
                        return LogResult::Err("Failed to parse size".to_string());
                    }
                    match date_time.unwrap().and_local_timezone(Utc) {
                        chrono::LocalResult::None => {
                            LogResult::Err("DateTime is None!".to_string())
                        }
                        chrono::LocalResult::Single(v) => LogResult::Ok(FileIndex::File(File {
                            name: after_split[3].to_string(),
                            size: size.unwrap(),
                            date: v,
                        })),
                        chrono::LocalResult::Ambiguous(_, _) => {
                            LogResult::Err("DateTime is ambiguos!".to_string())
                        }
                    }
                }
                _ => LogResult::Err("Incorrect index detected".to_string()),
            }
        }
        let possible_result = output.lines().map(|i| parse_line(i));
        let result: Vec<FileIndex> = possible_result.filter_map(|i| i.ok()).collect();
        if result.len() == 0 {
            LogResult::Err(format!("File or path {} was not found", path))
        } else {
            LogResult::Ok(result)
        }
    }

    fn download(&self, path: &str, to: &str) -> LogResult<()> {
        if let Err(_) = self.ls(path) {
            println!("Err");
            return LogResult::Err("Failed to upload file, file not found".to_string());
        }
        let output_path = format!("{}/{}", self.configs.output_path, to);
        self.get_output(&format!(
            "aws s3 cp {} {} --profile {}",
            self.get_path(path),
            output_path,
            self.configs.profile
        ));
        if Path::new(&output_path).exists() {
            LogResult::Ok(())
        } else {
            LogResult::Err("Failed to download file!".to_string())
        }
    }
}
