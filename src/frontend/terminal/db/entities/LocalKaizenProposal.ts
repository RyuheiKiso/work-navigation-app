// TBL-015 kaizen_proposals。改善提案、現場作業員が起票し承認フローを通る
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('kaizen_proposals')
@Index(['proposerId'])
@Index(['status'])
export class LocalKaizenProposal {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  proposerId!: string;

  @Column('text', { nullable: true })
  processId!: string | null;

  @Column('text')
  category!: string;

  @Column('text')
  title!: string;

  @Column('text')
  currentSituation!: string;

  @Column('text')
  proposalDetail!: string;

  @Column('text', { nullable: true })
  expectedBenefit!: string | null;

  @Column('text', { nullable: true })
  relatedSopId!: string | null;

  @Column('text')
  evidenceIds!: string;

  @Column('text')
  status!: string;

  @Column('text')
  createdAt!: string;
}
