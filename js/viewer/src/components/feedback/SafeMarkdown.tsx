import ReactMarkdown from 'react-markdown';
import DOMPurify from 'isomorphic-dompurify';

interface SafeMarkdownProps {
  content: string;
  className?: string;
}

export function SafeMarkdown({ content, className = '' }: SafeMarkdownProps) {
  // Sanitize HTML that might be in the markdown
  const sanitizedContent = DOMPurify.sanitize(content, {
    ALLOWED_TAGS: [], // Remove all HTML tags, only keep plain text
    ALLOWED_ATTR: [],
  });

  return (
    <div className={`prose prose-invert prose-sm max-w-none ${className}`}>
      <ReactMarkdown
        components={{
          // Style links
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
          // Style code blocks
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
          // Style block quotes
          blockquote: ({ children }) => (
            <blockquote className="border-l-4 border-violet-500/50 pl-4 italic text-gray-400">
              {children}
            </blockquote>
          ),
          // Style headings
          h1: ({ children }) => (
            <h1 className="text-2xl font-bold text-white mt-6 mb-4">{children}</h1>
          ),
          h2: ({ children }) => (
            <h2 className="text-xl font-bold text-white mt-5 mb-3">{children}</h2>
          ),
          h3: ({ children }) => (
            <h3 className="text-lg font-bold text-white mt-4 mb-2">{children}</h3>
          ),
          // Style lists
          ul: ({ children }) => (
            <ul className="list-disc list-inside space-y-1 text-gray-300">{children}</ul>
          ),
          ol: ({ children }) => (
            <ol className="list-decimal list-inside space-y-1 text-gray-300">{children}</ol>
          ),
          // Style paragraphs
          p: ({ children }) => (
            <p className="text-gray-300 leading-relaxed mb-4">{children}</p>
          ),
        }}
      >
        {sanitizedContent}
      </ReactMarkdown>
    </div>
  );
}
