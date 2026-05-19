// TBL-007 sops。マスタ（版管理）、JSONB 多言語テキストは TEXT として保存
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('sops')
@Index(['operationId'])
export class LocalSop {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  sopCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  descriptionJson!: string;

  @Column('text')
  sopType!: string;

  @Column('text')
  processId!: string;

  @Column('text')
  operationId!: string;

  @Column('text', { nullable: true })
  currentVersionId!: string | null;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
