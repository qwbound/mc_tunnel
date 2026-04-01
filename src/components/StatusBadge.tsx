interface StatusBadgeProps {
  status: 'connected' | 'disconnected';
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const isConnected = status === 'connected';
  
  return (
    <div className={`inline-flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium ${
      isConnected 
        ? 'bg-emerald-100 text-emerald-800' 
        : 'bg-red-100 text-red-800'
    }`}>
      <span className={`w-2.5 h-2.5 rounded-full ${
        isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'
      }`} />
      {isConnected ? 'Connected' : 'Disconnected'}
    </div>
  );
}
