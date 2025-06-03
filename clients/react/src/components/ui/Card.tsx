import { ReactNode } from 'react';
import { theme, cn } from '../../theme/constants';

interface CardProps {
  children: ReactNode;
  className?: string;
  variant?: 'default' | 'secondary';
}

export default function Card({ children, className, variant = 'default' }: CardProps) {
  const variantClasses = {
    default: theme.colors.background.secondary,
    secondary: theme.colors.background.tertiary
  };

  return (
    <div className={cn(
      variantClasses[variant],
      theme.spacing.card,
      className
    )}>
      {children}
    </div>
  );
}