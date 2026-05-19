// TBL-018 skills。スキル定義（レベル付き）。Step.skillLevelRequired と紐付けて表示制御に使う
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('skills')
export class LocalSkill {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  skillCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('integer', { default: 1 })
  level!: number;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
