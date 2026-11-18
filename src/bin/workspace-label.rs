use std::{env, error::Error, io, process};

use depth_workspaces::workspace_name::encode;
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Request, Response, Workspace, WorkspaceReferenceArg};

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [command, id, z, y] if command == "set" => {
            let id = id.parse::<u64>()?;
            let z = z.parse::<i32>()?;
            let y = y.parse::<i32>()?;

            set_workspace_name(id, z, y)
        }
        [command, z, y] if command == "unset" => {
            let z = z.parse::<i32>()?;
            let y = y.parse::<i32>()?;

            unset_workspace_name(z, y)
        }
        [command, z, y] if command == "adopt-current" => {
            let z = z.parse::<i32>()?;
            let y = y.parse::<i32>()?;

            adopt_current_workspace(z, y)
        }
        _ => {
            println!("Usage:");
            println!("  workspace-label set <workspace-id> <z> <y>");
            println!("  workspace-label unset <z> <y>");
            println!("  workspace-label adopt-current <z> <y>");
            Ok(())
        }
    }
}

fn get_workspaces(socket: &mut Socket) -> Result<Vec<Workspace>, Box<dyn Error>> {
    match socket.send(Request::Workspaces)? {
        Ok(Response::Workspaces(workspaces)) => Ok(workspaces),
        Ok(other) => Err(io::Error::other(format!("Unexpected response: {other:?}")).into()),
        Err(message) => Err(io::Error::other(message).into()),
    }
}

fn send_action(socket: &mut Socket, action: Action) -> Result<(), Box<dyn Error>> {
    match socket.send(Request::Action(action))? {
        Ok(Response::Handled) => Ok(()),
        Ok(other) => Err(io::Error::other(format!("Unexpected response: {other:?}")).into()),
        Err(message) => Err(io::Error::other(message).into()),
    }
}

/// 指定されたIDのワークスペースに座標名を設定する
fn set_workspace_name(workspace_id: u64, z: i32, y: i32) -> Result<(), Box<dyn Error>> {
    let mut socket = Socket::connect()?;
    let workspaces = get_workspaces(&mut socket)?;

    let name = encode(z, y);

    let target = workspaces
        .iter()
        .find(|workspace| workspace.id == workspace_id)
        .ok_or_else(|| io::Error::other(format!("Workspace ID {workspace_id} was not found.")))?;

    // すでに同じ名前なら何もしない
    if target.name.as_deref() == Some(name.as_str()) {
        println!("Workspace already has this coordinate.");
        return Ok(());
    }

    // 既存の名前を上書きしない
    if let Some(existing_name) = &target.name {
        return Err(
            io::Error::other(format!("Workspace already has a name: {existing_name}")).into(),
        );
    }

    // ウィンドウがあるワークスペースには設定しない
    if target.active_window_id.is_some() {
        return Err(io::Error::other("Target workspace is not empty.").into());
    }

    // 同じ座標がすでに使われていないか確認
    if let Some(existing) = workspaces
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(name.as_str()))
    {
        return Err(io::Error::other(format!(
            "Coordinate already exists on workspace ID {}.",
            existing.id
        ))
        .into());
    }

    let output = target.output.as_deref().unwrap_or("-");

    println!("Assigning workspace ID {workspace_id} on {output} to ({z}, {y})");

    // IDを指定してNiriに名前を設定
    send_action(
        &mut socket,
        Action::SetWorkspaceName {
            name: name.clone(),
            workspace: Some(WorkspaceReferenceArg::Id(workspace_id)),
        },
    )?;

    // 操作後の状態を再取得する
    let updated = get_workspaces(&mut socket)?;

    let named = updated
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(name.as_str()))
        .ok_or_else(|| io::Error::other("Workspace name could not be verified."))?;

    println!("Workspace name assigned successfully.");
    println!("Name: {}", name);
    println!("Current ID: {}", named.id);
    println!("Current index: {}", named.idx);

    Ok(())
}

/// 現在フォーカスしているワークスペースを3D空間に登録する
fn adopt_current_workspace(z: i32, y: i32) -> Result<(), Box<dyn Error>> {
    let mut socket = Socket::connect()?;
    let workspaces = get_workspaces(&mut socket)?;

    let name = encode(z, y);

    // 現在フォーカスしているワークスペースを取得
    let target = workspaces
        .iter()
        .find(|workspace| workspace.is_focused)
        .ok_or_else(|| io::Error::other("Focused workspace was not found."))?;

    let workspace_id = target.id;

    // すでに同じ座標なら何もしない
    if target.name.as_deref() == Some(name.as_str()) {
        println!("Workspace is already registered.");
        return Ok(());
    }

    // 既存の名前を上書きしない
    if let Some(existing_name) = &target.name {
        return Err(io::Error::other(format!(
            "Focused workspace already has a name: {existing_name}"
        ))
        .into());
    }

    // 同じ座標の重複を防止
    if let Some(existing) = workspaces
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(name.as_str()))
    {
        return Err(io::Error::other(format!(
            "Coordinate already exists on workspace ID {}.",
            existing.id
        ))
        .into());
    }

    println!("Adopting focused workspace ID {workspace_id} as ({z}, {y})");

    // 現在のワークスペースに名前を設定
    send_action(
        &mut socket,
        Action::SetWorkspaceName {
            name: name.clone(),
            workspace: Some(WorkspaceReferenceArg::Id(workspace_id)),
        },
    )?;

    // 設定結果を確認
    let updated = get_workspaces(&mut socket)?;

    let named = updated
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(name.as_str()))
        .ok_or_else(|| io::Error::other("Workspace registration failed."))?;

    println!("Workspace registered successfully.");
    println!("Name: {}", name);
    println!("Current ID: {}", named.id);

    Ok(())
}

/// 指定された座標の名前を解除する
fn unset_workspace_name(z: i32, y: i32) -> Result<(), Box<dyn Error>> {
    let mut socket = Socket::connect()?;
    let name = encode(z, y);

    let workspaces = get_workspaces(&mut socket)?;

    let target = workspaces
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(name.as_str()));

    if target.is_none() {
        println!("Workspace is already unassigned: {name}");
        return Ok(());
    }

    println!("Removing workspace name: {name}");

    // 名前を指定して解除する
    send_action(
        &mut socket,
        Action::UnsetWorkspaceName {
            reference: Some(WorkspaceReferenceArg::Name(name.clone())),
        },
    )?;

    // 解除されたことを確認
    let updated = get_workspaces(&mut socket)?;

    if updated
        .iter()
        .any(|workspace| workspace.name.as_deref() == Some(name.as_str()))
    {
        return Err(io::Error::other("Workspace name still exists after unset.").into());
    }

    println!("Workspace name removed successfully.");

    Ok(())
}
