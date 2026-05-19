// TBL-012 andon_alerts。アラート種別と重大度を保持し騒音環境を考慮した UI 通知に使う
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('andon_alerts')
@Index(['status'])
@Index(['workExecutionId'])
export class LocalAndonAlert {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  alertType!: string;

  @Column('text')
  severity!: string;

  @Column('text')
  status!: string;

  @Column('text', { nullable: true })
  workExecutionId!: string | null;

  @Column('text', { nullable: true })
  stepId!: string | null;

  @Column('text')
  raisedBy!: string;

  @Column('text')
  title!: string;

  @Column('text')
  description!: string;

  @Column('text')
  raisedAt!: string;

  @Column('text', { nullable: true })
  acknowledgedBy!: string | null;

  @Column('text', { nullable: true })
  acknowledgedAt!: string | null;

  @Column('text', { nullable: true })
  resolvedBy!: string | null;

  @Column('text', { nullable: true })
  resolvedAt!: string | null;

  @Column('text', { nullable: true })
  resolutionNote!: string | null;
}
