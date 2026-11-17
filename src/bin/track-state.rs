use niri_ipc::socket::Socket;
use niri_ipc::state::{EventStreamState, EventStreamStatePart};
use niri_ipc::{Event, Request, Response};

fn main() -> std::io::Result<()> {
    let mut socket = Socket::connect()?;

    // イベントストリームを開始
    let reply = socket.send(Request::EventStream)?;

    match reply {
        Ok(Response::Handled) => {
            println!("Connected to Niri.");
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

    // Niriの状態を保持する
    let mut state = EventStreamState::default();
    let mut read_event = socket.read_events();

    loop {
        let event = read_event()?;

        // 表示するイベントだけ選択
        let event_name = match &event {
            Event::WorkspacesChanged { .. } => Some("WorkspacesChanged"),
            Event::WorkspaceActivated { .. } => Some("WorkspaceActivated"),
            Event::WindowsChanged { .. } => Some("WindowsChanged"),
            Event::WindowOpenedOrChanged { .. } => Some("WindowOpenedOrChanged"),
            Event::WindowClosed { .. } => Some("WindowClosed"),
            _ => None,
        };

        // イベントを状態に反映
        state.apply(event);

        if let Some(name) = event_name {
            println!("\n=== {name} ===");
            println!("Total workspaces: {}", state.workspaces.workspaces.len());

            let mut workspaces: Vec<_> = state.workspaces.workspaces.values().collect();

            workspaces.sort_by_key(|workspace| (workspace.output.clone(), workspace.idx));

            for workspace in workspaces {
                let window_count = state
                    .windows
                    .windows
                    .values()
                    .filter(|window| window.workspace_id == Some(workspace.id))
                    .count();

                let status = if workspace.is_focused {
                    "FOCUSED"
                } else if workspace.is_active {
                    "ACTIVE"
                } else {
                    "INACTIVE"
                };

                println!(
                    "ID: {} | Index: {} | Output: {} | Windows: {} | {}",
                    workspace.id,
                    workspace.idx,
                    workspace.output.as_deref().unwrap_or("-"),
                    window_count,
                    status
                );
            }
        }
    }
}
