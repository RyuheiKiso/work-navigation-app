// 端末専用 app_settings。PK は常に 'singleton'、INSERT OR REPLACE で 1 レコードを維持
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('app_settings')
export class LocalAppSettings {
  @PrimaryColumn('text')
  settingsId!: string;

  @Column('text', { default: 'ja' })
  locale!: string;

  @Column('integer', { default: 0 })
  darkMode!: boolean;

  @Column('text')
  deviceId!: string;

  @Column('text', { nullable: true })
  jwtCache!: string | null;

  @Column('text', { nullable: true })
  jwtExpiresAt!: string | null;

  @Column('text', { nullable: true })
  lastMasterSyncAt!: string | null;

  @Column('text', { nullable: true })
  currentUserId!: string | null;

  @Column('integer', { default: 30000 })
  outboxIntervalMs!: number;

  @Column('integer', { default: 300000 })
  emergencyThresholdMs!: number;

  @Column('integer', { default: 60 })
  masterSyncIntervalMinutes!: number;
}
