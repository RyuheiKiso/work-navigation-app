// TBL-029 step_type_definitions。Step Engine が動的に評価する Step タイプの定義
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('step_type_definitions')
export class LocalStepTypeDefinition {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  typeCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  payloadSchema!: string;

  @Column('text')
  fallbackType!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
