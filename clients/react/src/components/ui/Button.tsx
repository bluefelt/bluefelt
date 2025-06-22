import { ButtonHTMLAttributes, ReactNode } from 'react';
import { theme } from '../../theme/constants';
import { cn } from '../../utils/cn';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'outline' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  children: ReactNode;
}

function Button({ 
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
    warning: theme.colors.button.warning,
    outline: 'border border-gray-400 text-gray-700 hover:bg-gray-50',
    ghost: 'text-gray-600 hover:bg-gray-100'
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

export default Button;
export { Button };