// TBL-006 work_orders。作業指示マスタの端末キャッシュ
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('work_orders')
@Index(['status'])
@Index(['assignedTo'])
export class LocalWorkOrder {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  workOrderNumber!: string;

  @Column('text')
  productId!: string;

  @Column('text')
  sopId!: string;

  @Column('text')
  sopVersionId!: string;

  @Column('text')
  processId!: string;

  @Column('text', { nullable: true })
  lotId!: string | null;

  @Column('text')
  scheduledStart!: string;

  @Column('text', { nullable: true })
  scheduledEnd!: string | null;

  @Column('text')
  status!: string;

  @Column('text', { nullable: true })
  assignedTo!: string | null;

  @Column('text')
  createdAt!: string;

  @Column('text')
  updatedAt!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
