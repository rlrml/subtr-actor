import { useCallback, useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import type { Components } from 'react-markdown';

// Custom components for proper markdown styling
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
    <h1 className="text-2xl font-bold text-white mt-6 mb-4">{children}</h1>
  ),
  h2: ({ children }) => (
    <h2 className="text-xl font-bold text-white mt-5 mb-3">{children}</h2>
  ),
  h3: ({ children }) => (
    <h3 className="text-lg font-bold text-white mt-4 mb-2">{children}</h3>
  ),
  h4: ({ children }) => (
    <h4 className="text-base font-bold text-white mt-3 mb-2">{children}</h4>
  ),
  ul: ({ children }) => (
    <ul className="list-disc list-inside space-y-1 text-gray-300 my-2">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-decimal list-inside space-y-1 text-gray-300 my-2">{children}</ol>
  ),
  li: ({ children }) => (
    <li className="text-gray-300">{children}</li>
  ),
  p: ({ children }) => (
    <p className="text-gray-300 leading-relaxed mb-3">{children}</p>
  ),
  strong: ({ children }) => (
    <strong className="font-bold text-white">{children}</strong>
  ),
  em: ({ children }) => (
    <em className="italic text-gray-200">{children}</em>
  ),
  pre: ({ children }) => (
    <pre className="bg-gray-800 rounded-lg p-4 overflow-x-auto my-4">{children}</pre>
  ),
  hr: () => (
    <hr className="border-gray-700 my-6" />
  ),
};

interface MarkdownSplitPaneProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  minHeight?: string;
}

export function MarkdownSplitPane({
  value,
  onChange,
  placeholder = 'Write markdown here...',
  minHeight = '200px',
}: MarkdownSplitPaneProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);

  // Synchronized scrolling
  const handleTextareaScroll = useCallback(() => {
    if (!textareaRef.current || !previewRef.current) return;

    const textarea = textareaRef.current;
    const preview = previewRef.current;

    const scrollPercentage =
      textarea.scrollTop / (textarea.scrollHeight - textarea.clientHeight);

    preview.scrollTop =
      scrollPercentage * (preview.scrollHeight - preview.clientHeight);
  }, []);

  // Handle Tab key for indentation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Tab') {
        e.preventDefault();
        const textarea = e.currentTarget;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;

        const newValue =
          value.substring(0, start) + '  ' + value.substring(end);

        onChange(newValue);

        // Restore cursor position
        requestAnimationFrame(() => {
          textarea.selectionStart = textarea.selectionEnd = start + 2;
        });
      }
    },
    [value, onChange]
  );

  // Auto-resize textarea to content (optional, for better UX)
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.max(
        textareaRef.current.scrollHeight,
        parseInt(minHeight)
      )}px`;
    }
  }, [value, minHeight]);

  return (
    <div
      className="grid grid-cols-2 gap-0 h-full"
      style={{ minHeight }}
    >
      {/* Left: Raw Markdown Editor */}
      <div className="flex flex-col border-r border-gray-700">
        <div className="px-3 py-2 border-b border-gray-700 bg-gray-800/50">
          <span className="text-xs font-medium text-gray-400 uppercase tracking-wide">
            Markdown
          </span>
        </div>
        <div className="flex-1 relative">
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleTextareaScroll}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            spellCheck={false}
            className="
              w-full h-full p-4
              bg-transparent
              font-mono text-sm text-gray-200
              placeholder:text-gray-500
              resize-none
              outline-none
              overflow-auto
            "
            style={{ minHeight }}
          />
        </div>
      </div>

      {/* Right: Preview */}
      <div className="flex flex-col">
        <div className="px-3 py-2 border-b border-gray-700 bg-gray-800/50">
          <span className="text-xs font-medium text-gray-400 uppercase tracking-wide">
            Preview
          </span>
        </div>
        <div
          ref={previewRef}
          className="flex-1 p-4 overflow-auto"
          style={{ minHeight }}
        >
          {value ? (
            <div className="max-w-none">
              <ReactMarkdown components={markdownComponents}>{value}</ReactMarkdown>
            </div>
          ) : (
            <p className="text-gray-500 italic text-sm">{placeholder}</p>
          )}
        </div>
      </div>
    </div>
  );
}
