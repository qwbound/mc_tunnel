import { useState } from 'react';
import { useStats } from './hooks/useStats';
import { StatusBadge } from './components/StatusBadge';
import { StatCard } from './components/StatCard';
import { TrafficChart } from './components/TrafficChart';
import { ConnectionsTable } from './components/ConnectionsTable';
import { formatBytes, formatUptime, formatNumber } from './utils/format';

// Icons as components
const UsersIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
  </svg>
);

const ChartIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
  </svg>
);

const ClockIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

const ServerIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
  </svg>
);

const TrendUpIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 18L9 11.25l4.306 4.307a11.95 11.95 0 015.814-5.519l2.74-1.22m0 0l-5.94-2.28m5.94 2.28l-2.28 5.941" />
  </svg>
);

const LinkIcon = () => (
  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m13.35-.622l1.757-1.757a4.5 4.5 0 00-6.364-6.364l-4.5 4.5a4.5 4.5 0 001.242 7.244" />
  </svg>
);

export default function App() {
  const [apiUrl] = useState('http://localhost:3001');
  const { stats, loading } = useStats(apiUrl);

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-purple-50 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-flex items-center justify-center w-16 h-16 mb-4">
            <div className="w-12 h-12 border-4 border-purple-200 border-t-purple-600 rounded-full animate-spin" />
          </div>
          <p className="text-gray-500">Loading tunnel stats...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-purple-50">
      {/* Header */}
      <header className="bg-white/80 backdrop-blur-sm border-b border-gray-100 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center shadow-lg shadow-purple-200">
                <span className="text-white text-xl">⛏️</span>
              </div>
              <div>
                <h1 className="text-xl font-bold text-gray-900">MC-Tunnel</h1>
                <p className="text-xs text-gray-500">Dashboard v0.2</p>
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              {stats && <StatusBadge status={stats.tunnel_status} />}
              <div className="text-sm text-gray-500">
                Last update: <span className="font-mono">{new Date().toLocaleTimeString()}</span>
              </div>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
          <StatCard
            title="Active Connections"
            value={stats?.active_connections ?? 0}
            icon={<UsersIcon />}
            subtitle="Players online now"
            color="blue"
          />
          
          <StatCard
            title="Total Traffic"
            value={formatBytes(stats?.total_bytes_transferred ?? 0)}
            icon={<ChartIcon />}
            subtitle="Total transferred"
            color="purple"
          />
          
          <StatCard
            title="Uptime"
            value={formatUptime(stats?.uptime_secs ?? 0)}
            icon={<ClockIcon />}
            subtitle="Since last restart"
            color="green"
          />
          
          <StatCard
            title="Peak Connections"
            value={stats?.peak_connections ?? 0}
            icon={<TrendUpIcon />}
            subtitle="Maximum concurrent"
            color="orange"
          />
          
          <StatCard
            title="Total Connections"
            value={formatNumber(stats?.total_connections ?? 0)}
            icon={<LinkIcon />}
            subtitle="All-time connections"
            color="blue"
          />
          
          <StatCard
            title="Tunnel Status"
            value={stats?.tunnel_status === 'connected' ? 'Online' : 'Offline'}
            icon={<ServerIcon />}
            subtitle={stats?.tunnel_status === 'connected' ? 'VPS ↔ Client' : 'Waiting for client'}
            color={stats?.tunnel_status === 'connected' ? 'green' : 'red'}
          />
        </div>

        {/* Charts and Tables */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          <TrafficChart currentBytes={stats?.total_bytes_transferred ?? 0} />
          <ConnectionsTable connections={[]} />
        </div>

        {/* Info Section */}
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">ℹ️ API Information</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="bg-gray-50 rounded-xl p-4">
              <p className="text-sm font-medium text-gray-700 mb-2">Stats Endpoint</p>
              <code className="text-sm font-mono bg-gray-100 px-2 py-1 rounded text-purple-600">
                GET /api/stats
              </code>
            </div>
            <div className="bg-gray-50 rounded-xl p-4">
              <p className="text-sm font-medium text-gray-700 mb-2">Response Format</p>
              <pre className="text-xs font-mono bg-gray-100 p-2 rounded text-gray-600 overflow-x-auto">
{`{
  "active_connections": 5,
  "total_bytes_transferred": 1234567890,
  "tunnel_status": "connected",
  "uptime_secs": 3600,
  "peak_connections": 12,
  "total_connections": 156
}`}
              </pre>
            </div>
          </div>
        </div>

        {/* Rust Integration Guide */}
        <div className="mt-6 bg-gradient-to-r from-purple-50 to-indigo-50 rounded-2xl border border-purple-100 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">🦀 Rust Integration</h3>
          <p className="text-sm text-gray-600 mb-4">
            This dashboard is designed to work with the mc-tunnel Rust backend. 
            The backend should expose a REST API on port 3001 with the <code className="bg-white px-1 rounded">/api/stats</code> endpoint.
          </p>
          <div className="flex items-center gap-2">
            <span className="px-2 py-1 bg-purple-100 text-purple-700 text-xs font-medium rounded">Demo Mode</span>
            <span className="text-xs text-gray-500">Currently showing mock data</span>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-gray-100 bg-white/50 mt-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between text-sm text-gray-500">
            <p>MC-Tunnel Dashboard • Made with ❤️ by qwbound + Claude</p>
            <a 
              href="https://github.com/qwbound/mc_tunnel" 
              target="_blank" 
              rel="noopener noreferrer"
              className="hover:text-purple-600 transition-colors"
            >
              GitHub →
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
