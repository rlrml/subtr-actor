interface CategoryBadgeProps {
  name: string;
  color: string;
  icon?: string | null;
  size?: 'sm' | 'md';
}

export function CategoryBadge({ name, color, size = 'md' }: CategoryBadgeProps) {
  const sizeClasses = size === 'sm' ? 'px-2 py-0.5 text-xs' : 'px-2.5 py-1 text-sm';

  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full font-medium ${sizeClasses}`}
      style={{
        backgroundColor: `${color}20`,
        color: color,
      }}
    >
      {name}
    </span>
  );
}
