// TBL-014 capa。是正処置・予防処置の進捗管理
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('capa')
@Index(['status'])
export class LocalCapa {
  @PrimaryColumn('text')
  id!: string;

  @Column('text', { nullable: true })
  nonconformityId!: string | null;

  @Column('text')
  title!: string;

  @Column('text')
  status!: string;

  @Column('text')
  rootCauseAnalysis!: string;

  @Column('text')
  correctiveAction!: string;

  @Column('text', { nullable: true })
  preventiveAction!: string | null;

  @Column('text')
  assignedTo!: string;

  @Column('text')
  dueDate!: string;

  @Column('text')
  createdBy!: string;

  @Column('text')
  createdAt!: string;

  @Column('text', { nullable: true })
  progressNote!: string | null;

  @Column('text', { nullable: true })
  closedAt!: string | null;

  @Column('text', { nullable: true })
  closedBy!: string | null;
}
