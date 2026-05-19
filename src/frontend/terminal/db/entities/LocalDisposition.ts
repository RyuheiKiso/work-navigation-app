// TBL-044 dispositions。処分（修理・廃棄・返品・特採）決定
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('dispositions')
@Index(['nonconformityId'])
export class LocalDisposition {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  nonconformityId!: string;

  @Column('text')
  dispositionType!: string;

  @Column('text')
  decision!: string;

  @Column('text')
  decisionReason!: string;

  @Column('text', { nullable: true })
  qualityAdminSignId!: string | null;

  @Column('text', { nullable: true })
  supervisorSignId!: string | null;

  @Column('text', { nullable: true })
  signedAt!: string | null;

  @Column('text')
  createdAt!: string;
}
