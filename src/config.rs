use std::{time::Duration};

use clap::{command, arg};

const DEFAULT_CONN_TIMEOUT: u64 = 1000;

#[derive(Debug)]
pub enum FileType {
    Obfs4(String),
    Proxy(String),
    DefaultObfs4
}

#[derive(Debug)]
pub struct Config {
    pub file: FileType,
    pub conn_timeout: Duration
}

impl Config {
    pub fn new() -> Self {
        let matches =  command!()
            .arg(
                arg!(--filetype <filetype>  "Set file type: obfs4, proxy, default")
                .required(true)
            )
            .arg(
                arg!(--filepath <FILEPATH>  "Set path to file")
                .required(false)
            )
            .arg(
                arg!(--timeout <MILLISECONDS> "Set connection timeout in milliseconds")
                .required(false)
            ).get_matches();

        let file_path = matches.get_one::<String>("filepath");

        let file_type = match matches.get_one::<String>("filetype") {
            Some(file_type) => {
                match file_type.as_str() {
                    "default" => FileType::DefaultObfs4,
                    "obfs4" => {
                        match file_path {
                            None => panic!("missing filepath for obfs4 filetype"),
                            Some(file_path) => FileType::Obfs4(file_path.clone())
                        }
                    }
                    "proxy" => {
                        match file_path {
                            None => panic!("missing filepath for obfs4 filetype"),
                            Some(file_path) => FileType::Proxy(file_path.clone())
                        }
                    }
                    _ => panic!("Unknown filetype: {}", file_type)
                }
            },
            None => {
                panic!("")
            }
        };

        let conn_timeout = match matches.get_one::<String>("timeout") {
            Some(conn_timeout) => Duration::from_millis(conn_timeout.parse::<u64>().unwrap()),
            None =>  Duration::from_millis(DEFAULT_CONN_TIMEOUT)
        };

        Config { file: file_type, conn_timeout: conn_timeout }
    }
}