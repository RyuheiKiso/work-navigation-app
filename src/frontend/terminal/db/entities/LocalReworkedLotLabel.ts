// TBL-047 reworked_lot_labels。修正済ロットのラベル印字情報
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('reworked_lot_labels')
@Index(['reworkId'])
export class LocalReworkedLotLabel {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  reworkId!: string;

  @Column('text')
  labelCode!: string;

  @Column('text')
  qrPayload!: string;

  @Column('text')
  printedAt!: string;

  @Column('text')
  printedBy!: string;
}
