// TBL-037 suppliers。仕入先マスタ
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('suppliers')
export class LocalSupplier {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  supplierCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text', { nullable: true })
  contactEmail!: string | null;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
