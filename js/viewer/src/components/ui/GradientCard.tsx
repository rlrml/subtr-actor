import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface GradientCardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
  onClick?: () => void;
}

export function GradientCard({
  children,
  className,
  hover = false,
  onClick,
}: GradientCardProps) {
  return (
    <div
      onClick={onClick}
      className={cn(
        // Outer container with gradient border
        'relative rounded-xl p-[1px]',
        'bg-gradient-to-br from-violet-500/50 to-blue-500/50',
        hover && 'cursor-pointer transition-all hover:from-violet-500 hover:to-blue-500',
        onClick && 'cursor-pointer',
        className
      )}
    >
      {/* Inner content with dark background */}
      <div className="rounded-[11px] bg-gray-900 p-6 h-full">
        {children}
      </div>
    </div>
  );
}

// Variant with glow effect
export function GlowCard({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'relative rounded-xl p-[1px]',
        'bg-gradient-to-br from-violet-500 to-blue-500',
        'shadow-lg shadow-violet-500/20',
        className
      )}
    >
      <div className="rounded-[11px] bg-gray-900 p-6 h-full">
        {children}
      </div>
    </div>
  );
}

// Variant for auth forms - visible gradient border like GlowCard
export function AuthCard({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'relative rounded-2xl p-[1px]',
        'bg-gradient-to-br from-violet-500 to-blue-500',
        'shadow-lg shadow-violet-500/20',
        className
      )}
    >
      <div className="rounded-[15px] bg-gray-900 p-6 h-full">
        {children}
      </div>
    </div>
  );
}
