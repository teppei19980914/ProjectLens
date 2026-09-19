//! MS-OVBA仕様 §2.4.1「Compressed Container」のRLE系解凍アルゴリズム。
//! vbaProject.bin（CFB）内の各 `VBA/<ModuleName>` ストリームはこの形式で格納されており、
//! xlsm/docm/pptm等どのコンテナ実装からも共用する（CLAUDE.md 原則2.2.1、単一箇所に集約）。

use crate::models::{AppError, AppResult};

const CHUNK_DECOMPRESSED_SIZE: usize = 4096;

/// 圧縮コンテナ（先頭1バイトのSignatureByte(0x01) + チャンク列）をデコードする。
pub fn decompress(compressed: &[u8]) -> AppResult<Vec<u8>> {
    if compressed.is_empty() || compressed[0] != 0x01 {
        return Err(AppError::static_analysis(
            "VBA圧縮ストリームのシグネチャバイトが不正です",
        ));
    }

    let mut out = Vec::new();
    let mut pos = 1usize;

    while pos < compressed.len() {
        if pos + 2 > compressed.len() {
            break; // 末尾の不完全なヘッダは無視して打ち切る
        }
        let header = u16::from_le_bytes([compressed[pos], compressed[pos + 1]]);
        let chunk_size = (header & 0x0FFF) as usize + 3; // ヘッダ2バイト込みのサイズ
        let is_compressed = (header & 0x8000) != 0;
        let chunk_end = (pos + chunk_size).min(compressed.len());
        pos += 2;

        if !is_compressed {
            // 非圧縮チャンクは常に4096バイトそのままコピーする
            let end = (pos + CHUNK_DECOMPRESSED_SIZE).min(compressed.len());
            out.extend_from_slice(&compressed[pos..end]);
        } else {
            let chunk_start_in_out = out.len();
            while pos < chunk_end {
                let flag_byte = compressed[pos];
                pos += 1;
                for bit in 0..8 {
                    if pos >= chunk_end {
                        break;
                    }
                    let is_copy_token = (flag_byte >> bit) & 1 == 1;
                    if !is_copy_token {
                        out.push(compressed[pos]);
                        pos += 1;
                    } else {
                        if pos + 2 > chunk_end {
                            break;
                        }
                        let token = u16::from_le_bytes([compressed[pos], compressed[pos + 1]]);
                        pos += 2;
                        let difference = (out.len() - chunk_start_in_out).max(1) as u32;
                        let bit_count = bit_count_for(difference);
                        let length_mask: u16 = 0xFFFF >> bit_count;
                        let offset_mask: u16 = !length_mask;
                        let length = (token & length_mask) as usize + 3;
                        let offset = ((token & offset_mask) >> (16 - bit_count)) as usize + 1;

                        if offset > out.len() {
                            return Err(AppError::static_analysis(
                                "VBA圧縮ストリームのコピートークンが不正な参照位置を指しています",
                            ));
                        }
                        let copy_source = out.len() - offset;
                        for i in 0..length {
                            let byte = out[copy_source + i];
                            out.push(byte);
                        }
                    }
                }
            }
        }

        pos = chunk_end;
    }

    Ok(out)
}

/// ceil(log2(difference)) を4以上12以下に丸めたもの（MS-OVBA §2.4.1.3.19 CopyTokenHelp）。
fn bit_count_for(difference: u32) -> u32 {
    let d = difference.max(1);
    let mut n = 0u32;
    while (1u32 << n) < d {
        n += 1;
    }
    n.clamp(4, 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompresses_uncompressed_chunk_verbatim() {
        // 非圧縮チャンク: フラグビット(0x8000)未設定、4096バイトそのままコピー
        let mut compressed = vec![0x01u8]; // signature byte
        let payload: Vec<u8> = (0..CHUNK_DECOMPRESSED_SIZE).map(|i| (i % 251) as u8).collect();
        let chunk_size = (CHUNK_DECOMPRESSED_SIZE - 1) as u16 & 0x0FFF; // header格納値(サイズ-3の下位12bit)
        let header = chunk_size; // 圧縮フラグなし
        compressed.extend_from_slice(&header.to_le_bytes());
        compressed.extend_from_slice(&payload);

        let result = decompress(&compressed).unwrap();
        assert_eq!(result, payload);
    }

    #[test]
    fn decompresses_literal_only_compressed_chunk() {
        // 圧縮チャンクだが全トークンがリテラル（フラグバイト=0x00）の単純ケース
        let data = b"Attribute VB_Name = \"Module1\"";
        let mut chunk_body = Vec::new();
        for group in data.chunks(8) {
            chunk_body.push(0x00u8); // 8ビット全てリテラル
            chunk_body.extend_from_slice(group);
        }
        let chunk_size_field = (chunk_body.len() + 2 - 3) as u16 & 0x0FFF;
        let header = chunk_size_field | 0x8000; // 圧縮フラグON
        let mut compressed = vec![0x01u8];
        compressed.extend_from_slice(&header.to_le_bytes());
        compressed.extend_from_slice(&chunk_body);

        let result = decompress(&compressed).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn decompresses_copy_token_back_reference() {
        // 手動構築: リテラル 'A','B' の後、length=4/offset=2 のコピートークンで "ABAB" を
        // 自己参照コピーし、"ABABAB" を復元できることを確認する（MS-OVBA §2.4.1.3.19）。
        let flag_byte = 0b0000_0100u8; // bit0=literal(A), bit1=literal(B), bit2=copy token
        let token: u16 = 0x1001; // offset(top4bit)=(offset-1)=1 -> offset=2, length(low12bit)=(length-3)=1 -> length=4
        let mut chunk_body = vec![flag_byte, b'A', b'B'];
        chunk_body.extend_from_slice(&token.to_le_bytes());

        let chunk_size_field = (chunk_body.len() + 2 - 3) as u16 & 0x0FFF;
        let header = chunk_size_field | 0x8000;
        let mut compressed = vec![0x01u8];
        compressed.extend_from_slice(&header.to_le_bytes());
        compressed.extend_from_slice(&chunk_body);

        let result = decompress(&compressed).unwrap();
        assert_eq!(result, b"ABABAB");
    }

    #[test]
    fn rejects_invalid_signature_byte() {
        let compressed = vec![0x02u8, 0x00, 0x00];
        assert!(decompress(&compressed).is_err());
    }
}
