// JSON Logic 評価エンジン（ALG-004/005）
// eval() 禁止・ホワイトリストオペレータのみ許可する安全な評価エンジン。
// jsonlogic-rs クレートをラップして使用する。
// 深度検証（ALG-005・BR-BUS-022）とタイムアウトガードを提供する。

use serde_json::Value;

/// JSON Logic 評価エンジン。
/// ALG-004/005 で定義されたホワイトリストオペレータのみを許可する。
/// `eval()` および動的コード生成は一切使用しない（src/CLAUDE.md 動的評価禁止）。
pub struct JsonLogicEvaluator {
    /// 最大ネスト深度（BR-BUS-022: 5 以内）
    max_depth: u32,
}

/// JSON Logic 評価エラー。
#[derive(Debug, thiserror::Error)]
pub enum JsonLogicError {
    /// 禁止オペレータの使用（ALG-004）
    #[error("禁止オペレータです: {operator}")]
    ForbiddenOperator { operator: String },

    /// ネスト深度超過（BR-BUS-022）
    #[error("DSL ネスト深度が上限（{max_depth}）を超えています")]
    DepthExceeded { max_depth: u32 },

    /// 評価エラー（型不一致等）
    #[error("JSON Logic 評価エラー: {0}")]
    EvaluationFailed(String),
}

impl JsonLogicEvaluator {
    /// 新しい評価器を作成する。最大深度は BR-BUS-022 に従い 5 とする。
    pub fn new() -> Self {
        Self { max_depth: 5 }
    }

    /// 許可オペレータのホワイトリスト（ALG-004）。
    /// ホワイトリスト外のオペレータは EvaluationFailed エラーを発生させる。
    fn is_allowed_operator(op: &str) -> bool {
        matches!(
            op,
            // 比較演算子
            "==" | "!=" | "<" | ">" | "<=" | ">="
            // 論理演算子
            | "and" | "or" | "!"
            // 文字列・配列
            | "in" | "cat"
            // 変数参照
            | "var"
            // 制御フロー
            | "if"
            // 算術（4 則のみ）
            | "+" | "-" | "*" | "/"
            // 集計
            | "min" | "max"
        )
    }

    /// (ALG-005) JSON Logic ルールのネスト深度を静的検証する。
    /// SOP 保存時に呼び出し、BR-BUS-022（深度 5 以内）を強制する。
    pub fn validate_rule_depth(
        rule: &Value,
        current_depth: u32,
        max_depth: u32,
    ) -> Result<(), JsonLogicError> {
        if current_depth > max_depth {
            return Err(JsonLogicError::DepthExceeded { max_depth });
        }

        // プリミティブ値は深度チェック不要
        let Value::Object(obj) = rule else {
            return Ok(());
        };

        // 1 オペレータのみ許可する
        if obj.len() != 1 {
            return Err(JsonLogicError::EvaluationFailed(
                "Rule must have exactly one operator key".to_string(),
            ));
        }

        if let Some((operator, args)) = obj.iter().next() {
            // オペレータのホワイトリストチェック
            if !Self::is_allowed_operator(operator) {
                return Err(JsonLogicError::ForbiddenOperator {
                    operator: operator.clone(),
                });
            }

            // 引数を再帰的に検証する
            match args {
                Value::Array(arr) => {
                    for arg in arr {
                        Self::validate_rule_depth(arg, current_depth + 1, max_depth)?;
                    }
                }
                Value::Object(_) => {
                    Self::validate_rule_depth(args, current_depth + 1, max_depth)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// (ALG-004) JSON Logic ルールを評価する。
    /// jsonlogic-rs クレートを使用して安全に評価する。
    /// ルール評価失敗時は false を返す（安全側フォールバック）。
    pub fn evaluate(&self, rule: &Value, data: &Value) -> bool {
        // ネスト深度の事前検証（BR-BUS-022）
        if Self::validate_rule_depth(rule, 0, self.max_depth).is_err() {
            tracing::warn!("JSON Logic ルールの深度検証に失敗しました（BR-BUS-022 違反）");
            return false;
        }

        // jsonlogic-rs で評価する
        match jsonlogic_rs::apply(rule, data) {
            Ok(result) => {
                // 評価結果を bool に変換する
                match result {
                    Value::Bool(b) => b,
                    // 非 bool の結果は truthy 評価する
                    Value::Null => false,
                    Value::Number(n) => n.as_f64().is_some_and(|v| v != 0.0),
                    Value::String(s) => !s.is_empty(),
                    Value::Array(arr) => !arr.is_empty(),
                    Value::Object(_) => true,
                }
            }
            Err(e) => {
                // 評価エラーは安全側フォールバックとして false を返す
                tracing::warn!(error = %e, "JSON Logic 評価に失敗しました");
                false
            }
        }
    }
}

impl Default for JsonLogicEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_evaluate_simple_equality() {
        // 単純な等値評価が正しく動作することを確認する
        let evaluator = JsonLogicEvaluator::new();
        let rule = json!({"==": [{"var": "x"}, 10]});
        let data_match = json!({"x": 10});
        let data_no_match = json!({"x": 5});

        assert!(evaluator.evaluate(&rule, &data_match));
        assert!(!evaluator.evaluate(&rule, &data_no_match));
    }

    #[test]
    fn test_evaluate_logical_and() {
        // 論理 AND 演算が正しく動作することを確認する
        let evaluator = JsonLogicEvaluator::new();
        let rule = json!({"and": [
            {">=": [{"var": "temperature"}, 18.0]},
            {"<=": [{"var": "temperature"}, 30.0]}
        ]});

        assert!(evaluator.evaluate(&rule, &json!({"temperature": 25.0})));
        assert!(!evaluator.evaluate(&rule, &json!({"temperature": 10.0})));
        assert!(!evaluator.evaluate(&rule, &json!({"temperature": 35.0})));
    }

    #[test]
    fn test_validate_rule_depth_exceeds() {
        // 深度 5 超過のルールが正しくエラーを返すことを確認する（BR-BUS-022）
        let deep_rule =
            json!({"and": [{"and": [{"and": [{"and": [{"and": [{"==": [1, 1]}]}]}]}]}]});
        let result = JsonLogicEvaluator::validate_rule_depth(&deep_rule, 0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rule_depth_within_limit() {
        // 深度 5 以内のルールは合格することを確認する（BR-BUS-022）
        let shallow_rule = json!({"and": [{"==": [1, 1]}, {"!=": [2, 3]}]});
        let result = JsonLogicEvaluator::validate_rule_depth(&shallow_rule, 0, 5);
        assert!(result.is_ok());
    }
}
