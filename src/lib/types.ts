export interface Sound {
  id: string;
  name: string;
  file_path: string;
  is_bundled: boolean;
  is_active: boolean;
  volume: number;
}

export interface AppState {
  sounds: Sound[];
  master_volume: number;
  is_paused: boolean;
  crossfade_duration: number;
  autostart_enabled: boolean;
  autopause_on_lock: boolean;
  auto_update_enabled: boolean;
}
