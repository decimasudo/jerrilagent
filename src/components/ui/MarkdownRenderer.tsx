// Simple markdown renderer — no ESM-only dependencies.
// Handles the AI output subset: ### headings, **bold**, - bullet lists, paragraphs.

interface Props {
  children: string
  className?: string
}

function parseLine(line: string, key: number) {
  // Replace **bold** spans inside a line
  const parts = line.split(/(\*\*[^*]+\*\*)/)
  return parts.map((part, i) =>
    part.startsWith('**') && part.endsWith('**')
      ? <strong key={i}>{part.slice(2, -2)}</strong>
      : part
  )
}

export function MarkdownRenderer({ children, className }: Props) {
  const lines = children.split('\n')

  const elements: React.ReactNode[] = []
  let listBuffer: string[] = []

  const flushList = (key: string) => {
    if (listBuffer.length === 0) return
    elements.push(
      <ul key={key} className="mb-8 list-disc pl-6 space-y-1">
        {listBuffer.map((item, i) => (
          <li key={i}>{parseLine(item, i)}</li>
        ))}
      </ul>
    )
    listBuffer = []
  }

  lines.forEach((line, idx) => {
    const trimmed = line.trim()

    if (trimmed.startsWith('### ')) {
      flushList(`list-before-${idx}`)
      elements.push(
        <h3 key={idx} className="text-lg font-black mt-10 mb-4 uppercase text-zinc-900 first:mt-0">
          {parseLine(trimmed.slice(4), 0)}
        </h3>
      )
    } else if (trimmed.startsWith('## ')) {
      flushList(`list-before-${idx}`)
      elements.push(
        <h2 key={idx} className="text-xl font-black mt-10 mb-4 text-zinc-900 first:mt-0">
          {parseLine(trimmed.slice(3), 0)}
        </h2>
      )
    } else if (/^[-*] /.test(trimmed)) {
      listBuffer.push(trimmed.slice(2))
    } else if (trimmed === '') {
      flushList(`list-${idx}`)
    } else {
      flushList(`list-before-${idx}`)
      elements.push(
        <p key={idx} className="mb-6 leading-relaxed">
          {parseLine(trimmed, 0)}
        </p>
      )
    }
  })

  flushList('list-end')

  return <div className={className}>{elements}</div>
}
