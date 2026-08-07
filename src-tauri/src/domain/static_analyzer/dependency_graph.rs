//! 依存関係グラフ構築（03_設計書.md §1.2/§4）。fan-in/fan-out算出・循環依存検出を行う。
//!
//! 注意: import文からのパス解決はTypeScript/JavaScript/Python的な相対パス記法
//! （`./foo`, `../foo/bar`）を対象としたベストエフォート実装であり、Java/Go/C#等の
//! パッケージ・名前空間ベースの解決は行わない（実装しても不正確なデータになるより、
//! 対象外として空のまま返すほうが安全なため）。

use crate::models::{DependencyEdge, DependencyGraph, DependencyNode, StaticAnalysisResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// import文字列中の相対パスらしき部分（クォート内の `.`始まりの文字列）を抜き出す
static RELATIVE_IMPORT_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"['"](\.[^'"]*)['"]"#).unwrap());

pub fn build(results: &[StaticAnalysisResult]) -> DependencyGraph {
    let known_paths: HashSet<&str> = results.iter().map(|r| r.file_path.as_str()).collect();

    let mut edges: Vec<DependencyEdge> = Vec::new();
    let mut fan_out: HashMap<String, u32> = HashMap::new();
    let mut fan_in: HashMap<String, u32> = HashMap::new();
    for r in results {
        fan_out.entry(r.file_path.clone()).or_insert(0);
        fan_in.entry(r.file_path.clone()).or_insert(0);
    }

    for result in results {
        for import_stmt in &result.imports {
            let Some(caps) = RELATIVE_IMPORT_PATH.captures(import_stmt) else {
                continue;
            };
            let rel = &caps[1];
            if let Some(resolved) = resolve_relative(&result.file_path, rel, &known_paths) {
                if resolved == result.file_path {
                    continue; // 自己参照は無視
                }
                edges.push(DependencyEdge {
                    from: result.file_path.clone(),
                    to: resolved.clone(),
                });
                *fan_out.entry(result.file_path.clone()).or_insert(0) += 1;
                *fan_in.entry(resolved).or_insert(0) += 1;
            }
        }
    }

    let nodes: Vec<DependencyNode> = results
        .iter()
        .map(|r| DependencyNode {
            id: r.file_path.clone(),
            fan_in: *fan_in.get(&r.file_path).unwrap_or(&0),
            fan_out: *fan_out.get(&r.file_path).unwrap_or(&0),
        })
        .collect();

    let cycles = detect_cycles(&nodes, &edges);

    DependencyGraph { nodes, edges, cycles }
}

fn resolve_relative(from_file: &str, rel_import: &str, known_paths: &HashSet<&str>) -> Option<String> {
    let from_dir = std::path::Path::new(from_file).parent()?;
    let joined = from_dir.join(rel_import);
    let normalized = normalize_path(&joined);

    // 拡張子省略記法（`./foo` が `foo.ts` を指す等）に対応するため、既知パスの中から
    // 「拡張子を除いた部分が一致する」ものを探す。
    known_paths
        .iter()
        .find(|p| {
            let p_no_ext = strip_known_extension(p);
            p_no_ext == normalized || **p == normalized.as_str()
        })
        .map(|p| p.to_string())
}

fn normalize_path(path: &std::path::Path) -> String {
    let mut parts: Vec<&std::ffi::OsStr> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(s) => parts.push(s),
            _ => {}
        }
    }
    parts
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn strip_known_extension(path: &str) -> String {
    for ext in [".ts", ".tsx", ".js", ".jsx", ".py", ".rs"] {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    path.to_string()
}

/// 単純なDFSによる循環依存検出（訪問中ノードへの再訪＝閉路）。
fn detect_cycles(nodes: &[DependencyNode], edges: &[DependencyEdge]) -> Vec<Vec<String>> {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        adjacency.entry(edge.from.as_str()).or_default().push(edge.to.as_str());
    }

    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Unvisited,
        Visiting,
        Done,
    }

    let mut state: HashMap<&str, State> = nodes.iter().map(|n| (n.id.as_str(), State::Unvisited)).collect();
    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut stack: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        adjacency: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, State>,
        stack: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        state.insert(node, State::Visiting);
        stack.push(node);

        if let Some(neighbors) = adjacency.get(node) {
            for &next in neighbors {
                match state.get(next).copied().unwrap_or(State::Unvisited) {
                    State::Unvisited => dfs(next, adjacency, state, stack, cycles),
                    State::Visiting => {
                        // stack中のnextからnodeまでが閉路
                        if let Some(pos) = stack.iter().position(|&n| n == next) {
                            let cycle: Vec<String> = stack[pos..].iter().map(|s| s.to_string()).collect();
                            cycles.push(cycle);
                        }
                    }
                    State::Done => {}
                }
            }
        }

        stack.pop();
        state.insert(node, State::Done);
    }

    for node in nodes {
        if state.get(node.id.as_str()).copied().unwrap_or(State::Unvisited) == State::Unvisited {
            dfs(node.id.as_str(), &adjacency, &mut state, &mut stack, &mut cycles);
        }
    }

    cycles
}
