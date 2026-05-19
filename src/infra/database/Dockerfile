FROM postgres:16-alpine

# postgres:16-alpine は musl libc を使用するため ja_JP.UTF-8 glibc ロケールは利用不可。
# データベース自体の日本語対応は ICU ロケールプロバイダ（00_create_databases.sh）で実現する。
# LANG は Alpine / musl が確実にサポートする en_US.utf8 を使用する。
ENV LANG=en_US.utf8
ENV TZ=Asia/Tokyo

# 初期化スクリプトをコピーする
COPY docker-entrypoint-initdb.d/ /docker-entrypoint-initdb.d/

# カスタム設定ファイルをコピーする（コンテナ起動後に postgresql.conf で参照）
COPY config/postgresql.conf /etc/postgresql/postgresql.conf
COPY config/pg_hba.conf /etc/postgresql/pg_hba.conf
COPY config/pg_ident.conf /etc/postgresql/pg_ident.conf

# ヘルスチェック: pg_isready で DB 起動確認
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
    CMD pg_isready -U ${POSTGRES_USER:-postgres} -d ${POSTGRES_DB:-postgres} || exit 1

EXPOSE 5432

# カスタム設定ファイルを起動時に参照させる
# postgres:alpine は /etc/postgresql/ を自動参照しないため config_file を明示指定する
CMD ["postgres", \
     "-c", "config_file=/etc/postgresql/postgresql.conf", \
     "-c", "hba_file=/etc/postgresql/pg_hba.conf", \
     "-c", "ident_file=/etc/postgresql/pg_ident.conf"]
