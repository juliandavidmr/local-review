import type { ReactNode } from "react"

type WorkspaceShellProps = {
  title: string
  subtitle: string
  children: ReactNode
}

export function WorkspaceShell({
  title,
  subtitle,
  children,
}: WorkspaceShellProps) {
  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="border-b border-border bg-card">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
          <div>
            <p className="text-xs font-medium uppercase text-muted-foreground">
              Local Review
            </p>
            <h1 className="mt-1 text-xl font-semibold">{title}</h1>
            <p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>
          </div>
          <div className="text-right text-xs text-muted-foreground">
            <p>Mock session</p>
            <p>Local-first desktop skeleton</p>
          </div>
        </div>
      </div>
      <div className="mx-auto max-w-7xl px-6 py-6">{children}</div>
    </main>
  )
}
