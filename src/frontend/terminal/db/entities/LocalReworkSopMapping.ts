// TBL-046 rework_sop_mappings。不適合カテゴリと修正 SOP の対応表
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('rework_sop_mappings')
@Index(['ncCategory'])
export class LocalReworkSopMapping {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  ncCategory!: string;

  @Column('text')
  reworkSopId!: string;

  @Column('text')
  reworkSopVersionId!: string;

  @Column('integer', { default: 1 })
  isActive!: boolean;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
