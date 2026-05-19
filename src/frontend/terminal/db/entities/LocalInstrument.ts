// TBL-026 instruments。計測器マスタ、校正期限と関連付ける
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('instruments')
export class LocalInstrument {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  instrumentCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text', { nullable: true })
  calibrationDueAt!: string | null;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
