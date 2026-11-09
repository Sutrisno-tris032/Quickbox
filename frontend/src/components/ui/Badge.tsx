import React from 'react'
import { cn } from '../../lib/utils'

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info'
}

export const Badge: React.FC<BadgeProps> = ({
  className,
  variant = 'default',
  ...props
}) => {
  return (
    <span
      className={cn(
        'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold',
        {
          'bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-200':
            variant === 'default',
          'bg-emerald-100 text-emerald-800 dark:bg-emerald-950/50 dark:text-emerald-400':
            variant === 'success',
          'bg-amber-100 text-amber-800 dark:bg-amber-950/50 dark:text-amber-400':
            variant === 'warning',
          'bg-rose-100 text-rose-800 dark:bg-rose-950/50 dark:text-rose-400':
            variant === 'danger',
          'bg-blue-100 text-blue-800 dark:bg-blue-950/50 dark:text-blue-400':
            variant === 'info',
        },
        className
      )}
      {...props}
    />
  )
}

