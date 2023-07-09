use std::{
    net::{SocketAddr, TcpStream, Ipv4Addr, IpAddr, Shutdown},
    str::{FromStr}, time::Duration, fs::{File}, 
    io::{ErrorKind, Write}
};

enum BridgesType {
    Obfs4,
    Snowflake,
    Proxy
}

const BRIDGE_TYPE: BridgesType = BridgesType::Proxy;
const CONN_TIMEOUT_MS: u64 = 200;

fn main() {
    let obfs4_bridges = include_str!("../bridges-obfs4");
    let snowflake_bridges = include_str!("../bridges-snowflake-ipv4");
    let proxy_list = include_str!("../socks5.txt");
    const WORKING_PATH: &str = "./working_bridges.txt";

    let working  =  match BRIDGE_TYPE {
        BridgesType::Obfs4 => get_working_obfs4(obfs4_bridges),
        BridgesType::Snowflake => get_working_snowflake(snowflake_bridges),
        BridgesType::Proxy => get_working_proxy(proxy_list)
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

fn get_working_obfs4(bridges: &str) -> Vec<&str> {
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

fn get_working_snowflake(bridges: &str) -> Vec<&str> {
    bridges.trim().lines().filter(|row| {
        let ip = Ipv4Addr::from_str(row)
            .expect("ip addr parse error.\nCant parse: {ip}");
        
        let sock_443 = SocketAddr::new(IpAddr::V4(ip), 443);
        let sock_80 = SocketAddr::new(IpAddr::V4(ip), 80);

        check_conn(&sock_443, Duration::from_millis(CONN_TIMEOUT_MS))
        || check_conn(&sock_80, Duration::from_millis(CONN_TIMEOUT_MS))
    }).collect()
}

fn get_working_proxy(proxies: &str) -> Vec<&str> {
    proxies.trim().lines().filter(|row| {
        let (ip, port) = row.split_once(':').unwrap();
        let ip = Ipv4Addr::from_str(ip)
            .expect("ip addr parse error.\nCant parse: {ip}");
        let port: u16 = port.parse().unwrap();
        let sock = SocketAddr::new(IpAddr::V4(ip), port);
        check_conn(&sock, Duration::from_millis(CONN_TIMEOUT_MS))
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