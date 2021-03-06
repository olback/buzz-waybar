use {
    return_data::ReturnData,
    serde_json,
    std::{
        collections::HashMap,
        env, fs,
        io::{self, Read, Write},
        net::Shutdown,
        os::unix::net::{UnixListener, UnixStream},
        path::PathBuf,
        sync::{Arc, Mutex},
        thread,
    },
};

mod config;
mod fetch;
mod return_data;

fn main() -> Result<(), io::Error> {
    let servers = config::load_servers()?;
    let ret_data = Arc::new(Mutex::new(HashMap::<String, Result<usize, String>>::new()));
    let socket_path = env::var("SWAYSOCK")
        .map(|path| {
            let mut new_path = PathBuf::from(path);
            new_path.set_extension("buzz.sock");
            new_path
        })
        .expect("Failed to get $SWAYSOCK path");

    if env::args().nth(1) == Some("--server".to_string()) {
        let ac_ret_data = Arc::clone(&ret_data);
        thread::spawn(move || fetch::runner(servers, ac_ret_data));
        // Server
        if PathBuf::from(&socket_path).exists() {
            fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        for maybe_stream in listener.incoming() {
            let mut stream = maybe_stream?;
            let rd = ret_data.lock().expect("Worker thread toxic").clone();
            stream.write(
                serde_json::to_string(&ReturnData::new(&rd))
                    .expect("Failed to serialize json")
                    .as_str()
                    .as_bytes(),
            )?;
            stream.shutdown(Shutdown::Both)?;
        }
    } else {
        // Client
        let mut client = UnixStream::connect(&socket_path)?;
        let mut buf = [0u8; 2048];
        let len = client.read(&mut buf)?;
        let json_str = String::from_utf8_lossy(&buf[0..len]);
        println!("{}", json_str);
    }

    Ok(())
}
