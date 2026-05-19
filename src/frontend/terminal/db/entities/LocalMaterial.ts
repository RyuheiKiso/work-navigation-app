// TBL-036 materials。材料マスタ、IQC で照合する
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('materials')
export class LocalMaterial {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  materialCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  materialType!: string;

  @Column('text')
  unit!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
