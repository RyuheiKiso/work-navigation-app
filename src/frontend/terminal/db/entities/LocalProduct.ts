// TBL-023 products。製品マスタ
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('products')
export class LocalProduct {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  productCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
