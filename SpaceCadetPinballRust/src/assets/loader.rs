use super::DatFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderMetadata {
    pub app_name: String,
    pub description: String,
    pub group_count: usize,
    pub named_groups: Vec<String>,
}

impl LoaderMetadata {
    pub fn has_group(&self, group_name: &str) -> bool {
        self.named_groups.iter().any(|name| name == group_name)
    }
}

impl DatFile {
    pub fn loader_metadata(&self) -> LoaderMetadata {
        LoaderMetadata {
            app_name: self.app_name.clone(),
            description: self.description.clone(),
            group_count: self.groups.len(),
            named_groups: self
                .groups
                .iter()
                .filter_map(|group| group.group_name.clone())
                .collect(),
        }
    }
}

pub fn extract_loader_metadata(dat_file: &DatFile) -> LoaderMetadata {
    dat_file.loader_metadata()
}
