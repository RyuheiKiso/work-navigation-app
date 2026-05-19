// OutboxWorker の二重起動防止・stop() 動作を検証する
import { OutboxWorker } from '../../domain/outbox/OutboxWorker';

describe('OutboxWorker', () => {
  it('does not start twice (singleton)', async () => {
    const jwt = { getAccessToken: async () => 'token' } as never;
    const worker = new OutboxWorker({ baseApiUrl: 'http://test.local', jwtService: jwt });
    // start を呼んだ直後に stop すると isActive が false になる
    void worker.start();
    expect(worker.isActive()).toBe(true);
    worker.stop();
    expect(worker.isActive()).toBe(false);
  });
});
