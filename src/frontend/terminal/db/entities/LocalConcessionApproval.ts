// TBL-041 concession_approvals。特採承認、電子サインと有効範囲を紐付け
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('concession_approvals')
@Index(['incomingInspectionId'])
export class LocalConcessionApproval {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  incomingInspectionId!: string;

  @Column('text')
  requestedBy!: string;

  @Column('text', { nullable: true })
  approvedBy!: string | null;

  @Column('text', { nullable: true })
  approvalSign!: string | null;

  @Column('text', { nullable: true })
  electronicSignId!: string | null;

  @Column('text')
  conditionNote!: string;

  @Column('text')
  validityScope!: string;

  @Column('text', { nullable: true })
  validUntil!: string | null;

  @Column('text', { default: 'PENDING' })
  status!: string;

  @Column('text')
  createdAt!: string;
}
