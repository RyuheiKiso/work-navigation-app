// TBL-049 scrap_records。廃棄処理記録、立会者サインを伴う
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('scrap_records')
@Index(['nonconformityId'])
export class LocalScrapRecord {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  nonconformityId!: string;

  @Column('text')
  scrappedBy!: string;

  @Column('text')
  witnessId!: string;

  @Column('text')
  scrappedAt!: string;

  @Column('real')
  quantity!: number;

  @Column('text')
  note!: string;
}
