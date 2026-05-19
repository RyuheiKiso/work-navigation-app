// 端末設定は singleton レコードを INSERT OR REPLACE で管理する
import { getDataSource } from '../data-source';
import { LocalAppSettings } from '../entities/LocalAppSettings';

const SINGLETON_ID = 'singleton';

export class AppSettingsRepository {
  private get repo() {
    return getDataSource().getRepository(LocalAppSettings);
  }

  async get(): Promise<LocalAppSettings | null> {
    return this.repo.findOne({ where: { settingsId: SINGLETON_ID } });
  }

  async save(settings: Omit<LocalAppSettings, 'settingsId'>): Promise<void> {
    await this.repo.save({ settingsId: SINGLETON_ID, ...settings });
  }

  async patch(patch: Partial<LocalAppSettings>): Promise<void> {
    const current = await this.get();
    if (current === null) return;
    await this.repo.save({ ...current, ...patch });
  }
}
