import { ButtonHTMLAttributes, ReactNode } from 'react';
import { theme, cn } from '../../theme/constants';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning';
  size?: 'sm' | 'md' | 'lg';
  children: ReactNode;
}

export default function Button({ 
  variant = 'primary', 
  size = 'md',
  className,
  children,
  ...props 
}: ButtonProps) {
  const variantClasses = {
    primary: theme.colors.button.primary,
    secondary: theme.colors.button.secondary,
    success: theme.colors.button.success,
    danger: theme.colors.button.danger,
    warning: theme.colors.button.warning
  };

  const sizeClasses = {
    sm: 'px-3 py-1 text-sm',
    md: 'px-4 py-2',
    lg: 'px-6 py-3 text-lg'
  };

  return (
    <button
      className={cn(
        'rounded-lg text-white font-medium',
        theme.transitions.default,
        variantClasses[variant],
        sizeClasses[size],
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
}