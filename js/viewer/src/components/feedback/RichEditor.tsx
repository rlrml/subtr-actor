import { useRef, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import type { Components } from 'react-markdown';
import {
  Bold,
  Italic,
  List,
  ListOrdered,
  Code,
  Quote,
  Heading2,
  Link as LinkIcon,
  HelpCircle,
  X,
} from 'lucide-react';

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
  li: ({ children }) => <li className="text-gray-300">{children}</li>,
  p: ({ children }) => (
    <p className="text-gray-300 leading-relaxed mb-3">{children}</p>
  ),
  strong: ({ children }) => (
    <strong className="font-bold text-white">{children}</strong>
  ),
  em: ({ children }) => <em className="italic text-gray-200">{children}</em>,
  pre: ({ children }) => (
    <pre className="bg-gray-800 rounded-lg p-4 overflow-x-auto my-4">
      {children}
    </pre>
  ),
  hr: () => <hr className="border-gray-700 my-6" />,
};

// Markdown syntax helper content
const markdownHelp = [
  { syntax: '**text**', description: 'Bold' },
  { syntax: '*text*', description: 'Italic' },
  { syntax: '## Title', description: 'Heading' },
  { syntax: '- item', description: 'Bullet list' },
  { syntax: '1. item', description: 'Numbered list' },
  { syntax: '> quote', description: 'Quote' },
  { syntax: '`code`', description: 'Inline code' },
  { syntax: '[text](url)', description: 'Link' },
];

interface RichEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  minHeight?: string;
}

export function RichEditor({
  value,
  onChange,
  placeholder = 'Write your feedback here...',
  minHeight = '300px',
}: RichEditorProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [showHelp, setShowHelp] = useState(false);

  // Insert markdown syntax at cursor position
  const insertSyntax = (before: string, after: string = '', placeholderText: string = '') => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = value.substring(start, end);
    const textToInsert = selectedText || placeholderText;

    const newValue =
      value.substring(0, start) +
      before +
      textToInsert +
      after +
      value.substring(end);

    onChange(newValue);

    // Set cursor position after insertion
    setTimeout(() => {
      textarea.focus();
      if (selectedText) {
        // If text was selected, put cursor after the whole thing
        const newPos = start + before.length + textToInsert.length + after.length;
        textarea.setSelectionRange(newPos, newPos);
      } else {
        // Select the placeholder text so user can type over it
        const selectStart = start + before.length;
        const selectEnd = selectStart + textToInsert.length;
        textarea.setSelectionRange(selectStart, selectEnd);
      }
    }, 0);
  };

  // Toolbar actions
  const handleBold = () => insertSyntax('**', '**', 'bold text');
  const handleItalic = () => insertSyntax('*', '*', 'italic text');
  const handleHeading = () => insertSyntax('## ', '', 'Heading');
  const handleBulletList = () => insertSyntax('- ', '', 'list item');
  const handleNumberedList = () => insertSyntax('1. ', '', 'list item');
  const handleQuote = () => insertSyntax('> ', '', 'quote');
  const handleCode = () => insertSyntax('`', '`', 'code');
  const handleLink = () => {
    const url = window.prompt('Enter URL:');
    if (url) {
      insertSyntax('[', `](${url})`, 'link text');
    }
  };

  const ToolbarButton = ({
    onClick,
    children,
    title,
  }: {
    onClick: () => void;
    children: React.ReactNode;
    title: string;
  }) => (
    <button
      type="button"
      onClick={onClick}
      title={title}
      className="p-2 rounded-lg transition-colors text-gray-400 hover:bg-gray-700 hover:text-white"
    >
      {children}
    </button>
  );

  // Convert single newlines to <br> for preview
  const previewContent = value
    .split('\n\n')
    .map(block => block.replace(/\n/g, '  \n'))
    .join('\n\n');

  return (
    <div className="rounded-xl border border-gray-700 bg-gray-900/50 overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between gap-2 px-3 py-2 border-b border-gray-700 bg-gray-800/50">
        <div className="flex items-center gap-1">
          <ToolbarButton onClick={handleBold} title="Bold (**text**)">
            <Bold className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleItalic} title="Italic (*text*)">
            <Italic className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleCode} title="Code (`code`)">
            <Code className="w-4 h-4" />
          </ToolbarButton>

          <div className="w-px h-6 bg-gray-700 mx-1" />

          <ToolbarButton onClick={handleHeading} title="Heading (## Title)">
            <Heading2 className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleBulletList} title="Bullet list (- item)">
            <List className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleNumberedList} title="Numbered list (1. item)">
            <ListOrdered className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleQuote} title="Quote (> text)">
            <Quote className="w-4 h-4" />
          </ToolbarButton>
          <ToolbarButton onClick={handleLink} title="Link ([text](url))">
            <LinkIcon className="w-4 h-4" />
          </ToolbarButton>
        </div>

        {/* Help toggle */}
        <button
          type="button"
          onClick={() => setShowHelp(!showHelp)}
          className={`
            flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-medium transition-colors
            ${showHelp
              ? 'bg-violet-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
            }
          `}
        >
          <HelpCircle className="w-3.5 h-3.5" />
          Help
        </button>
      </div>

      {/* Help panel */}
      {showHelp && (
        <div className="px-4 py-3 bg-gray-800/70 border-b border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-gray-300">Markdown Syntax</span>
            <button
              type="button"
              onClick={() => setShowHelp(false)}
              className="text-gray-500 hover:text-gray-300"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            {markdownHelp.map((item) => (
              <div
                key={item.syntax}
                className="flex flex-col gap-0.5 px-2 py-1.5 rounded bg-gray-900/50 text-xs"
              >
                <code className="text-violet-400 font-mono">{item.syntax}</code>
                <span className="text-gray-500">{item.description}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Split pane: Editor + Preview */}
      <div className="grid grid-cols-2 gap-0" style={{ minHeight }}>
        {/* Left: Markdown Editor */}
        <div className="flex flex-col border-r border-gray-700">
          <div className="px-3 py-1.5 border-b border-gray-700 bg-gray-800/30">
            <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
              Edit
            </span>
          </div>
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            className="
              flex-1 w-full p-4
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

        {/* Right: Preview */}
        <div className="flex flex-col">
          <div className="px-3 py-1.5 border-b border-gray-700 bg-gray-800/30">
            <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
              Preview
            </span>
          </div>
          <div
            className="flex-1 p-4 overflow-auto"
            style={{ minHeight }}
          >
            {value ? (
              <div className="max-w-none">
                <ReactMarkdown components={markdownComponents}>
                  {previewContent}
                </ReactMarkdown>
              </div>
            ) : (
              <p className="text-gray-500 italic text-sm">{placeholder}</p>
            )}
          </div>
        </div>
      </div>

      {/* Footer with character count */}
      <div className="px-4 py-2 border-t border-gray-700 bg-gray-800/30 flex items-center justify-between">
        <span className="text-xs text-gray-500">
          Use the toolbar or type markdown directly
        </span>
        <span className="text-xs text-gray-500">
          {value.length.toLocaleString()} characters
        </span>
      </div>
    </div>
  );
}
