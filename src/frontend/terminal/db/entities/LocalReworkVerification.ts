// TBL-045 rework_verifications。修正後の再検査結果、検査者は別作業者であること必須
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';
import { boolTransformer } from './_transformers';

@Entity('rework_verifications')
@Index(['reworkId'])
export class LocalReworkVerification {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  reworkId!: string;

  @Column('text')
  verifierId!: string;

  @Column('text')
  verifiedAt!: string;

  @Column('integer', { default: 0, transformer: boolTransformer })
  passed!: boolean;

  @Column('text')
  note!: string;

  @Column('text')
  evidenceIds!: string;
}
