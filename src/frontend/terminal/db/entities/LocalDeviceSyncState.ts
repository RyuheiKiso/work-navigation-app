// TBL-034 device_sync_states。マスタ差分同期のカーソル管理
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('device_sync_states')
export class LocalDeviceSyncState {
  @PrimaryColumn('text')
  entityType!: string;

  @Column('text', { nullable: true })
  cursor!: string | null;

  @Column('text', { nullable: true })
  lastSyncedAt!: string | null;

  @Column('integer', { default: 0 })
  pendingCount!: number;
}
