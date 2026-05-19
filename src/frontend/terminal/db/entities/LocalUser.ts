// TBL-016 users。ユーザー情報、displayNameJson は多言語 JSON
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('users')
@Index(['loginId'], { unique: true })
@Index(['factoryId'])
export class LocalUser {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  loginId!: string;

  @Column('text')
  username!: string;

  @Column('text')
  displayNameJson!: string;

  @Column('text', { nullable: true })
  email!: string | null;

  @Column('text')
  role!: string;

  @Column('text')
  roles!: string;

  @Column('text')
  factoryId!: string;

  @Column('text', { default: 'ja' })
  locale!: string;

  @Column('integer', { default: 1 })
  isActive!: boolean;

  @Column('text')
  createdAt!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
