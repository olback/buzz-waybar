use {
    super::config::Server,
    native_tls,
    std::{
        collections::HashMap,
        net::TcpStream,
        process::{Command, Stdio},
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    },
};

type ImapSession = imap::Session<native_tls::TlsStream<TcpStream>>;

pub fn runner(servers: HashMap<String, Server>, return_data: Arc<Mutex<HashMap<String, usize>>>) {
    let mut connections =
        HashMap::<String, (Server, Option<ImapSession>)>::with_capacity(servers.len());
    for (key, server) in servers {
        let tls = native_tls::TlsConnector::builder()
            .build()
            .expect("Failed to create TLS connector");
        let client = match imap::connect(
            (server.address.clone(), server.port),
            server.address.clone(),
            &tls,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let password = match Command::new("sh")
            .arg("-c")
            .arg(&server.password_cmd)
            .stdout(Stdio::piped())
            .output()
        {
            Ok(output) => {
                let pass = String::from_utf8_lossy(&output.stdout);
                if server.trim_password {
                    pass.trim().to_string()
                } else {
                    pass.to_string()
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let imap_session = match client.login(server.username.clone(), password) {
            Ok(is) => is,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }
        };

        println!("Connected to {} ({}:{})", key, server.address, server.port);
        connections.insert(key, (server.clone(), Some(imap_session)));
    }

    loop {
        // println!("{:#?}", connections);
        for (name, (server, con)) in &mut connections {
            if let Some(c) = con {
                let _ = c.select(&server.folder);
                let uids = c.uid_search("UNSEEN 1:*").unwrap();
                let num_unseen = uids.len();
                let mut lock = return_data.lock().expect("Toxic lock");
                if let Some(last_unseen) = lock.get(name) {
                    if num_unseen > *last_unseen && server.notification_cmd.is_some() {
                        let _ = Command::new("sh")
                            .arg("-c")
                            .arg(
                                server
                                    .notification_cmd
                                    .clone()
                                    .unwrap()
                                    .replace("{name}", name)
                                    .replace("{username}", &server.username)
                                    // .replace("{subject}", "TODO")
                                    // .replace("{from}", "TODO"),
                            )
                            .spawn();
                    }
                }
                lock.insert(name.clone(), num_unseen);
            }
        }
        if let Some(time) = std::env::var("BUZZ_SLEEP")
            .ok()
            .and_then(|time| time.parse::<u64>().ok())
        {
            thread::sleep(Duration::from_secs(time));
        }
    }
}
