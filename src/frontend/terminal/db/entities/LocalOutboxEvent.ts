// TBL-003 outbox_events。送信キューは created_at 昇順で OutboxWorker が順次処理する
import { Entity, PrimaryColumn, Column, Index, Generated } from 'typeorm';
import { boolTransformer } from './_transformers';

@Entity('outbox_events')
@Index(['sent', 'createdAt'])
export class LocalOutboxEvent {
  @PrimaryColumn('integer')
  @Generated('rowid')
  id!: number;

  @Column('text')
  eventId!: string;

  @Column('text')
  idempotencyKey!: string;

  @Column('text')
  payload!: string;

  @Column('text')
  prevHash!: string;

  @Column('text')
  createdAt!: string;

  @Column('integer', { default: 0, transformer: boolTransformer })
  sent!: boolean;

  @Column('integer', { default: 0 })
  retryCount!: number;

  @Column('text', { nullable: true })
  lastError!: string | null;

  @Column('text')
  nextRetryAt!: string;
}
