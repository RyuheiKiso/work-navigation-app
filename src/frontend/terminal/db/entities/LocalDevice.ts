// TBL-033 devices。端末マスタ、ターミナルコードで PG と照合
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('devices')
@Index(['terminalCode'], { unique: true })
@Index(['factoryId'])
export class LocalDevice {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  terminalCode!: string;

  @Column('text', { nullable: true })
  externalKey!: string | null;

  @Column('text')
  factoryId!: string;

  @Column('integer', { default: 1 })
  isActive!: boolean;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
