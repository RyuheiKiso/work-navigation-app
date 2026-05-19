// TBL-011 suspensions。Append-only、再開時刻は SuspensionFlow が UPDATE する例外列
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('suspensions')
@Index(['workExecutionId'])
export class LocalSuspension {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  workExecutionId!: string;

  @Column('text')
  reasonCode!: string;

  @Column('text')
  reasonDetail!: string;

  @Column('text')
  suspendedAt!: string;

  @Column('text', { nullable: true })
  resumedAt!: string | null;

  @Column('text', { nullable: true })
  resumedBy!: string | null;
}
