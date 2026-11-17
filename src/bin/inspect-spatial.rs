use depth_workspaces::workspace_name::decode;
use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};

fn main() -> std::io::Result<()> {
    let mut socket = Socket::connect()?;

    let workspaces = match socket.send(Request::Workspaces)? {
        Ok(Response::Workspaces(workspaces)) => workspaces,
        Ok(other) => {
            eprintln!("Unexpected response: {other:?}");
            return Ok(());
        }
        Err(message) => {
            eprintln!("Niri IPC error: {message}");
            return Ok(());
        }
    };

    println!("=== Spatial Workspace Inspector ===");
    println!("Total: {}", workspaces.len());

    for workspace in workspaces {
        let position = workspace.name.as_deref().and_then(decode);

        match position {
            Some(position) => {
                println!(
                    "ID: {} | Z: {} | Y: {} | Output: {}",
                    workspace.id,
                    position.z,
                    position.y,
                    workspace.output.as_deref().unwrap_or("-"),
                );
            }
            None => {
                println!(
                    "ID: {} | UNASSIGNED | Index: {} | Name: {}",
                    workspace.id,
                    workspace.idx,
                    workspace.name.as_deref().unwrap_or("-"),
                );
            }
        }
    }

    Ok(())
}
