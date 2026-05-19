// TBL-019 user_roles。ユーザーとロールの多対多関連
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('user_roles')
@Index(['userId'])
@Index(['roleId'])
export class LocalUserRole {
  @PrimaryColumn('text')
  userId!: string;

  @PrimaryColumn('text')
  roleId!: string;

  @Column('text')
  grantedAt!: string;

  @Column('text', { nullable: true })
  grantedBy!: string | null;
}
