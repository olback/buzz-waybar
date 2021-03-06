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

// TODO:
// enum ShouldRetry {
//     Yes(String),
//     No(String)
// }

// let mut connections = HashMap::<String, (Server, Result<ImapSession, Result<usize, ShouldRetry>>)>::with_capacity(servers.len());

type ImapSession = imap::Session<native_tls::TlsStream<TcpStream>>;

pub fn runner(
    servers: HashMap<String, Server>,
    return_data: Arc<Mutex<HashMap<String, Result<usize, String>>>>,
) {
    let mut connections =
        HashMap::<String, (Server, Result<ImapSession, String>)>::with_capacity(servers.len());
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
                connections.insert(key, (server.clone(), Err(format!("{}", e))));
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
                // println!("{}", output.status);
                if output.status.success() {
                    let pass = String::from_utf8_lossy(&output.stdout);
                    if server.trim_password {
                        pass.trim().to_string()
                    } else {
                        pass.to_string()
                    }
                } else {
                    connections.insert(
                        key,
                        (
                            server.clone(),
                            Err(format!(
                                "Password command failed with status: {}",
                                output.status
                            )),
                        ),
                    );
                    continue;
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                connections.insert(key, (server.clone(), Err(format!("{:?}", e))));
                continue;
            }
        };

        match client.login(server.username.clone(), password) {
            Ok(imap_session) => {
                println!("Connected to {} ({}:{})", key, server.address, server.port);
                connections.insert(key, (server.clone(), Ok(imap_session)));
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to {} ({}:{}): {:#?}",
                    key, server.address, server.port, e
                );
                connections.insert(key, (server.clone(), Err(format!("{:?}", e))));
            }
        }
    }

    loop {
        println!("{:#?}", connections);
        for (name, (server, con)) in &mut connections {
            match con {
                Ok(c) => {
                    let _ = c.select(&server.folder); // TODO: Handle this error! c.select().and_then()?
                    let uids = c.uid_search("UNSEEN 1:*").unwrap(); // TODO: Handle this error!
                    let num_unseen = uids.len();
                    let mut lock = return_data.lock().expect("Toxic lock");
                    if let Some(last_unseen) = lock.get(name) {
                        if Ok(num_unseen) > *last_unseen && server.notification_cmd.is_some() {
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
                    lock.insert(name.clone(), Ok(num_unseen));
                }
                Err(e) => {
                    // TODO: Retry connection
                    let mut lock = return_data.lock().expect("Toxic lock");
                    lock.insert(name.clone(), Err(format!("{:?}", e)));
                }
            }
        }
        let time = std::env::var("BUZZ_SLEEP")
            .map(|s| s.parse::<u64>().unwrap_or(60))
            .unwrap_or(60);
        thread::sleep(Duration::from_secs(time));
    }
}
