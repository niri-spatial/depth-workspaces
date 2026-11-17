use std::collections::BTreeMap;

/// NiriのワークスペースID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub u64);

/// 拡張機能が管理する奥行き空間のID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthId(pub u64);

/// 1つのワークスペースの状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSlot {
    pub id: WorkspaceId,
    pub y: i32,
    pub window_count: usize,
}

/// 1つの奥行き空間
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthPlane {
    pub id: DepthId,
    pub workspaces: BTreeMap<i32, WorkspaceSlot>,
}

/// 削除された空間の情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedPlane {
    pub id: DepthId,
    pub workspace_ids: Vec<WorkspaceId>,
}

/// 空間管理全体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialState {
    planes: Vec<DepthPlane>,
    current_plane_index: usize,
    current_y: i32,
    next_plane_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialError {
    DuplicateWorkspaceId,
    DuplicateYCoordinate,
    UnknownDepthId,
}

impl SpatialState {
    /// 最初のワークスペースから空間管理を開始する
    pub fn new(initial_workspace_id: u64, window_count: usize) -> Self {
        let initial_workspace = WorkspaceSlot {
            id: WorkspaceId(initial_workspace_id),
            y: 0,
            window_count,
        };

        let initial_plane = DepthPlane {
            id: DepthId(0),
            workspaces: BTreeMap::from([(0, initial_workspace)]),
        };

        Self {
            planes: vec![initial_plane],
            current_plane_index: 0,
            current_y: 0,
            next_plane_id: 1,
        }
    }

    /// 管理している空間の数
    pub fn plane_count(&self) -> usize {
        self.planes.len()
    }

    /// 現在の空間ID
    pub fn current_plane_id(&self) -> DepthId {
        self.planes[self.current_plane_index].id
    }

    /// 現在のZ座標
    pub fn current_z(&self) -> isize {
        0
    }

    /// 特定の空間の現在地から見たZ座標
    pub fn relative_z(&self, id: DepthId) -> Option<isize> {
        let index = self.planes.iter().position(|plane| plane.id == id)?;

        Some(index as isize - self.current_plane_index as isize)
    }

    /// 現在のY座標
    pub fn current_y(&self) -> i32 {
        self.current_y
    }

    /// 現在位置に対応するワークスペースID
    pub fn current_workspace_id(&self) -> Option<WorkspaceId> {
        self.planes
            .get(self.current_plane_index)?
            .workspaces
            .get(&self.current_y)
            .map(|workspace| workspace.id)
    }

    /// 新しい空間を生成する
    fn create_plane(&mut self) -> DepthPlane {
        let id = DepthId(self.next_plane_id);
        self.next_plane_id += 1;

        DepthPlane {
            id,
            workspaces: BTreeMap::new(),
        }
    }

    /// 一つ奥に進む
    pub fn move_forward(&mut self) -> DepthId {
        let next_index = self.current_plane_index + 1;

        if next_index >= self.planes.len() {
            let plane = self.create_plane();
            self.planes.push(plane);
        }

        self.current_plane_index = next_index;

        self.current_plane_id()
    }

    /// 一つ手前に戻る
    pub fn move_back(&mut self) -> DepthId {
        if self.current_plane_index == 0 {
            let plane = self.create_plane();
            self.planes.insert(0, plane);
        } else {
            self.current_plane_index -= 1;
        }

        self.current_plane_id()
    }

    /// 指定した空間にワークスペースを登録する
    pub fn add_workspace(
        &mut self,
        depth_id: DepthId,
        y: i32,
        workspace_id: u64,
        window_count: usize,
    ) -> Result<(), SpatialError> {
        if self.planes.iter().any(|plane| {
            plane
                .workspaces
                .values()
                .any(|workspace| workspace.id == WorkspaceId(workspace_id))
        }) {
            return Err(SpatialError::DuplicateWorkspaceId);
        }

        let plane = self
            .planes
            .iter_mut()
            .find(|plane| plane.id == depth_id)
            .ok_or(SpatialError::UnknownDepthId)?;

        if plane.workspaces.contains_key(&y) {
            return Err(SpatialError::DuplicateYCoordinate);
        }

        plane.workspaces.insert(
            y,
            WorkspaceSlot {
                id: WorkspaceId(workspace_id),
                y,
                window_count,
            },
        );

        Ok(())
    }

