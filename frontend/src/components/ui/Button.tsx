import React from 'react'
import { cn } from '../../lib/utils'

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center font-medium transition-colors rounded-xl focus:outline-none disabled:opacity-50 disabled:pointer-events-none cursor-pointer',
          {
            'bg-slate-900 text-white hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-900 dark:hover:bg-white shadow-sm':
              variant === 'primary',
            'bg-slate-100 text-slate-800 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700':
              variant === 'secondary',
            'border border-slate-200 hover:bg-slate-100 dark:border-slate-800 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300':
              variant === 'outline',
            'hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300':
              variant === 'ghost',
            'bg-rose-500 text-white hover:bg-rose-600': variant === 'danger',
            'text-xs px-2.5 py-1.5 h-8 gap-1.5': size === 'sm',
            'text-sm px-4 py-2 h-10 gap-2': size === 'md',
            'text-base px-6 py-3 h-12 gap-2.5': size === 'lg',
          },
          className
        )}
        {...props}
      />
    )
  }
)
Button.displayName = 'Button'
