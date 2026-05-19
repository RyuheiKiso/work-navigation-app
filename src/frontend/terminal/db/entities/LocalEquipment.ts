// TBL-025 equipments。設備マスタ
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('equipments')
export class LocalEquipment {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  equipmentCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text', { nullable: true })
  locationCode!: string | null;

  @Column('integer', { default: 1 })
  isActive!: boolean;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
