// Types for mc-tunnel dashboard

export interface TunnelStats {
  active_connections: number;
  total_bytes_transferred: number;
  tunnel_status: 'connected' | 'disconnected';
  uptime_secs: number;
  peak_connections: number;
  total_connections: number;
}

export interface ConnectionInfo {
  id: string;
  player_addr: string;
  connected_at: string;
  bytes_up: number;
  bytes_down: number;
}
