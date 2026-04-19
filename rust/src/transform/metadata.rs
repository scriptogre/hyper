use std::collections::HashSet;

/// Runtime helpers that can be imported from the hyper runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Helper {
    Escape,
    Safe,
    RenderClass,
    RenderStyle,
    RenderAttr,
    RenderData,
    RenderAria,
    SpreadAttrs,
}

impl Helper {
    /// The Python import name for this helper
    pub fn import_name(&self) -> &'static str {
        match self {
            Helper::Escape => "escape",
            Helper::Safe => "safe",
            Helper::RenderClass => "render_class",
            Helper::RenderStyle => "render_style",
            Helper::RenderAttr => "render_attr",
            Helper::RenderData => "render_data",
            Helper::RenderAria => "render_aria",
            Helper::SpreadAttrs => "spread_attrs",
        }
    }

    /// All helper variants, in import order
    pub const ALL: &'static [Helper] = &[
        Helper::Escape,
        Helper::Safe,
        Helper::RenderClass,
        Helper::RenderStyle,
        Helper::RenderAttr,
        Helper::RenderData,
        Helper::RenderAria,
        Helper::SpreadAttrs,
    ];
}

/// Metadata collected during transformation
/// This is populated by analysis plugins and used by the generator
#[derive(Debug, Clone, Default)]
pub struct TransformMetadata {
    pub helpers_used: HashSet<Helper>,
    pub is_async: bool,
    pub slots_used: HashSet<String>,
}

impl TransformMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uses_children(&self) -> bool {
        self.slots_used.contains("children")
    }

    pub fn get_slot_names(&self) -> Vec<String> {
        self.slots_used.iter().cloned().collect()
    }
}
