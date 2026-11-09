import React from 'react'

export interface AppLayoutProps {
  sidebar: React.ReactNode
  list: React.ReactNode
  detail: React.ReactNode
  isReadingMobile?: boolean
}

export const AppLayout: React.FC<AppLayoutProps> = ({
  sidebar,
  list,
  detail,
  isReadingMobile = false,
}) => {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100 antialiased font-sans">
      {/* 1. Left Navigation Rail (Desktop) */}
      <aside className="hidden md:flex w-64 flex-col border-r border-slate-200 dark:border-slate-800/80 bg-white dark:bg-slate-900 shrink-0 select-none">
        {sidebar}
      </aside>

      {/* 2. Middle Message List Column */}
      <section
        className={`w-full md:w-88 lg:w-96 flex flex-col border-r border-slate-200 dark:border-slate-800/80 bg-slate-50/50 dark:bg-slate-900/40 shrink-0 ${
          isReadingMobile ? 'hidden md:flex' : 'flex'
        }`}
      >
        {list}
      </section>

      {/* 3. Right Message Reader Canvas */}
      <main
        className={`flex-1 flex flex-col min-w-0 bg-white dark:bg-slate-950 overflow-hidden ${
          !isReadingMobile ? 'hidden md:flex' : 'flex'
        }`}
      >
        {detail}
      </main>
    </div>
  )
}

