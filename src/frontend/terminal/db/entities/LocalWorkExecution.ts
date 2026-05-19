// TBL-005 work_executions。作業実行は status と current_step_id が更新される
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('work_executions')
@Index(['status'])
@Index(['operatorId'])
export class LocalWorkExecution {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  workOrderId!: string;

  @Column('text')
  operatorId!: string;

  @Column('text')
  deviceId!: string;

  @Column('text')
  status!: string;

  @Column('text', { nullable: true })
  currentStepId!: string | null;

  @Column('integer', { default: 0 })
  completedStepCount!: number;

  @Column('integer', { default: 0 })
  totalStepCount!: number;

  @Column('text')
  sopVersionSnapshot!: string;

  @Column('text')
  startedAt!: string;

  @Column('text')
  lastEventAt!: string;

  @Column('text', { nullable: true })
  completedAt!: string | null;

  @Column('text')
  createdAt!: string;
}
