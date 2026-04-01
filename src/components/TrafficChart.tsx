import { useState, useEffect } from 'react';

interface DataPoint {
  time: string;
  value: number;
}

interface TrafficChartProps {
  currentBytes: number;
}

export function TrafficChart({ currentBytes }: TrafficChartProps) {
  const [history, setHistory] = useState<DataPoint[]>([]);
  
  useEffect(() => {
    const now = new Date().toLocaleTimeString('ru-RU', { 
      hour: '2-digit', 
      minute: '2-digit', 
      second: '2-digit' 
    });
    
    setHistory(prev => {
      const newHistory = [...prev, { time: now, value: currentBytes }];
      // Keep last 20 points
      return newHistory.slice(-20);
    });
  }, [currentBytes]);

  if (history.length < 2) {
    return (
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Traffic History</h3>
        <div className="h-48 flex items-center justify-center text-gray-400">
          Collecting data...
        </div>
      </div>
    );
  }

  const maxValue = Math.max(...history.map(h => h.value));
  const minValue = Math.min(...history.map(h => h.value));
  const range = maxValue - minValue || 1;

  const points = history.map((point, index) => {
    const x = (index / (history.length - 1)) * 100;
    const y = 100 - ((point.value - minValue) / range) * 80 - 10;
    return `${x},${y}`;
  }).join(' ');

  const areaPoints = `0,100 ${points} 100,100`;

  return (
    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Traffic History</h3>
      <div className="h-48 relative">
        <svg 
          viewBox="0 0 100 100" 
          preserveAspectRatio="none" 
          className="w-full h-full"
        >
          {/* Gradient definition */}
          <defs>
            <linearGradient id="areaGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stopColor="#8B5CF6" stopOpacity="0.3" />
              <stop offset="100%" stopColor="#8B5CF6" stopOpacity="0.05" />
            </linearGradient>
          </defs>
          
          {/* Grid lines */}
          {[20, 40, 60, 80].map(y => (
            <line 
              key={y} 
              x1="0" 
              y1={y} 
              x2="100" 
              y2={y} 
              stroke="#E5E7EB" 
              strokeWidth="0.3" 
            />
          ))}
          
          {/* Area fill */}
          <polygon 
            points={areaPoints} 
            fill="url(#areaGradient)" 
          />
          
          {/* Line */}
          <polyline
            points={points}
            fill="none"
            stroke="#8B5CF6"
            strokeWidth="0.8"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          
          {/* Current point */}
          {history.length > 0 && (
            <circle
              cx="100"
              cy={100 - ((history[history.length - 1].value - minValue) / range) * 80 - 10}
              r="1.5"
              fill="#8B5CF6"
              className="animate-pulse"
            />
          )}
        </svg>
        
        {/* Labels */}
        <div className="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-gray-400 pt-2">
          <span>{history[0]?.time}</span>
          <span>{history[history.length - 1]?.time}</span>
        </div>
      </div>
    </div>
  );
}
