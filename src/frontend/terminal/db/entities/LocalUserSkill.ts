// TBL-020 user_skills。ユーザーが保有するスキルレベル
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('user_skills')
@Index(['userId'])
export class LocalUserSkill {
  @PrimaryColumn('text')
  userId!: string;

  @PrimaryColumn('text')
  skillId!: string;

  @Column('integer', { default: 1 })
  level!: number;

  @Column('text')
  acquiredAt!: string;

  @Column('text', { nullable: true })
  expiresAt!: string | null;
}
