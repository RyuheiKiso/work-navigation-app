// ALG-006: canonical JSON 正規化モジュール
// 決定論的なハッシュ計算のため、JSON キーをアルファベット順にソートして
// コンパクト形式でシリアライズする。同一入力は必ず同一出力を返す。

use serde_json::Value;
use std::collections::BTreeMap;

/// ALG-006: serde_json::Value を canonical JSON 文字列に変換する。
///
/// # 正規化規則
/// - マップキーは BTreeMap により常にアルファベット昇順にソートする
/// - 浮動小数点値が整数として表現できる場合は整数として出力する（例: 1.0 → 1）
/// - null は "null"、true は "true"、false は "false" として出力する
/// - ホワイトスペースなしのコンパクト JSON で出力する
/// - 末尾の改行は付与しない
///
/// # 決定論性の保証
/// 同一の入力 Value に対して、呼び出し回数や実行環境によらず必ず同一の文字列を返す。
pub fn canonical_json(value: &Value) -> String {
    serialize_value(value)
}

/// Value を再帰的にシリアライズする内部関数。
fn serialize_value(value: &Value) -> String {
    match value {
        // null 値はそのまま出力する
        Value::Null => "null".to_string(),

        // 真偽値はそのまま出力する
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }

        // 数値: 整数として表現できる浮動小数点は整数で出力する
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                // 整数として表現できるか確認する（例: 1.0 → "1"）
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{}", f as i64)
                } else {
                    // 有限でない浮動小数点はそのまま出力する
                    format!("{}", f)
                }
            } else {
                // フォールバック: serde_json の Number 文字列表現を使用する
                n.to_string()
            }
        }

        // 文字列: JSON エスケープを適用して出力する
        Value::String(s) => {
            // serde_json に任せて適切にエスケープする
            serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
        }

        // 配列: 各要素を再帰的にシリアライズする
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(serialize_value).collect();
            format!("[{}]", elements.join(","))
        }

        // オブジェクト: BTreeMap でキーをアルファベット昇順にソートして出力する
        Value::Object(map) => {
            // BTreeMap に変換してキーを確実にソートする
            let sorted: BTreeMap<&str, &Value> = map.iter().map(|(k, v)| (k.as_str(), v)).collect();
            let pairs: Vec<String> = sorted
                .iter()
                .map(|(k, v)| {
                    let key_json =
                        serde_json::to_string(k).unwrap_or_else(|_| format!("\"{}\"", k));
                    format!("{}:{}", key_json, serialize_value(v))
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_json_object_key_sort() {
        // オブジェクトキーがアルファベット順にソートされることを確認する
        let value = json!({ "z": 1, "a": 2, "m": 3 });
        let result = canonical_json(&value);
        assert_eq!(result, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_canonical_json_float_to_int() {
        // 整数として表現できる浮動小数点は整数で出力されることを確認する
        let value = json!({ "val": 1.0 });
        let result = canonical_json(&value);
        assert_eq!(result, r#"{"val":1}"#);
    }

    #[test]
    fn test_canonical_json_null() {
        // null が正しく出力されることを確認する
        assert_eq!(canonical_json(&Value::Null), "null");
    }

    #[test]
    fn test_canonical_json_bool() {
        // 真偽値が正しく出力されることを確認する
        assert_eq!(canonical_json(&json!(true)), "true");
        assert_eq!(canonical_json(&json!(false)), "false");
    }

    #[test]
    fn test_canonical_json_nested() {
        // ネストしたオブジェクトもキーがソートされることを確認する
        let value = json!({ "b": { "z": 1, "a": 2 }, "a": "hello" });
        let result = canonical_json(&value);
        assert_eq!(result, r#"{"a":"hello","b":{"a":2,"z":1}}"#);
    }

    #[test]
    fn test_canonical_json_determinism() {
        // 同一入力に対して複数回呼び出しても同一の結果を返すことを確認する
        let value = json!({ "case_id": "abc", "event_id": "def", "activity": "step_completed" });
        let result1 = canonical_json(&value);
        let result2 = canonical_json(&value);
        assert_eq!(result1, result2);
        // キーがアルファベット順に並んでいることも確認する
        assert_eq!(
            result1,
            r#"{"activity":"step_completed","case_id":"abc","event_id":"def"}"#
        );
    }
}
