use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

#[derive(Debug, Clone, Default)]
pub struct ComponentGroup {
    ordered: Vec<ComponentId>,
    names: HashMap<String, ComponentId>,
    group_indexes: HashMap<i32, ComponentId>,
}

impl ComponentGroup {
    pub fn register(&mut self, id: ComponentId, name: impl Into<String>) {
        self.register_with_group_index(id, name, None);
    }

    pub fn register_with_group_index(
        &mut self,
        id: ComponentId,
        name: impl Into<String>,
        group_index: Option<i32>,
    ) {
        let name = name.into();
        if !self.ordered.contains(&id) {
            self.ordered.push(id);
        }
        self.names.insert(name, id);
        if let Some(group_index) = group_index {
            self.group_indexes.insert(group_index, id);
        }
    }

    pub fn get(&self, index: usize) -> Option<ComponentId> {
        self.ordered.get(index).copied()
    }

    pub fn find(&self, name: &str) -> Option<ComponentId> {
        self.names.get(name).copied()
    }

    pub fn find_by_group_index(&self, group_index: i32) -> Option<ComponentId> {
        self.group_indexes.get(&group_index).copied()
    }

    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.ordered.iter().copied()
    }
}
