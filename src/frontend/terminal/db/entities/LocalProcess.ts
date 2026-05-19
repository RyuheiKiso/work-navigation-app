// TBL-021 processes。工程マスタ
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('processes')
export class LocalProcess {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  processCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  descriptionJson!: string;

  @Column('integer', { default: 1 })
  isActive!: boolean;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
