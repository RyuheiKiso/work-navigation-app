// TBL-048 rework_cost_records。修正コストの集計、原価管理向け
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('rework_cost_records')
@Index(['reworkId'])
export class LocalReworkCostRecord {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  reworkId!: string;

  @Column('real')
  laborMinutes!: number;

  @Column('real')
  materialCost!: number;

  @Column('real')
  otherCost!: number;

  @Column('text')
  costCurrency!: string;

  @Column('text')
  recordedAt!: string;

  @Column('text')
  recordedBy!: string;
}
