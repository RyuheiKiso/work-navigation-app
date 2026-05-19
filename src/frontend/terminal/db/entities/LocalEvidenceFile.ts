// TBL-009 evidence_files。Append-only、SHA-256 ハッシュは Exif 除去後に算出する
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('evidence_files')
@Index(['stepId'])
@Index(['workExecutionId'])
export class LocalEvidenceFile {
  @PrimaryColumn('text')
  id!: string;

  @Column('text')
  workExecutionId!: string;

  @Column('text')
  stepId!: string;

  @Column('text')
  evidenceType!: string;

  @Column('text')
  filePath!: string;

  @Column('text')
  fileHashSha256!: string;

  @Column('integer', { default: 0 })
  fileSizeBytes!: number;

  @Column('integer', { nullable: true })
  widthPx!: number | null;

  @Column('integer', { nullable: true })
  heightPx!: number | null;

  @Column('text')
  description!: string;

  @Column('text')
  uploadedBy!: string;

  @Column('text')
  uploadedAt!: string;

  @Column('integer', { default: 0 })
  synced!: boolean;
}
