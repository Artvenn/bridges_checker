use std::{
    net::{SocketAddr, TcpStream, Ipv4Addr, IpAddr, Shutdown},
    str::{FromStr}, time::Duration, fs::{File}, 
    io::{ErrorKind, Write}
};

fn main() {
    let bridges = include_str!("../bridges-obfs4");
    const WORKING_PATH: &str = "./working_bridges.txt";
    let working = get_working(bridges);

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
                    .expect("cant create file: {WORKING_PATH}") 
            },
            _ => panic!("cant open file: {WORKING_PATH}")
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
}

fn get_working(bridges: &str) -> Vec<&str>{
    bridges.trim().lines().filter_map(|row| {
        let (ip_str, port) = row
            .trim()
            .split(' ')
            .nth(1).expect("Cant get ip:port section")
            .split_once(':').expect("ip:port section must contain semicolon between");

        let ip = Ipv4Addr::from_str(ip_str)
            .expect("ip addr parse error.\nCant parse: {ip}");

        let port = port.parse().unwrap_or(0);
        let sock = SocketAddr::new(IpAddr::V4(ip), port);

        if check_conn(&sock, Duration::from_secs(1)) {
            Some(row)
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