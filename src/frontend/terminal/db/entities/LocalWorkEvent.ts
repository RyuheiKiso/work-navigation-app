// TBL-001 work_events の端末ミラー。Append-only で UPDATE/DELETE 禁止
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';
import { boolTransformer } from './_transformers';

@Entity('work_events')
@Index(['caseId'])
@Index(['caseId', 'stepId'])
@Index(['synced'])
export class LocalWorkEvent {
  @PrimaryColumn('text')
  eventId!: string;

  @Column('text')
  caseId!: string;

  @Column('text')
  activity!: string;

  @Column('text')
  timestampClient!: string;

  @Column('text')
  resource!: string;

  @Column('text')
  sopVersionId!: string;

  @Column('text')
  stepId!: string;

  @Column('text')
  payload!: string;

  @Column('text')
  prevHash!: string;

  @Column('text')
  contentHash!: string;

  @Column('text')
  terminalId!: string;

  @Column('integer', { default: 0, transformer: boolTransformer })
  synced!: boolean;
}
