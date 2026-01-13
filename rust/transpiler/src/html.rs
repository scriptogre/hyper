/// HTML element classification for compile-time validation.

/// Void elements: cannot have children or a closing tag.
/// https://html.spec.whatwg.org/multipage/syntax.html#void-elements
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

/// Elements that auto-close when a block element is encountered inside them.
/// In practice, putting a <div> inside a <p> is always a bug.
const AUTO_CLOSE_ELEMENTS: &[&str] = &[
    "p",
];

/// Block-level elements that cannot appear inside auto-close elements.
const BLOCK_ELEMENTS: &[&str] = &[
    "address", "article", "aside", "blockquote", "details", "dialog",
    "dd", "div", "dl", "dt", "fieldset", "figcaption", "figure",
    "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6",
    "header", "hgroup", "hr", "li", "main", "nav", "ol", "p",
    "pre", "section", "table", "ul",
];

/// Interactive elements that cannot be nested inside themselves.
const INTERACTIVE_ELEMENTS: &[&str] = &[
    "a", "button",
];

pub fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str())
}

pub fn is_auto_close_element(tag: &str) -> bool {
    AUTO_CLOSE_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str())
}

pub fn is_block_element(tag: &str) -> bool {
    BLOCK_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str())
}

pub fn is_interactive_element(tag: &str) -> bool {
    INTERACTIVE_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str())
}
