//! Shared utility functions for language adapters.


use tree_sitter::Node;

/// Collect all named children of a node.
pub fn named_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        children.push(child);
    }
    children
}

/// Get the depth of a node in the tree.
pub fn node_depth(node: &Node) -> usize {
    let mut depth = 0;
    let mut current = *node;
    while let Some(parent) = current.parent() {
        depth += 1;
        current = parent;
    }
    depth
}

/// Find the first ancestor of a given node type.
pub fn find_ancestor<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut current = node.parent()?;
    loop {
        if current.kind() == kind {
            return Some(current);
        }
        current = current.parent()?;
    }
}

/// Extract the text of a node from source code.
pub fn node_text<'a>(node: &Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

/// Check if a node is a leaf (has no children).
pub fn is_leaf(node: &Node) -> bool {
    node.child_count() == 0
}

/// Walk a tree and collect all nodes of a specific kind.
pub fn find_nodes_by_kind<'a>(root: Node<'a>, kind: &str) -> Vec<Node<'a>> {
    let mut results = Vec::new();
    collect_nodes_by_kind(root, kind, &mut results);
    results
}

fn collect_nodes_by_kind<'a>(node: Node<'a>, kind: &str, results: &mut Vec<Node<'a>>) {
    if node.kind() == kind {
        results.push(node);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_nodes_by_kind(child, kind, results);
        }
    }
}

/// Compute the indentation level (number of leading spaces) for a line in source.
pub fn line_indentation(source: &str, byte_offset: usize) -> usize {
    // Find the start of the line containing byte_offset.
    let line_start = source[..byte_offset]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    source[line_start..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_indentation() {
        let source = "fn main() {\n    let x = 1;\n}";
        // "    let x = 1;" starts at byte 12, the 'l' at byte 16
        assert_eq!(line_indentation(source, 16), 4);
    }
}
