use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};

fn main() -> std::io::Result<()> {
    // Niri IPCに接続
    let mut socket = Socket::connect()?;

    // イベントストリームを要求
    let reply = socket.send(Request::EventStream)?;

    match reply {
        Ok(Response::Handled) => {
            println!("Connected to Niri event stream.");
            println!("Waiting for events... (Ctrl+C to stop)");
        }
        Ok(other) => {
            eprintln!("Unexpected response: {other:?}");
            return Ok(());
        }
        Err(message) => {
            eprintln!("Niri IPC error: {message}");
            return Ok(());
        }
    }

    // イベントを受信し続ける
    let mut read_event = socket.read_events();

    loop {
        match read_event() {
            Ok(event) => {
                println!("{event:#?}");
            }
            Err(error) => {
                eprintln!("Event stream error: {error}");
                return Err(error);
            }
        }
    }
}
