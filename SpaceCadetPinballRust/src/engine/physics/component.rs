use crate::gameplay::components::ComponentId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionComponentMetadata {
    pub component_id: ComponentId,
    pub group_index: i32,
    pub collision_group: u32,
    pub smoothness: f32,
    pub elasticity: f32,
    pub threshold: f32,
    pub boost: f32,
    pub soft_hit_sound_id: i32,
    pub hard_hit_sound_id: i32,
    pub wall_float_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CollisionComponentRegistry {
    components: Vec<CollisionComponentMetadata>,
}

impl CollisionComponentRegistry {
    pub fn register(&mut self, metadata: CollisionComponentMetadata) {
        if let Some(existing) = self
            .components
            .iter_mut()
            .find(|component| component.component_id == metadata.component_id)
        {
            *existing = metadata;
        } else {
            self.components.push(metadata);
        }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CollisionComponentMetadata> {
        self.components.iter()
    }
}
