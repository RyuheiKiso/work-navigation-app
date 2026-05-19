// TBL-042 lot_qc_states。ロット単位の QC ステータスを更新管理する
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('lot_qc_states')
export class LocalLotQcState {
  @PrimaryColumn('text')
  lotId!: string;

  @Column('text')
  qcStatus!: string;

  @Column('text', { nullable: true })
  concessionApprovalId!: string | null;

  @Column('text', { nullable: true })
  validUntil!: string | null;

  @Column('text')
  updatedAt!: string;
}
