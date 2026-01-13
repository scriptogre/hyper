use std::collections::HashSet;

/// Metadata collected during transformation
/// This is populated by analysis plugins and used by the generator
#[derive(Debug, Clone, Default)]
pub struct TransformMetadata {
    pub helpers_used: HashSet<String>,
    pub is_async: bool,
    pub slots_used: HashSet<String>,
}

impl TransformMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uses_safe(&self) -> bool {
        self.helpers_used.contains("safe")
    }

    pub fn uses_render_class(&self) -> bool {
        self.helpers_used.contains("render_class")
    }

    pub fn uses_render_style(&self) -> bool {
        self.helpers_used.contains("render_style")
    }

    pub fn uses_render_attr(&self) -> bool {
        self.helpers_used.contains("render_attr")
    }

    pub fn uses_render_data(&self) -> bool {
        self.helpers_used.contains("render_data")
    }

    pub fn uses_render_aria(&self) -> bool {
        self.helpers_used.contains("render_aria")
    }

    pub fn uses_spread_attrs(&self) -> bool {
        self.helpers_used.contains("spread_attrs")
    }

    pub fn uses_children(&self) -> bool {
        self.slots_used.contains("children")
    }

    pub fn get_slot_names(&self) -> Vec<String> {
        self.slots_used.iter().cloned().collect()
    }
}
