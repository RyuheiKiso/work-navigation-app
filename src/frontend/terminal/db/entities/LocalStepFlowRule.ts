// TBL-030 step_flow_rules。JSON Logic 形式の遷移ルール、eval 禁止のため json-logic-js で評価
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('step_flow_rules')
@Index(['stepId'])
export class LocalStepFlowRule {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  stepId!: string;

  @Column('text')
  ruleJson!: string;

  @Column('text')
  onPassNextStepId!: string;

  @Column('text', { nullable: true })
  onFailNextStepId!: string | null;
}
