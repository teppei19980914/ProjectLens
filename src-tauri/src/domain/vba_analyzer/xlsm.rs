//! xlsm/xlsb（OOXML zipコンテナ）内のVBAプロジェクト抽出（04_実装詳細.md §8.6）。
//! `xl/vbaProject.bin`（CFB/OLE2）から各モジュールのVBAソースをMS-OVBA解凍して取り出す。
//!
//! 実サンプル未入手のため、モジュール本文の開始位置（PerformanceCacheの終端）は
//! `dir`ストリームの厳密なレコード解析ではなく、ストリーム先頭からMS-OVBA圧縮コンテナの
//! シグネチャバイト(0x01)候補を探索して復号を試みる方式（フォールバック手法）を採用する。
//! 実サンプル入手後（04_実装詳細.md §10 残課題）に `dir` ストリームの厳密パースへの
//! 置き換えを検討する。

use super::{msovba, VbaContainerAnalyzer};
use crate::models::constants::macro_tools;
use crate::models::{AppError, AppResult, VbaComponent, VbaModule};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

pub struct XlsmAnalyzer;

const VBA_PROJECT_ENTRY: &str = "xl/vbaProject.bin";
const NON_MODULE_STREAMS: &[&str] = &["dir", "PROJECT", "PROJECTwm", "_VBA_PROJECT"];

impl VbaContainerAnalyzer for XlsmAnalyzer {
    fn tool_name(&self) -> &'static str {
        macro_tools::EXCEL_VBA
    }

    fn detect(&self, _relative_path: &Path, absolute_path: &Path) -> bool {
        let Ok(file) = std::fs::File::open(absolute_path) else {
            return false;
        };
        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return false;
        };
        let found = archive.by_name(VBA_PROJECT_ENTRY).is_ok();
        found
    }

    fn extract(&self, relative_path: &Path, absolute_path: &Path) -> AppResult<(VbaComponent, String)> {
        let file = std::fs::File::open(absolute_path)
            .map_err(|e| AppError::static_analysis(format!("Excelファイルを開けませんでした: {e}")))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::static_analysis(format!("xlsm/xlsbのzip展開に失敗しました: {e}")))?;
        let vba_bytes = {
            let mut entry = archive
                .by_name(VBA_PROJECT_ENTRY)
                .map_err(|e| AppError::static_analysis(format!("vbaProject.binが見つかりません: {e}")))?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| AppError::static_analysis(format!("vbaProject.binの読み込みに失敗しました: {e}")))?;
            buf
        };

        let mut cfb = cfb::CompoundFile::open(Cursor::new(vba_bytes))
            .map_err(|e| AppError::static_analysis(format!("vbaProject.binのOLE解析に失敗しました: {e}")))?;

        let module_kinds = read_project_stream_kinds(&mut cfb);
        let external_references = read_dir_stream_reference_names(&mut cfb);

        let stream_paths: Vec<std::path::PathBuf> = cfb
            .walk()
            .filter(|entry| entry.is_stream() && entry.path().starts_with("/VBA"))
            .map(|entry| entry.path().to_path_buf())
            .collect();

        let mut modules = Vec::new();
        let mut combined_source = String::new();

        for stream_path in stream_paths {
            let name = match stream_path.file_name().and_then(|n| n.to_str()) {
                Some(n) if !NON_MODULE_STREAMS.contains(&n) => n.to_string(),
                _ => continue,
            };

            let Ok(mut stream) = cfb.open_stream(&stream_path) else {
                continue;
            };
            let mut raw = Vec::new();
            if stream.read_to_end(&mut raw).is_err() {
                continue;
            }

            let Some(source) = decompress_module_stream(&raw) else {
                continue; // 復号できないストリームはスキップし継続（NFR-03）
            };

            let loc = source.lines().count() as u32;
            let kind = module_kinds.get(&name).cloned().unwrap_or_else(|| "standard".into());

            combined_source.push_str(&format!("'--- Module: {name} ({kind}) ---\n{source}\n\n"));
            modules.push(VbaModule { name, kind, loc });
        }

        let workbook_name = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok((
            VbaComponent {
                component_path: relative_path.to_string_lossy().replace('\\', "/"),
                workbook_name,
                modules,
                external_references,
            },
            combined_source,
        ))
    }
}

/// モジュール本文の開始位置（PerformanceCache終端）をdirストリームの厳密解析ではなく、
/// ストリーム内でMS-OVBA圧縮コンテナとして復号可能な先頭候補を探索して決定する。
fn decompress_module_stream(raw: &[u8]) -> Option<String> {
    let search_limit = raw.len().min(8192);
    for offset in 0..search_limit {
        if raw[offset] != 0x01 {
            continue;
        }
        if let Ok(decompressed) = msovba::decompress(&raw[offset..]) {
            let text = String::from_utf8_lossy(&decompressed).to_string();
            if text.contains("Attribute VB_Name") || text.contains("Attribute VB_") {
                return Some(text);
            }
        }
    }
    None
}

/// `/PROJECT`ストリーム（プレーンテキスト、MS-OVBA圧縮なし）からモジュール種別を読み取る。
/// 例: "Module=Module1" / "Class=Class1" / "BaseClass=UserForm1" / "Document=ThisWorkbook/&H00000000"
fn read_project_stream_kinds<F: Read + Seek>(cfb: &mut cfb::CompoundFile<F>) -> HashMap<String, String> {
    let mut kinds = HashMap::new();
    let Ok(mut stream) = cfb.open_stream("/PROJECT") else {
        return kinds;
    };
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
        return kinds;
    }
    let text = String::from_utf8_lossy(&buf);
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let kind = match key {
            "Module" => "standard",
            "Class" => "class",
            "BaseClass" => "form",
            "Document" => "document",
            _ => continue,
        };
        let name = value.split('/').next().unwrap_or(value).trim();
        if !name.is_empty() {
            kinds.insert(name.to_string(), kind.to_string());
        }
    }
    kinds
}

/// `dir`ストリーム（先頭からMS-OVBA圧縮）を復号し、REFERENCENAMEレコード(id=0x0016)らしき
/// バイト列からベストエフォートで参照名を抽出する（04_実装詳細.md §10 残課題:
/// 厳密なレコード文法での解析は実サンプル入手後に検討）。
fn read_dir_stream_reference_names<F: Read + Seek>(cfb: &mut cfb::CompoundFile<F>) -> Vec<String> {
    let Ok(mut stream) = cfb.open_stream("/VBA/dir") else {
        return Vec::new();
    };
    let mut raw = Vec::new();
    if stream.read_to_end(&mut raw).is_err() {
        return Vec::new();
    }
    let Ok(decompressed) = msovba::decompress(&raw) else {
        return Vec::new();
    };

    let mut names = Vec::new();
    let mut i = 0usize;
    while i + 6 <= decompressed.len() {
        let id = u16::from_le_bytes([decompressed[i], decompressed[i + 1]]);
        let size = u32::from_le_bytes([
            decompressed[i + 2],
            decompressed[i + 3],
            decompressed[i + 4],
            decompressed[i + 5],
        ]) as usize;
        if id == 0x0016 && size > 0 && size <= 260 && i + 6 + size <= decompressed.len() {
            let bytes = &decompressed[i + 6..i + 6 + size];
            if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                if let Ok(name) = std::str::from_utf8(bytes) {
                    names.push(name.to_string());
                }
            }
        }
        i += 1;
    }
    names
}
