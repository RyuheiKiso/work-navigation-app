// TBL-051 case_locks。例外的に UPDATE/DELETE 許可、heartbeat と lock_status を更新する
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('case_locks')
@Index(['terminalId'])
@Index(['lockStatus'])
export class LocalCaseLock {
  @PrimaryColumn('text')
  caseId!: string;

  @Column('text')
  terminalId!: string;

  @Column('text')
  userId!: string;

  @Column('text')
  acquiredAt!: string;

  @Column('text')
  heartbeatAt!: string;

  @Column('text')
  lockStatus!: string;
}
