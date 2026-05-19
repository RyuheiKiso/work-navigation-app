// TBL-017 roles。RBAC ロール定義
import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('roles')
export class LocalRole {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  roleCode!: string;

  @Column('text')
  nameJson!: string;

  @Column('text')
  descriptionJson!: string;

  @Column('text', { nullable: true })
  deletedAt!: string | null;
}
