// TBL-035 idempotency_keys。送信済 UUID と応答ハッシュをキャッシュ、リトライ時の重複防止
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('idempotency_keys')
@Index(['expiresAt'])
export class LocalIdempotencyKey {
  @PrimaryColumn('text')
  key!: string;

  @Column('text', { nullable: true })
  responseHash!: string | null;

  @Column('text')
  createdAt!: string;

  @Column('text')
  expiresAt!: string;
}
