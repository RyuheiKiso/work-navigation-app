// TBL-002 electronic_signs。電子サインは Ed25519 署名値を hex で保存する
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('electronic_signs')
@Index(['contextType', 'contextId'])
export class LocalElectronicSign {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  signerId!: string;

  @Column('text')
  signedContentHash!: string;

  @Column('text')
  contextType!: string;

  @Column('text')
  contextId!: string;

  @Column('text', { nullable: true })
  stepId!: string | null;

  @Column('text')
  signedAt!: string;

  @Column('text')
  hashChainBlockId!: string;

  @Column('text')
  hashChainValue!: string;

  @Column('text')
  hashChainPrev!: string;

  @Column('text')
  deviceId!: string;

  @Column('integer', { default: 0 })
  synced!: boolean;
}
