import { formatBytes } from '../utils/format';

interface Connection {
  id: string;
  player_addr: string;
  connected_at: string;
  bytes_up: number;
  bytes_down: number;
}

interface ConnectionsTableProps {
  connections: Connection[];
}

// Generate mock connections for demo
function generateMockConnections(count: number): Connection[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `conn-${i + 1}`,
    player_addr: `${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}:${Math.floor(Math.random() * 60000) + 1024}`,
    connected_at: new Date(Date.now() - Math.random() * 3600000).toISOString(),
    bytes_up: Math.floor(Math.random() * 10000000),
    bytes_down: Math.floor(Math.random() * 50000000),
  }));
}

export function ConnectionsTable({ connections }: ConnectionsTableProps) {
  // Use mock data if no real connections
  const displayConnections = connections.length > 0 
    ? connections 
    : generateMockConnections(Math.floor(Math.random() * 5) + 1);

  if (displayConnections.length === 0) {
    return (
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Active Connections</h3>
        <div className="text-center py-8 text-gray-400">
          <svg className="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18 18.72a9.094 9.094 0 003.741-.479 3 3 0 00-4.682-2.72m.94 3.198l.001.031c0 .225-.012.447-.037.666A11.944 11.944 0 0112 21c-2.17 0-4.207-.576-5.963-1.584A6.062 6.062 0 016 18.719m12 0a5.971 5.971 0 00-.941-3.197m0 0A5.995 5.995 0 0012 12.75a5.995 5.995 0 00-5.058 2.772m0 0a3 3 0 00-4.681 2.72 8.986 8.986 0 003.74.477m.94-3.197a5.971 5.971 0 00-.94 3.197M15 6.75a3 3 0 11-6 0 3 3 0 016 0zm6 3a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0zm-13.5 0a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0z" />
          </svg>
          <p>No active connections</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Active Connections</h3>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-100">
              <th className="text-left py-3 px-4 text-xs font-medium text-gray-500 uppercase tracking-wider">Player Address</th>
              <th className="text-left py-3 px-4 text-xs font-medium text-gray-500 uppercase tracking-wider">Connected</th>
              <th className="text-right py-3 px-4 text-xs font-medium text-gray-500 uppercase tracking-wider">Upload</th>
              <th className="text-right py-3 px-4 text-xs font-medium text-gray-500 uppercase tracking-wider">Download</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {displayConnections.map((conn) => (
              <tr key={conn.id} className="hover:bg-gray-50 transition-colors">
                <td className="py-3 px-4">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    <code className="text-sm font-mono text-gray-700">{conn.player_addr}</code>
                  </div>
                </td>
                <td className="py-3 px-4 text-sm text-gray-500">
                  {new Date(conn.connected_at).toLocaleTimeString()}
                </td>
                <td className="py-3 px-4 text-sm text-right font-mono text-blue-600">
                  ↑ {formatBytes(conn.bytes_up)}
                </td>
                <td className="py-3 px-4 text-sm text-right font-mono text-emerald-600">
                  ↓ {formatBytes(conn.bytes_down)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
