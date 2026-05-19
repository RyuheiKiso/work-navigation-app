// TBL-024 lots。ロット番号と製品の紐付け、外部キーは MES 連携用
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('lots')
@Index(['productId'])
@Index(['lotNumber'], { unique: true })
export class LocalLot {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  lotNumber!: string;

  @Column('text')
  productId!: string;

  @Column('text', { nullable: true })
  externalKey!: string | null;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
