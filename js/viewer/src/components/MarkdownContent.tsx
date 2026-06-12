import ReactMarkdown from 'react-markdown';
import type { Components } from 'react-markdown';

/**
 * Shared markdown renderer used for long-form content (announcements,
 * editorial pages, etc.). Styled to match the gaming/dark aesthetic with
 * violet accents.
 *
 * NOTE: Do not pass untrusted HTML in `content`; we don't enable
 * `rehype-raw`, so any inline HTML is rendered as plain text.
 */
const markdownComponents: Components = {
  a: ({ children, href }) => (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="text-violet-400 hover:text-violet-300 underline"
    >
      {children}
    </a>
  ),
  code: ({ children, className }) => {
    const isInline = !className;
    return isInline ? (
      <code className="px-1.5 py-0.5 rounded bg-gray-800 text-violet-300 text-sm">
        {children}
      </code>
    ) : (
      <code className={className}>{children}</code>
    );
  },
  blockquote: ({ children }) => (
    <blockquote className="border-l-4 border-violet-500/50 pl-4 italic text-gray-400 my-4">
      {children}
    </blockquote>
  ),
  h1: ({ children }) => (
    <h1 className="text-3xl font-bold text-white mt-8 mb-4">{children}</h1>
  ),
  h2: ({ children }) => (
    <h2 className="text-2xl font-bold text-white mt-6 mb-3">{children}</h2>
  ),
  h3: ({ children }) => (
    <h3 className="text-xl font-bold text-white mt-5 mb-2">{children}</h3>
  ),
  h4: ({ children }) => (
    <h4 className="text-lg font-bold text-white mt-4 mb-2">{children}</h4>
  ),
  ul: ({ children }) => (
    <ul className="list-disc list-inside space-y-1 text-gray-300 my-3">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-decimal list-inside space-y-1 text-gray-300 my-3">{children}</ol>
  ),
  li: ({ children }) => <li className="text-gray-300">{children}</li>,
  p: ({ children }) => (
    <p className="text-gray-300 leading-relaxed mb-4">{children}</p>
  ),
  strong: ({ children }) => (
    <strong className="font-bold text-white">{children}</strong>
  ),
  em: ({ children }) => <em className="italic text-gray-200">{children}</em>,
  pre: ({ children }) => (
    <pre className="bg-gray-900/80 border border-gray-800 rounded-lg p-4 overflow-x-auto my-4 text-sm">
      {children}
    </pre>
  ),
  hr: () => <hr className="border-gray-800 my-6" />,
  img: ({ src, alt }) => (
    <img
      src={src}
      alt={alt ?? ''}
      className="rounded-lg border border-gray-800 my-4 max-w-full h-auto"
      loading="lazy"
    />
  ),
};

interface MarkdownContentProps {
  content: string;
  className?: string;
}

export function MarkdownContent({ content, className = '' }: MarkdownContentProps) {
  return (
    <div className={`max-w-none ${className}`}>
      <ReactMarkdown components={markdownComponents}>{content}</ReactMarkdown>
    </div>
  );
}

export default MarkdownContent;
