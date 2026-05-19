// TBL-022 operations。作業種別マスタ
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('operations')
@Index(['processId'])
export class LocalOperation {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  operationCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  processId!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
