// TBL-013 nonconformities。不適合品記録、4M 原因と証拠 ID リストを保持
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('nonconformities')
@Index(['workExecutionId'])
@Index(['lotId'])
export class LocalNonconformity {
  @PrimaryColumn('text')
  id!: string;

  @Column('text', { nullable: true })
  alertId!: string | null;

  @Column('text', { nullable: true })
  workExecutionId!: string | null;

  @Column('text', { nullable: true })
  lotId!: string | null;

  @Column('text')
  ncType!: string;

  @Column('text')
  description!: string;

  @Column('text')
  discoveredBy!: string;

  @Column('text', { nullable: true })
  discoveryStepId!: string | null;

  @Column('text')
  evidenceIds!: string;

  @Column('text')
  createdAt!: string;
}
