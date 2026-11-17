/// depth-workspacesが管理するワークスペース名の形式
const PREFIX: &str = "depth-workspaces:v1:";

/// ワークスペースの論理座標
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceCoordinates {
    pub z: i32,
    pub y: i32,
}

/// 座標をNiriのワークスペース名に変換する
pub fn encode(z: i32, y: i32) -> String {
    format!("{PREFIX}z={z}:y={y}")
}

/// Niriのワークスペース名から座標を取得する
pub fn decode(name: &str) -> Option<WorkspaceCoordinates> {
    let value = name.strip_prefix(PREFIX)?;
    let (z, y) = value.split_once(":y=")?;

    let z = z.strip_prefix("z=")?.parse::<i32>().ok()?;
    let y = y.parse::<i32>().ok()?;

    Some(WorkspaceCoordinates { z, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_coordinates() {
        assert_eq!(encode(1, 2), "depth-workspaces:v1:z=1:y=2");
    }

    #[test]
    fn decodes_coordinates() {
        assert_eq!(
            decode("depth-workspaces:v1:z=1:y=2"),
            Some(WorkspaceCoordinates { z: 1, y: 2 })
        );
    }

    #[test]
    fn supports_negative_coordinates() {
        assert_eq!(
            decode("depth-workspaces:v1:z=-2:y=-3"),
            Some(WorkspaceCoordinates { z: -2, y: -3 })
        );
    }

    #[test]
    fn rejects_unmanaged_workspace_names() {
        assert_eq!(decode("Development"), None);
        assert_eq!(decode("Browser"), None);
    }

    #[test]
    fn rejects_invalid_names() {
        assert_eq!(decode("depth-workspaces:v2:z=1:y=2"), None);

        assert_eq!(decode("depth-workspaces:v1:z=abc:y=2"), None);
    }
}
