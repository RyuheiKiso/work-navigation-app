// TBL-040 incoming_inspection_measurements。IQC のサンプル毎測定値
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('incoming_inspection_measurements')
@Index(['inspectionId'])
export class LocalIncomingInspectionMeasurement {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  inspectionId!: string;

  @Column('integer')
  sampleNo!: number;

  @Column('real', { nullable: true })
  measuredValue!: number | null;

  @Column('integer', { default: 0 })
  defectFlag!: boolean;

  @Column('text', { nullable: true })
  evidenceFileId!: string | null;

  @Column('text')
  recordedAt!: string;

  @Column('text')
  recordedBy!: string;
}
