//! 言語別tree-sitter設定（03_設計書.md §1.2「言語判定、メトリクス算出、import/export解析」）。
//! 言語追加はこのテーブルに1エントリ加えるだけでよい構造とする（CLAUDE.md 原則2.2.2）。

use tree_sitter::Language;

pub struct LanguageSpec {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub language_fn: fn() -> Language,
    /// 関数として抽出するノード種別
    pub function_kinds: &'static [&'static str],
    /// クラス（相当）として抽出するノード種別
    pub class_kinds: &'static [&'static str],
    /// 循環的複雑度の分岐点として数えるノード種別
    pub decision_kinds: &'static [&'static str],
    /// ネスト深度算出のためのブロック相当ノード種別
    pub block_kinds: &'static [&'static str],
    /// import文として抽出するノード種別
    pub import_kinds: &'static [&'static str],
    /// export文として抽出するノード種別（言語によっては空。例: Python）
    pub export_kinds: &'static [&'static str],
}

pub fn all_specs() -> Vec<LanguageSpec> {
    vec![
        LanguageSpec {
            name: "typescript",
            extensions: &["ts", "tsx"],
            language_fn: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            function_kinds: &["function_declaration", "method_definition", "arrow_function"],
            class_kinds: &["class_declaration", "interface_declaration"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "case_clause",
                "catch_clause",
                "ternary_expression",
                "binary_expression",
            ],
            block_kinds: &["statement_block"],
            import_kinds: &["import_statement"],
            export_kinds: &["export_statement"],
        },
        LanguageSpec {
            name: "javascript",
            extensions: &["js", "jsx"],
            language_fn: || tree_sitter_javascript::LANGUAGE.into(),
            function_kinds: &["function_declaration", "method_definition", "arrow_function"],
            class_kinds: &["class_declaration"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "switch_case",
                "catch_clause",
                "ternary_expression",
            ],
            block_kinds: &["statement_block"],
            import_kinds: &["import_statement"],
            export_kinds: &["export_statement"],
        },
        LanguageSpec {
            name: "rust",
            extensions: &["rs"],
            language_fn: || tree_sitter_rust::LANGUAGE.into(),
            function_kinds: &["function_item"],
            class_kinds: &["struct_item", "enum_item", "trait_item"],
            decision_kinds: &[
                "if_expression",
                "if_let_expression",
                "for_expression",
                "while_expression",
                "while_let_expression",
                "match_arm",
            ],
            block_kinds: &["block"],
            import_kinds: &["use_declaration"],
            export_kinds: &[], // Rustはpub修飾子ベースのため専用ノード種別なし（本バージョンでは抽出しない）
        },
        LanguageSpec {
            name: "python",
            extensions: &["py"],
            language_fn: || tree_sitter_python::LANGUAGE.into(),
            function_kinds: &["function_definition"],
            class_kinds: &["class_definition"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "while_statement",
                "except_clause",
                "conditional_expression",
            ],
            block_kinds: &["block"],
            import_kinds: &["import_statement", "import_from_statement"],
            export_kinds: &[], // Pythonに明示的なexport構文はない
        },
        LanguageSpec {
            name: "java",
            extensions: &["java"],
            language_fn: || tree_sitter_java::LANGUAGE.into(),
            function_kinds: &["method_declaration", "constructor_declaration"],
            class_kinds: &["class_declaration", "interface_declaration"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "enhanced_for_statement",
                "while_statement",
                "do_statement",
                "switch_label",
                "catch_clause",
                "ternary_expression",
            ],
            block_kinds: &["block"],
            import_kinds: &["import_declaration"],
            export_kinds: &[],
        },
        LanguageSpec {
            name: "go",
            extensions: &["go"],
            language_fn: || tree_sitter_go::LANGUAGE.into(),
            function_kinds: &["function_declaration", "method_declaration"],
            class_kinds: &["type_declaration"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "expression_switch_statement",
                "type_switch_statement",
                "select_statement",
            ],
            block_kinds: &["block"],
            import_kinds: &["import_spec"],
            export_kinds: &[], // Goは大文字始まりの識別子がexport相当のため専用ノードなし
        },
        LanguageSpec {
            name: "csharp",
            extensions: &["cs"],
            language_fn: || tree_sitter_c_sharp::LANGUAGE.into(),
            function_kinds: &["method_declaration", "constructor_declaration"],
            class_kinds: &["class_declaration", "interface_declaration", "struct_declaration"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "foreach_statement",
                "while_statement",
                "do_statement",
                "switch_section",
                "catch_clause",
                "conditional_expression",
            ],
            block_kinds: &["block"],
            import_kinds: &["using_directive"],
            export_kinds: &[],
        },
        LanguageSpec {
            name: "c",
            extensions: &["c", "h"],
            language_fn: || tree_sitter_c::LANGUAGE.into(),
            function_kinds: &["function_definition"],
            class_kinds: &["struct_specifier"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "while_statement",
                "do_statement",
                "case_statement",
            ],
            block_kinds: &["compound_statement"],
            import_kinds: &["preproc_include"],
            export_kinds: &[],
        },
        LanguageSpec {
            name: "cpp",
            extensions: &["cpp", "hpp", "cc", "cxx"],
            language_fn: || tree_sitter_cpp::LANGUAGE.into(),
            function_kinds: &["function_definition"],
            class_kinds: &["class_specifier", "struct_specifier"],
            decision_kinds: &[
                "if_statement",
                "for_statement",
                "for_range_loop",
                "while_statement",
                "do_statement",
                "case_statement",
                "catch_clause",
            ],
            block_kinds: &["compound_statement"],
            import_kinds: &["preproc_include"],
            export_kinds: &[],
        },
    ]
}

pub fn find_spec_for_extension(ext: &str) -> Option<LanguageSpec> {
    let ext = ext.trim_start_matches('.').to_ascii_lowercase();
    all_specs().into_iter().find(|s| s.extensions.contains(&ext.as_str()))
}
