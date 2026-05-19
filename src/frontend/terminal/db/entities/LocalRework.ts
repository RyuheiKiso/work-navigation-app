// TBL-043 reworks。修正作業のメタ、新 case_id と親 case_id を紐付ける
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('reworks')
@Index(['parentCaseId'])
@Index(['nonconformityId'])
@Index(['status'])
export class LocalRework {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  parentCaseId!: string;

  @Column('text')
  reworkCaseId!: string;

  @Column('text')
  nonconformityId!: string;

  @Column('text')
  sopId!: string;

  @Column('text')
  reworkSopVersionId!: string;

  @Column('text', { nullable: true })
  assignedTo!: string | null;

  @Column('text')
  status!: string;

  @Column('integer', { default: 1 })
  reworkCount!: number;

  @Column('text', { nullable: true })
  deadline!: string | null;

  @Column('text')
  createdAt!: string;
}
