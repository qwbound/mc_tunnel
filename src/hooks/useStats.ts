import { useState, useEffect, useCallback } from 'react';
import type { TunnelStats } from '../types';

// Mock data for demo (when API is not available)
const generateMockStats = (): TunnelStats => ({
  active_connections: Math.floor(Math.random() * 10),
  total_bytes_transferred: Math.floor(Math.random() * 1000000000) + 50000000,
  tunnel_status: Math.random() > 0.1 ? 'connected' : 'disconnected',
  uptime_secs: Math.floor(Date.now() / 1000) % 86400,
  peak_connections: Math.floor(Math.random() * 20) + 5,
  total_connections: Math.floor(Math.random() * 100) + 10,
  active_ips: [
    '192.168.1.100',
    '10.0.0.50', 
    '172.16.0.25'
  ].slice(0, Math.floor(Math.random() * 3) + 1),
});

export function useStats(apiUrl: string, refreshInterval: number = 2000) {
  const [stats, setStats] = useState<TunnelStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [useMock] = useState(false);

  const fetchStats = useCallback(async () => {
    if (useMock) {
      setStats(generateMockStats());
      setLoading(false);
      return;
    }

    try {
      const response = await fetch(`${apiUrl}/api/stats`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const data = await response.json();
      setStats(data);
      setError(null);
    } catch (err) {
      console.error('API Error:', err);
      setError('Failed to connect to Rust backend');
      // setStats(null); // Можно занулить, чтобы не видеть старые данные
    } finally {
      setLoading(false);
    }
  },
  [apiUrl, useMock]);
  
  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, refreshInterval);
    return () => clearInterval(interval);
  }, [fetchStats, refreshInterval]);

  return { stats, error, loading, refetch: fetchStats };
}
