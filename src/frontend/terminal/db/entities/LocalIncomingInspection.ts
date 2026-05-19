// TBL-038 incoming_inspections。受入検査ヘッダ、AQL 判定の入力
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('incoming_inspections')
@Index(['lotId'])
@Index(['supplierId'])
@Index(['qcStatus'])
export class LocalIncomingInspection {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  lotId!: string;

  @Column('text')
  supplierId!: string;

  @Column('text')
  materialId!: string;

  @Column('real')
  receivedQty!: number;

  @Column('text')
  samplingPlanId!: string;

  @Column('integer')
  sampleSizeN!: number;

  @Column('integer')
  acceptNumberAc!: number;

  @Column('integer')
  rejectNumberRe!: number;

  @Column('text', { default: 'NORMAL' })
  severityState!: string;

  @Column('text')
  qcStatus!: string;

  @Column('integer', { default: 0 })
  defectCount!: number;

  @Column('text', { nullable: true })
  inspectedAt!: string | null;

  @Column('text', { nullable: true })
  judgedAt!: string | null;

  @Column('text', { nullable: true })
  judgedBy!: string | null;

  @Column('text')
  createdAt!: string;
}
