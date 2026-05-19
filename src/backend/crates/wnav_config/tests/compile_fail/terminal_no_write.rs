// TerminalDatabaseConfig に write フィールドが存在しないことを確認するコンパイルエラーテスト
// このファイルはコンパイルエラーになることを期待する（trybuild::compile_fail!）

fn main() {
    // TerminalApiConfig::database.write にアクセスしようとするとコンパイルエラーになる
    // DB ロール物理保証: terminal_api は write ロールを型として保有できない
    let _: wnav_config::schema::TerminalDatabaseConfig;
    // 下記のアクセスはコンパイルエラーになることを期待する:
    // _x.write のようなアクセスは TerminalDatabaseConfig に write フィールドがないため失敗する
    // 型推論でコンパイルエラーを引き起こすためにダミーの式を書く
    let dummy: wnav_config::TerminalApiConfig = todo!();
    let _: () = dummy.database.write; //~ ERROR no field `write` on type `TerminalDatabaseConfig`
}
