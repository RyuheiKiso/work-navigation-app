// TBL-031 hash_chain_blocks。Append-only、prevHash と contentHash で改竄検知チェーンを構成
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('hash_chain_blocks')
@Index(['blockNumber'])
export class LocalHashChainBlock {
  @PrimaryColumn('text')
  id!: string;

  @Column('integer')
  blockNumber!: number;

  @Column('text')
  prevHash!: string;

  @Column('text')
  contentHash!: string;

  @Column('text')
  payload!: string;

  @Column('text')
  createdAt!: string;
}
