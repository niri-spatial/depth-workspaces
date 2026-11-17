use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};

fn main() -> std::io::Result<()> {
    // NiriのIPCソケットに接続
    let mut socket = Socket::connect()?;

    // ワークスペース情報を取得
    let reply = socket.send(Request::Workspaces)?;

    // Niriからの応答を確認
    let workspaces = match reply {
        Ok(Response::Workspaces(workspaces)) => workspaces,
        Ok(other) => {
            eprintln!("予期しない応答: {other:?}");
            return Ok(());
        }
        Err(message) => {
            eprintln!("Niri IPCエラー: {message}");
            return Ok(());
        }
    };

    println!("=== Niri Workspaces ===");
    println!("Total: {}", workspaces.len());

    for workspace in workspaces {
        let state = if workspace.is_focused {
            "FOCUSED"
        } else if workspace.is_active {
            "ACTIVE"
        } else {
            "INACTIVE"
        };

        println!(
            "ID: {} | Index: {} | Output: {} | Name: {} | State: {}",
            workspace.id,
            workspace.idx,
            workspace.output.as_deref().unwrap_or("-"),
            workspace.name.as_deref().unwrap_or("-"),
            state,
        );
    }

    Ok(())
}
