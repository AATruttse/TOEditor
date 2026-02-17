//! SVG export functionality for organizational charts

use anyhow::Result;
use crate::models::{Library, Unit};
use std::path::Path;

const BOX_WIDTH: f64 = 160.0;
const BOX_HEIGHT: f64 = 50.0;
const H_SPACING: f64 = 30.0;
const V_SPACING: f64 = 60.0;
const PADDING: f64 = 40.0;

/// Calculated layout node
struct LayoutNode {
    x: f64,
    y: f64,
    label: String,
    sublabel: String,
    children: Vec<LayoutNode>,
}

/// Calculate the total width needed for a unit subtree.
fn subtree_width(unit: &Unit) -> f64 {
    if unit.children.is_empty() {
        BOX_WIDTH
    } else {
        let children_total: f64 = unit
            .children
            .iter()
            .map(subtree_width)
            .sum::<f64>()
            + H_SPACING * (unit.children.len() as f64 - 1.0).max(0.0);
        children_total.max(BOX_WIDTH)
    }
}

/// Layout a unit tree starting at (x, y) centered on the given width.
fn layout_unit(unit: &Unit, x: f64, y: f64, available_width: f64) -> LayoutNode {
    let cx = x + available_width / 2.0;
    let sublabel = format!(
        "P:{} E:{}",
        unit.personnel.len(),
        unit.equipment.iter().map(|e| e.quantity).sum::<usize>()
    );

    let mut children_layouts = Vec::new();

    if !unit.children.is_empty() {
        let total_children_width: f64 = unit
            .children
            .iter()
            .map(subtree_width)
            .sum::<f64>()
            + H_SPACING * (unit.children.len() as f64 - 1.0).max(0.0);

        let start_x = cx - total_children_width / 2.0;
        let child_y = y + BOX_HEIGHT + V_SPACING;
        let mut cur_x = start_x;

        for child in &unit.children {
            let cw = subtree_width(child);
            children_layouts.push(layout_unit(child, cur_x, child_y, cw));
            cur_x += cw + H_SPACING;
        }
    }

    LayoutNode {
        x: cx - BOX_WIDTH / 2.0,
        y,
        label: unit.name.clone(),
        sublabel,
        children: children_layouts,
    }
}

/// Calculate the maximum depth of a layout tree.
fn max_depth(node: &LayoutNode) -> usize {
    if node.children.is_empty() {
        1
    } else {
        1 + node.children.iter().map(max_depth).max().unwrap_or(0)
    }
}

/// Render a layout node and its children to SVG elements.
fn render_node(node: &LayoutNode, elements: &mut Vec<String>) {
    let cx = node.x + BOX_WIDTH / 2.0;
    let cy = node.y + BOX_HEIGHT;

    // Draw connecting lines to children
    for child in &node.children {
        let child_cx = child.x + BOX_WIDTH / 2.0;
        let child_top = child.y;
        let mid_y = cy + V_SPACING / 2.0;

        elements.push(format!(
            r##"  <path d="M{cx},{cy} L{cx},{mid_y} L{child_cx},{mid_y} L{child_cx},{child_top}" fill="none" stroke="#666" stroke-width="1.5"/>"##,
        ));
    }

    // Draw box
    let escaped_label = xml_escape(&node.label);
    let escaped_sub = xml_escape(&node.sublabel);
    elements.push(format!(
        r##"  <rect x="{}" y="{}" width="{}" height="{}" rx="6" ry="6" fill="#f0f4f8" stroke="#4a6fa5" stroke-width="1.5"/>"##,
        node.x, node.y, BOX_WIDTH, BOX_HEIGHT
    ));
    elements.push(format!(
        r##"  <text x="{}" y="{}" text-anchor="middle" font-size="12" font-family="sans-serif" fill="#1a1a2e">{}</text>"##,
        node.x + BOX_WIDTH / 2.0,
        node.y + 20.0,
        escaped_label
    ));
    elements.push(format!(
        r##"  <text x="{}" y="{}" text-anchor="middle" font-size="10" font-family="sans-serif" fill="#666">{}</text>"##,
        node.x + BOX_WIDTH / 2.0,
        node.y + 38.0,
        escaped_sub
    ));

    for child in &node.children {
        render_node(child, elements);
    }
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Export library to SVG organizational chart.
///
/// Renders each top-level unit as a tree with boxes and connecting lines.
/// Produces a valid SVG file with the library name as title.
pub fn export_svg(library: &Library, path: &Path) -> Result<()> {
    let mut elements = Vec::new();

    if library.units.is_empty() {
        // Minimal SVG for empty library
        let title = xml_escape(&library.name);
        let svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="100">
  <text x="200" y="50" text-anchor="middle" font-size="16" font-family="sans-serif">{title} (no units)</text>
</svg>"#
        );
        std::fs::write(path, svg)?;
        return Ok(());
    }

    // Layout all top-level units side by side
    let mut layouts = Vec::new();
    let mut total_width = PADDING;
    for unit in &library.units {
        let w = subtree_width(unit);
        layouts.push(layout_unit(unit, total_width, PADDING + 30.0, w));
        total_width += w + H_SPACING * 2.0;
    }
    total_width += PADDING;

    let depth = layouts.iter().map(max_depth).max().unwrap_or(1);
    let total_height = PADDING * 2.0 + 30.0 + (depth as f64) * (BOX_HEIGHT + V_SPACING);

    // Title
    let title = xml_escape(&library.name);
    elements.push(format!(
        r##"  <text x="{}" y="30" text-anchor="middle" font-size="18" font-weight="bold" font-family="sans-serif" fill="#1a1a2e">{}</text>"##,
        total_width / 2.0,
        title
    ));

    for layout in &layouts {
        render_node(layout, &mut elements);
    }

    let svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n{}\n</svg>",
        total_width as i64,
        total_height as i64,
        elements.join("\n")
    );

    std::fs::write(path, svg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Library, Unit, Personnel, Equipment};
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_svg_empty() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let file = NamedTempFile::new().unwrap();
        export_svg(&library, file.path()).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("svg"));
        assert!(content.contains("no units"));
    }

    #[test]
    fn test_export_svg_with_units() {
        let mut library = Library::new(
            "US Army".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let mut platoon = Unit::new("1st Platoon".to_string(), "platoon".to_string());
        platoon.personnel.push(Personnel::new("Leader".to_string()));
        platoon.equipment.push(Equipment::new("Rifle".to_string(), 10));

        let squad1 = Unit::new("Alpha Squad".to_string(), "squad".to_string());
        let squad2 = Unit::new("Bravo Squad".to_string(), "squad".to_string());
        platoon.children.push(squad1);
        platoon.children.push(squad2);

        library.units.push(platoon);

        let file = NamedTempFile::new().unwrap();
        export_svg(&library, file.path()).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("US Army"));
        assert!(content.contains("1st Platoon"));
        assert!(content.contains("Alpha Squad"));
        assert!(content.contains("Bravo Squad"));
        assert!(content.contains("rect"));
        assert!(content.contains("path"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_subtree_width_leaf() {
        let unit = Unit::new("Leaf".to_string(), "squad".to_string());
        assert_eq!(subtree_width(&unit), BOX_WIDTH);
    }

    #[test]
    fn test_subtree_width_with_children() {
        let mut parent = Unit::new("Parent".to_string(), "platoon".to_string());
        parent.children.push(Unit::new("A".to_string(), "squad".to_string()));
        parent.children.push(Unit::new("B".to_string(), "squad".to_string()));
        let expected = BOX_WIDTH * 2.0 + H_SPACING;
        assert!((subtree_width(&parent) - expected).abs() < 0.01);
    }
}
