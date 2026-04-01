import { ReactNode } from 'react';

interface StatCardProps {
  title: string;
  value: string | number;
  icon: ReactNode;
  subtitle?: string;
  trend?: 'up' | 'down' | 'neutral';
  color?: 'blue' | 'green' | 'purple' | 'orange' | 'red';
}

const colorClasses = {
  blue: 'from-blue-500 to-blue-600',
  green: 'from-emerald-500 to-emerald-600',
  purple: 'from-purple-500 to-purple-600',
  orange: 'from-orange-500 to-orange-600',
  red: 'from-red-500 to-red-600',
};

const iconBgClasses = {
  blue: 'bg-blue-500/10',
  green: 'bg-emerald-500/10',
  purple: 'bg-purple-500/10',
  orange: 'bg-orange-500/10',
  red: 'bg-red-500/10',
};

const iconTextClasses = {
  blue: 'text-blue-500',
  green: 'text-emerald-500',
  purple: 'text-purple-500',
  orange: 'text-orange-500',
  red: 'text-red-500',
};

export function StatCard({ title, value, icon, subtitle, color = 'blue' }: StatCardProps) {
  return (
    <div className="relative overflow-hidden bg-white rounded-2xl shadow-sm border border-gray-100 p-6 hover:shadow-md transition-shadow duration-200">
      <div className="flex items-start justify-between">
        <div className="space-y-2">
          <p className="text-sm font-medium text-gray-500">{title}</p>
          <p className="text-3xl font-bold text-gray-900">{value}</p>
          {subtitle && (
            <p className="text-sm text-gray-400">{subtitle}</p>
          )}
        </div>
        <div className={`p-3 rounded-xl ${iconBgClasses[color]}`}>
          <div className={iconTextClasses[color]}>
            {icon}
          </div>
        </div>
      </div>
      
      {/* Decorative gradient line at bottom */}
      <div className={`absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r ${colorClasses[color]}`} />
    </div>
  );
}
