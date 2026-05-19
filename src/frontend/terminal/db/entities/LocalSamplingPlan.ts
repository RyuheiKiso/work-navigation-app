// TBL-039 sampling_plans。AQL サンプリング計画のスナップショット
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('sampling_plans')
export class LocalSamplingPlan {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  planCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('real')
  aqlValue!: number;

  @Column('text')
  inspectionLevel!: string;

  @Column('text')
  planSnapshot!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
