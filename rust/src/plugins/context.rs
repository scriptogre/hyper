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

/// Spread names that are automatically injected into the function signature
/// when used as `{**name}` on a component or element without explicit declaration.
pub const BLESSED_SPREAD_NAMES: &[&str] = &["kwargs", "props", "rest", "attrs", "attributes"];
