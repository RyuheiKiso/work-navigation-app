// MOD-SH-003: ClockService（時刻抽象）
// テスト時に時刻を固定できるよう、時刻取得を trait で抽象化する。
// この設計により、日付境界・タイムアウト・TTL 検証のテストが決定論的に行える。

use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// 時刻取得の抽象トレイト。
///
/// テスト時に FakeClock（固定時刻）に差し替えることで、
/// 時刻依存のテストを決定論的に実行できる。
///
/// # DI パターン
/// 本番コードでは `Arc<SystemClock>` を、テストでは `Arc<FakeClock>` を注入する。
pub trait Clock: Send + Sync {
    /// 現在時刻を UTC で返す。
    fn now(&self) -> DateTime<Utc>;

    /// 現在時刻をエポックからのミリ秒数（UTC）で返す。
    ///
    /// `now()` のデフォルト実装を提供する。オーバーライド不要。
    fn now_millis(&self) -> i64 {
        self.now().timestamp_millis()
    }
}

/// 本番用時刻サービス: システム時刻（UTC）を返す。
pub struct SystemClock;

impl Clock for SystemClock {
    /// システムの現在時刻（UTC）を返す。
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// テスト用時刻サービス: 固定時刻を返す。
///
/// `Mutex<DateTime<Utc>>` で保持するため、テスト中に時刻を更新できる。
/// `advance()` でインクリメント、`set()` で任意の時刻に変更可能。
pub struct FakeClock {
    /// 固定時刻（Mutex でスレッドセーフに保護する）
    fixed_time: std::sync::Mutex<DateTime<Utc>>,
}

impl FakeClock {
    /// 指定した時刻で固定時計を作成する。
    ///
    /// `Arc` で包んで返すことで、複数のコンポーネントが同一の FakeClock を共有できる。
    #[must_use]
    pub fn new(t: DateTime<Utc>) -> Arc<Self> {
        Arc::new(Self {
            fixed_time: std::sync::Mutex::new(t),
        })
    }

    /// 固定時刻を指定した Duration だけ進める。
    ///
    /// タイムアウト・TTL 期限切れのテストに使用する。
    ///
    /// # Panics
    ///
    /// 内部 `Mutex` がポイズンされた場合（別スレッドがロック保持中にパニックした場合）にパニックする。
    pub fn advance(&self, duration: Duration) {
        // Mutex のロックを取得して時刻を更新する
        let mut t = self
            .fixed_time
            .lock()
            .expect("FakeClock の Mutex がポイズンされています");
        *t += duration;
    }

    /// 固定時刻を指定した時刻に変更する。
    ///
    /// 特定の日付・時刻でテストしたい場合に使用する（例: 日付境界・月末処理）。
    ///
    /// # Panics
    ///
    /// 内部 `Mutex` がポイズンされた場合（別スレッドがロック保持中にパニックした場合）にパニックする。
    pub fn set(&self, t: DateTime<Utc>) {
        // Mutex のロックを取得して時刻を上書きする
        *self
            .fixed_time
            .lock()
            .expect("FakeClock の Mutex がポイズンされています") = t;
    }
}

impl Clock for FakeClock {
    /// 固定時刻を返す（システム時計ではなく設定された時刻を返す）。
    fn now(&self) -> DateTime<Utc> {
        *self
            .fixed_time
            .lock()
            .expect("FakeClock の Mutex がポイズンされています")
    }
}

/// アプリケーションへの DI 用エイリアス。
/// `Arc<dyn Clock>` を型エイリアスとして提供する。
pub type ClockRef = Arc<dyn Clock>;

/// 本番用 ClockRef（SystemClock）を生成する。
///
/// main.rs やアプリケーションの初期化コードで呼び出す。
#[must_use]
pub fn system_clock() -> ClockRef {
    Arc::new(SystemClock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_system_clock_returns_utc() {
        // SystemClock が UTC 時刻を返すことを確認する
        let clock = system_clock();
        let t = clock.now();
        // タイムゾーンが UTC であることを確認する（比較はできないが型で確認する）
        let _ = t.timestamp();
        assert!(t.timestamp() > 0);
    }

    #[test]
    fn test_fake_clock_fixed_time() {
        // FakeClock が固定時刻を返すことを確認する
        let fixed = Utc.with_ymd_and_hms(2026, 5, 19, 0, 0, 0).unwrap();
        let clock = FakeClock::new(fixed);

        // 時刻を変更せずに複数回呼び出しても同じ値を返すことを確認する
        let t1 = clock.now();
        let t2 = clock.now();
        assert_eq!(t1, fixed);
        assert_eq!(t2, fixed);
    }

    #[test]
    fn test_fake_clock_advance() {
        // FakeClock の advance() が時刻を正しく進めることを確認する
        let start = Utc.with_ymd_and_hms(2026, 5, 19, 12, 0, 0).unwrap();
        let clock = FakeClock::new(start);

        // 1 時間進める
        clock.advance(Duration::hours(1));
        let after_1h = clock.now();
        let expected_1h = Utc.with_ymd_and_hms(2026, 5, 19, 13, 0, 0).unwrap();
        assert_eq!(after_1h, expected_1h);

        // さらに 30 分進める
        clock.advance(Duration::minutes(30));
        let after_30m = clock.now();
        let expected_90m = Utc.with_ymd_and_hms(2026, 5, 19, 13, 30, 0).unwrap();
        assert_eq!(after_30m, expected_90m);
    }

    #[test]
    fn test_fake_clock_set() {
        // FakeClock の set() が時刻を任意に変更できることを確認する
        let start = Utc.with_ymd_and_hms(2026, 5, 19, 0, 0, 0).unwrap();
        let clock = FakeClock::new(start);

        // 任意の時刻に変更する
        let new_time = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        clock.set(new_time);
        assert_eq!(clock.now(), new_time);
    }

    #[test]
    fn test_fake_clock_now_millis() {
        // FakeClock の now_millis() がミリ秒タイムスタンプを正しく返すことを確認する
        let fixed = Utc.with_ymd_and_hms(2026, 5, 19, 0, 0, 0).unwrap();
        let clock = FakeClock::new(fixed);

        let millis = clock.now_millis();
        assert_eq!(millis, fixed.timestamp_millis());
    }

    #[test]
    fn test_clock_ref_polymorphism() {
        // ClockRef が SystemClock と FakeClock を透過的に使えることを確認する
        let system: ClockRef = system_clock();
        let fake: ClockRef = FakeClock::new(Utc::now());

        // どちらも Clock トレイトの now() を呼べることを確認する
        let _t1 = system.now();
        let _t2 = fake.now();
    }
}
