//! ディレクトリ構造ツリーの組み立て（04_実装詳細.md §11、CLAUDE.md原則2.2.1）。
//! 新規のファイル走査は行わず、Phase1のスキャン結果（`ScanResult.files`）をそのまま再利用する
//! 純粋関数（I/Oなし）。

use crate::models::{DirectoryNode, ScannedFile};

/// スキャン済みファイル一覧からネスト構造のディレクトリツリーを構築する。
pub fn build(files: &[ScannedFile], root_name: &str) -> DirectoryNode {
    let mut root = DirectoryNode {
        name: root_name.to_string(),
        path: String::new(),
        kind: "dir".to_string(),
        children: Vec::new(),
        size_bytes: None,
    };

    for file in files {
        insert(&mut root, &file.path, file.size_bytes);
    }

    sort_children(&mut root);
    root
}

fn insert(root: &mut DirectoryNode, relative_path: &str, size_bytes: u64) {
    let parts: Vec<&str> = relative_path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current = root;
    let mut path_so_far = String::new();

    for (i, part) in parts.iter().enumerate() {
        if !path_so_far.is_empty() {
            path_so_far.push('/');
        }
        path_so_far.push_str(part);

        let is_last = i == parts.len() - 1;
        let existing_index = current.children.iter().position(|c| c.name == *part);

        let index = match existing_index {
            Some(idx) => idx,
            None => {
                current.children.push(DirectoryNode {
                    name: part.to_string(),
                    path: path_so_far.clone(),
                    kind: if is_last { "file".to_string() } else { "dir".to_string() },
                    children: Vec::new(),
                    size_bytes: if is_last { Some(size_bytes) } else { None },
                });
                current.children.len() - 1
            }
        };

        current = &mut current.children[index];
    }
}

fn sort_children(node: &mut DirectoryNode) {
    node.children.sort_by(|a, b| match (a.kind.as_str(), b.kind.as_str()) {
        ("dir", "file") => std::cmp::Ordering::Less,
        ("file", "dir") => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    for child in &mut node.children {
        sort_children(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> ScannedFile {
        ScannedFile {
            path: path.to_string(),
            hash: "h".into(),
            size_bytes: 10,
            rpa_tool: None,
            macro_tool: None,
        }
    }

    #[test]
    fn builds_nested_tree_with_dirs_before_files() {
        let files = vec![file("src/main.rs"), file("README.md"), file("src/lib/util.rs")];
        let tree = build(&files, "myproject");

        assert_eq!(tree.name, "myproject");
        assert_eq!(tree.children.len(), 2); // "src" (dir) と "README.md" (file)
        assert_eq!(tree.children[0].name, "src");
        assert_eq!(tree.children[0].kind, "dir");
        assert_eq!(tree.children[1].name, "README.md");
        assert_eq!(tree.children[1].kind, "file");

        let src = &tree.children[0];
        assert_eq!(src.children.len(), 2);
        assert_eq!(src.children[0].name, "lib");
        assert_eq!(src.children[1].name, "main.rs");

        let lib_dir = &src.children[0];
        assert_eq!(lib_dir.children.len(), 1);
        assert_eq!(lib_dir.children[0].name, "util.rs");
        assert_eq!(lib_dir.children[0].path, "src/lib/util.rs");
    }
}
