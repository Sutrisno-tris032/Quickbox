import React from 'react'

export interface AppLayoutProps {
  topNavbar?: React.ReactNode
  statusBar?: React.ReactNode
  mailboxNav: React.ReactNode
  sidebar: React.ReactNode
  list: React.ReactNode
  detail: React.ReactNode
  isReadingMobile?: boolean
}

export const AppLayout: React.FC<AppLayoutProps> = ({
  topNavbar,
  statusBar,
  mailboxNav,
  sidebar,
  list,
  detail,
  isReadingMobile = false,
}) => {
  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#F8FAFC] dark:bg-[#0B0F19] text-slate-900 dark:text-slate-100 antialiased font-sans">
      {/* 1. Global Top Navbar */}
      {topNavbar && (
        <header className="h-13 shrink-0 border-b border-slate-200/90 dark:border-slate-800 bg-white dark:bg-[#0F172A] z-20">
          {topNavbar}
        </header>
      )}

      {/* 2. Sub Status Banner / Node Status */}
      {statusBar && (
        <div className="h-7 shrink-0 border-b border-slate-200/60 dark:border-slate-800/80 bg-slate-50 dark:bg-[#090D16] text-[11px] px-4 flex items-center justify-between z-10">
          {statusBar}
        </div>
      )}

      {/* 3. Main Multi-Column Workspace */}
      <div className="flex flex-1 min-h-0 overflow-hidden">
        {/* Rail Column 1: Mailbox Types / Categories */}
        <aside className="hidden lg:flex w-52 flex-col border-r border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-[#0F172A] shrink-0 select-none">
          {mailboxNav}
        </aside>

        {/* Rail Column 2: Riwayat Inbox (Generated Mailboxes) */}
        <aside className="hidden md:flex w-64 xl:w-72 flex-col border-r border-slate-200/80 dark:border-slate-800/80 bg-[#FBFBFC] dark:bg-[#0D1322] shrink-0 select-none">
          {sidebar}
        </aside>

        {/* Rail Column 3: Pesan Masuk (Message List) */}
        <section
          className={`w-full md:w-80 lg:w-88 xl:w-96 flex flex-col border-r border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-[#0F172A] shrink-0 ${
            isReadingMobile ? 'hidden md:flex' : 'flex'
          }`}
        >
          {list}
        </section>

        {/* Rail Column 4: Reader & Action Panel (Message Detail) */}
        <main
          className={`flex-1 flex flex-col min-w-0 bg-[#F8FAFC] dark:bg-[#0B0F19] overflow-hidden ${
            !isReadingMobile ? 'hidden md:flex' : 'flex'
          }`}
        >
          {detail}
        </main>
      </div>
    </div>
  )
}
