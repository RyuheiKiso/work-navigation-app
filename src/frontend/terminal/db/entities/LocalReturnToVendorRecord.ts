// TBL-050 return_to_vendor_records。仕入先返却の追跡番号記録
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('return_to_vendor_records')
@Index(['nonconformityId'])
@Index(['supplierId'])
export class LocalReturnToVendorRecord {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  nonconformityId!: string;

  @Column('text')
  supplierId!: string;

  @Column('text')
  trackingNo!: string;

  @Column('text')
  returnedBy!: string;

  @Column('text')
  returnedAt!: string;

  @Column('real')
  quantity!: number;
}
