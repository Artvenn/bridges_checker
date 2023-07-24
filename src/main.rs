use std::{
    net::{SocketAddr, TcpStream, Ipv4Addr, IpAddr, Shutdown},
    str::FromStr, time::Duration, fs::File, 
    io::{ErrorKind, Write, Read}, process::Command
};

use config::{Config, FileType};

mod config;

fn main() {
    let config = Config::new();
    let default_obfs4 = include_str!("../bridges-obfs4");
    const WORKING_PATH: &str = "./working_bridges.txt";

    let working  =  match &config.file {
        FileType::Obfs4(path) => {
            let mut file = File::open(path)
                .expect(format!("Cant open filepath: {}", path).as_str());

            let mut file_content = String::new();
            file.read_to_string(&mut file_content).expect("Cant read file with addresses");
            get_working_bridges(&file_content, &config)
        },
        FileType::Vanilla(path) => {
            let mut file = File::open(path)
                .expect(format!("Cant open filepath: {}", path).as_str());

            let mut file_content = String::new();
            file.read_to_string(&mut file_content).expect("Cant read file with addresses");
            get_working_bridges(&file_content, &config)
        },
        FileType::Proxy(path) => get_working_proxy(&path, &config),
        FileType::DefaultObfs4 => get_working_bridges(default_obfs4, &config)
    };

    if working.is_empty() {
        println!("There is no working bridges :(");
        return;
    }

    let mut file = match File::options()
        .append(true).create(true).write(true).open(WORKING_PATH) 
    {
        Ok(f) => f,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => { 
                File::create(WORKING_PATH)
                    .expect(format!("cant create file: {}", WORKING_PATH).as_str())
            },
            err => panic!("cant open file: {}\n{}", WORKING_PATH, err)
        }
    };

    println!("-----------------Working bridges list----------------------");
    for bridge in working {
        println!("{bridge}");
        file.write(bridge.as_bytes()).unwrap();
        file.write(b"\n").unwrap();
        println!("-----------------------------------------------------------");
    }

    file.flush().unwrap();
    println!("Working bridges was saved in working_bridges.txt, located in executable directory");
    println!("Press enter to exit");
    let _ = Command::new("pause").status().unwrap();
}

fn get_working_bridges<'a>(bridges: &'a str, config: &Config) -> Vec<String> {
    let start_pos = match config.file {
        FileType::Obfs4(_) => 1,
        FileType::Vanilla(_) => 0,
        _ => panic!("get_working_bridge() expect FileType vanilla or obfs4")
    };

    bridges.trim().lines().filter_map(|row| {
        let (ip_str, port) = row
            .trim()
            .split(' ')
            .nth(start_pos).expect("Cant get ip:port section")
            .split_once(':').expect("ip:port section must contain semicolon between");

        let ip = Ipv4Addr::from_str(ip_str)
            .expect("ip addr parse error.\nCant parse: {ip}");

        let port = port.parse().unwrap_or(0);
        let sock = SocketAddr::new(IpAddr::V4(ip), port);

        if check_conn(&sock, config.conn_timeout) {
            Some(row.to_owned())
        } else {
            None
        }
    }).collect()
}

fn get_working_proxy<'a>(proxies: &'a str, config: &Config) -> Vec<String> {
    proxies.trim().lines().filter_map(|row| {
        let (ip, port) = row.split_once(':').unwrap();
        let ip = Ipv4Addr::from_str(ip)
            .expect("ip addr parse error.\nCant parse: {ip}");
        let port: u16 = port.parse().unwrap();
        let sock = SocketAddr::new(IpAddr::V4(ip), port);
        if check_conn(&sock, config.conn_timeout) {
            Some(row.to_owned())
        } else {
            None
        }
    }).collect()
}

fn check_conn(sock: &SocketAddr, conn_timeout: Duration) -> bool {
    match TcpStream::connect_timeout(sock, conn_timeout) {
        Ok(stream) => {
            println!("[OK]\t{}", sock);
            stream.shutdown(Shutdown::Both).unwrap();
            true
        },
        Err(_) => {
            println!("[ERR]\t{}", sock);
            false
        } 
    }
}