    /// ワークスペースのウィンドウ数を更新する
    pub fn update_window_count(&mut self, workspace_id: WorkspaceId, count: usize) -> bool {
        for plane in &mut self.planes {
            for workspace in plane.workspaces.values_mut() {
                if workspace.id == workspace_id {
                    workspace.window_count = count;
                    return true;
                }
            }
        }

        false
    }

    /// ワークスペースIDから現在位置を更新する
    pub fn activate_workspace(&mut self, workspace_id: WorkspaceId) -> bool {
        let target = self
            .planes
            .iter()
            .enumerate()
            .find_map(|(plane_index, plane)| {
                plane
                    .workspaces
                    .iter()
                    .find(|(_, workspace)| workspace.id == workspace_id)
                    .map(|(y, _)| (plane_index, *y))
            });

        if let Some((plane_index, y)) = target {
            self.current_plane_index = plane_index;
            self.current_y = y;
            true
        } else {
            false
        }
    }

    /// 使用されていない空間を削除して詰める
    pub fn prune_empty_inactive_planes(&mut self) -> Vec<RemovedPlane> {
        let current_id = self.current_plane_id();
        let mut remaining = Vec::new();
        let mut removed = Vec::new();
        let mut new_current_index = 0;

        for plane in self.planes.drain(..) {
            let is_current = plane.id == current_id;

            let has_windows = plane
                .workspaces
                .values()
                .any(|workspace| workspace.window_count > 0);

            if is_current || has_windows {
                if is_current {
                    new_current_index = remaining.len();
                }

                remaining.push(plane);
            } else {
                removed.push(RemovedPlane {
                    id: plane.id,
                    workspace_ids: plane
                        .workspaces
                        .values()
                        .map(|workspace| workspace.id)
                        .collect(),
                });
            }
        }

        self.planes = remaining;
        self.current_plane_index = new_current_index;

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_one_plane() {
        let state = SpatialState::new(1, 1);

        assert_eq!(state.plane_count(), 1);
        assert_eq!(state.current_z(), 0);
        assert_eq!(state.current_workspace_id(), Some(WorkspaceId(1)));
    }

    #[test]
    fn move_forward_creates_a_new_plane() {
        let mut state = SpatialState::new(1, 1);

        let next = state.move_forward();

        assert_eq!(state.plane_count(), 2);
        assert_eq!(state.current_z(), 0);
        assert_eq!(state.relative_z(DepthId(0)), Some(-1));
        assert_eq!(state.relative_z(next), Some(0));
        assert_eq!(state.current_workspace_id(), None);
    }

    #[test]
    fn move_back_creates_a_new_plane_before_current() {
        let mut state = SpatialState::new(1, 1);

        let previous = state.move_back();

        assert_eq!(state.plane_count(), 2);
        assert_eq!(state.relative_z(previous), Some(0));
        assert_eq!(state.relative_z(DepthId(0)), Some(1));
    }

    #[test]
    fn empty_inactive_plane_is_removed_and_compacted() {
        let mut state = SpatialState::new(1, 1);

        // 中間の空間を作る
        let middle = state.move_forward();

        state.add_workspace(middle, 0, 2, 0).unwrap();

        // さらに奥の空間を作る
        let back = state.move_forward();

        state.add_workspace(back, 0, 3, 1).unwrap();

        state.activate_workspace(WorkspaceId(3));

        // 空の中間空間を削除
        let removed = state.prune_empty_inactive_planes();

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].id, middle);
        assert_eq!(removed[0].workspace_ids, vec![WorkspaceId(2)]);

        assert_eq!(state.plane_count(), 2);
        assert_eq!(state.current_plane_id(), back);
        assert_eq!(state.current_z(), 0);

        // 奥の空間が現在地になり、前の空間は-1になる
        assert_eq!(state.relative_z(DepthId(0)), Some(-1));
        assert_eq!(state.relative_z(back), Some(0));
    }

    #[test]
    fn current_empty_plane_is_preserved() {
        let mut state = SpatialState::new(1, 0);

        let removed = state.prune_empty_inactive_planes();

        assert!(removed.is_empty());
        assert_eq!(state.plane_count(), 1);
    }
}
