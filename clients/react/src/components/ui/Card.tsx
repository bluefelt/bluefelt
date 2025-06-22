import { ReactNode } from 'react';
import { theme } from '../../theme/constants';
import { cn } from '../../utils/cn';

interface CardProps {
  children: ReactNode;
  className?: string;
  variant?: 'default' | 'secondary';
}

function Card({ children, className, variant = 'default' }: CardProps) {
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

export default Card;
export { Card };