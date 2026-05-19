// TBL-008 steps。SOP 内ステップ。stepPayload は JSON 文字列で格納する
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('steps')
@Index(['sopVersionId', 'stepNumber'])
export class LocalStep {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  sopVersionId!: string;

  @Column('integer')
  stepNumber!: number;

  @Column('text')
  stepType!: string;

  @Column('text')
  titleJson!: string;

  @Column('text')
  instructionJson!: string;

  @Column('text')
  payload!: string;

  @Column('integer', { default: 0 })
  isMandatory!: boolean;

  @Column('integer', { default: 0 })
  requiresEvidence!: boolean;

  @Column('integer', { default: 0 })
  requiresSign!: boolean;

  @Column('integer', { default: 0 })
  skillLevelRequired!: number;

  @Column('integer', { default: 0 })
  estimatedSeconds!: number;

  @Column('text')
  fallbackType!: string;

  @Column('text')
  flowRules!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
