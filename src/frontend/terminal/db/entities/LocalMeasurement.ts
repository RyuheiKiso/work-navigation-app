// TBL-010 measurements。数値測定値と単位を保持し USL/LSL とのスペック判定に使用する
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('measurements')
@Index(['workExecutionId'])
@Index(['stepId'])
export class LocalMeasurement {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  workExecutionId!: string;

  @Column('text')
  stepId!: string;

  @Column('real')
  value!: number;

  @Column('text')
  unit!: string;

  @Column('real', { nullable: true })
  usl!: number | null;

  @Column('real', { nullable: true })
  lsl!: number | null;

  @Column('integer', { default: 0 })
  inSpec!: boolean;

  @Column('text')
  recordedBy!: string;

  @Column('text')
  recordedAt!: string;
}
