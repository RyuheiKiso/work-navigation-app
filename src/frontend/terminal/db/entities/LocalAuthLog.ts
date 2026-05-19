// TBL-032 auth_logs。認証成功/失敗を Append-only で記録、ログイン試行制限のローカル判定に使う
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('auth_logs')
@Index(['loginId'])
@Index(['createdAt'])
export class LocalAuthLog {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  loginId!: string;

  @Column('text')
  outcome!: string;

  @Column('text', { nullable: true })
  reason!: string | null;

  @Column('text', { nullable: true })
  ipAddress!: string | null;

  @Column('text')
  deviceId!: string;

  @Column('text')
  createdAt!: string;
}
