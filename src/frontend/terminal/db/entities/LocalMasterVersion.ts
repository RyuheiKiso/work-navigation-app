// TBL-004 master_versions。マスタの版管理情報
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('master_versions')
@Index(['sopId'])
@Index(['entityType', 'entityId'])
export class LocalMasterVersion {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  sopId!: string;

  @Column('text')
  entityType!: string;

  @Column('text')
  entityId!: string;

  @Column('text')
  version!: string;

  @Column('text')
  status!: string;

  @Column('text')
  changeSummary!: string;

  @Column('integer', { default: 0 })
  stepCount!: number;

  @Column('text')
  createdAt!: string;

  @Column('text')
  createdBy!: string;

  @Column('text', { nullable: true })
  submittedAt!: string | null;

  @Column('text', { nullable: true })
  submittedBy!: string | null;

  @Column('text', { nullable: true })
  approvedBy!: string | null;

  @Column('text', { nullable: true })
  approvedAt!: string | null;

  @Column('text', { nullable: true })
  publishedAt!: string | null;

  @Column('text', { nullable: true })
  publishedBy!: string | null;

  @Column('text', { nullable: true })
  deprecatedAt!: string | null;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
