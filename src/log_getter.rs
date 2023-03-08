use crate::types::*;

pub struct LogGetterImpl<Op: LogOperations> {
    op: Op,
}

impl<Op: LogOperations> LogGetter for LogGetterImpl<Op> {
    fn list_folders(&self, device: &String) -> LogResult<Vec<String>> {
        let dates = self.op.ls(&format!("{}/",device))?;
        if dates.len() == 0 {
            return LogResult::Err("No entries for this device".to_string());
        }
        let dates = dates.iter().map(|i| match i {
            FileIndex::File(_) => LogResult::Err("Should be prefixes only!".to_string()),
            FileIndex::Prefix(p) => LogResult::Ok(p.name.clone()),
        });
        if let Some(first_err) = dates.clone().find(|i| i.is_err()) {
            LogResult::Err(format!(
                "Failed to list dates, error : {}",
                first_err.unwrap_err()
            ))
        } else {
            let mut res : Vec<String> = dates.filter_map(|i| i.ok()).collect();
            res.sort();
            res.reverse();
            LogResult::Ok(res)
        }
    }

    fn download_folder(&self, device: &String, folder: &String) -> LogResult<String> {
        let input_path = format!("{}/{}/", device, folder).replace("//", "/");
        let files = self.op.ls(&input_path)?;

        let gen_output_path = |log_file: &str| -> String {
            format!("{}/{}/{}", device, folder, log_file.replace(".log.log", ".log"))
        };
        fn filter_func(i: &FileIndex) -> Option<File> {
            if let FileIndex::File(f) = i {
                if f.name.ends_with(".log.log") {
                    Some(f.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
        let download_map = files.iter().filter_map(|i| filter_func(i)).map(|f| {
            println!("Downloading {}", &f.name.replace(".log.log", ".log"));
            self.op.download(
                &format!("{}{}", input_path, &f.name),
                &gen_output_path(&f.name),
            )
        });
        if let Some(first_err) = download_map.clone().find(|f| f.is_err()) {
            LogResult::Err(first_err.unwrap_err())
        }
        else {
            LogResult::Ok(format!("{}/{}/", device, folder).replace("//", "/"))
        }
    }

    fn new(configs: Configs) -> Self {
        LogGetterImpl {
            op: Op::new(configs),
        }
    }
}
