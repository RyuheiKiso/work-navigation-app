// expo-network の状態を 4 段階の NetworkQuality に変換する
import * as Network from 'expo-network';
import type { NetworkQuality } from '@wnav/shared';

const POLL_INTERVAL_MS = 5000;

type Listener = (quality: NetworkQuality) => void;

export class NetworkMonitor {
  private timer: ReturnType<typeof setInterval> | null = null;
  private readonly listeners = new Set<Listener>();
  private lastQuality: NetworkQuality = 'disconnected';

  // 5 秒間隔でネットワーク状態をポーリングし、変化があれば listener に通知する
  start(): void {
    if (this.timer !== null) return;
    void this.poll();
    this.timer = setInterval(() => {
      void this.poll();
    }, POLL_INTERVAL_MS);
  }

  stop(): void {
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private async poll(): Promise<void> {
    const state = await Network.getNetworkStateAsync();
    const quality = this.toQuality(state);
    if (quality !== this.lastQuality) {
      this.lastQuality = quality;
      for (const listener of this.listeners) listener(quality);
    }
  }

  // 接続有無と接続種別から high/low/disconnected を判定する。emergency は Context 側で判定する
  private toQuality(state: Network.NetworkState): NetworkQuality {
    if (state.isConnected === false || state.isInternetReachable === false) {
      return 'disconnected';
    }
    if (state.type === Network.NetworkStateType.WIFI || state.type === Network.NetworkStateType.ETHERNET) {
      return 'high';
    }
    return 'low';
  }
}
