// TBL-052/053 work_assignments。外部システム経由の作業指示割当受信
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('work_assignments')
@Index(['targetTerminalId'])
@Index(['status'])
@Index(['externalOrderId', 'externalSystem'], { unique: true })
export class LocalWorkAssignment {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  externalOrderId!: string;

  @Column('text')
  externalSystem!: string;

  @Column('text')
  sopId!: string;

  @Column('text')
  sopName!: string;

  @Column('text')
  targetTerminalId!: string;

  @Column('text', { nullable: true })
  lotId!: string | null;

  @Column('text', { nullable: true })
  lotNumber!: string | null;

  @Column('text', { nullable: true })
  suggestedWorkerId!: string | null;

  @Column('text', { nullable: true })
  suggestedEquipmentId!: string | null;

  @Column('text', { nullable: true })
  dueAt!: string | null;

  @Column('integer', { default: 0 })
  priority!: number;

  @Column('text')
  status!: string;

  @Column('text')
  receivedAt!: string;

  @Column('text', { nullable: true })
  acknowledgedAt!: string | null;

  @Column('text', { nullable: true })
  cancelledAt!: string | null;
}
