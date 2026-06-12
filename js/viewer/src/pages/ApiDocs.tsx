import { useState } from 'react';
import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Highlight, type PrismTheme } from 'prism-react-renderer';
import {
  Book,
  Upload,
  Key,
  Copy,
  Check,
  Terminal,
} from 'lucide-react';
import { AuthCard } from '@/components/ui/GradientCard';
import { toast } from 'sonner';

// Custom theme matching site colors (violet/fuchsia accent)
const customTheme: PrismTheme = {
  plain: {
    color: '#d1d5db', // gray-300
    backgroundColor: 'rgb(10, 10, 15)',
  },
  styles: [
    {
      types: ['comment', 'prolog', 'doctype', 'cdata'],
      style: { color: '#6b7280', fontStyle: 'italic' }, // gray-500
    },
    {
      types: ['punctuation'],
      style: { color: '#9ca3af' }, // gray-400
    },
    {
      types: ['property', 'tag', 'constant', 'symbol', 'deleted'],
      style: { color: '#f472b6' }, // pink-400
    },
    {
      types: ['boolean', 'number'],
      style: { color: '#c084fc' }, // purple-400
    },
    {
      types: ['string', 'char', 'attr-value', 'inserted'],
      style: { color: '#4ade80' }, // green-400
    },
    {
      types: ['operator', 'entity', 'url', 'variable'],
      style: { color: '#67e8f9' }, // cyan-300
    },
    {
      types: ['function'],
      style: { color: '#a78bfa' }, // violet-400
    },
    {
      types: ['keyword', 'atrule'],
      style: { color: '#e879f9' }, // fuchsia-400
    },
    {
      types: ['attr-name', 'class-name'],
      style: { color: '#fbbf24' }, // amber-400
    },
    {
      types: ['selector', 'regex', 'important'],
      style: { color: '#fb923c' }, // orange-400
    },
  ],
};

interface CodeBlockProps {
  code: string;
  language?: string;
}

function CodeBlock({ code, language = 'bash' }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      toast.success('Copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('Failed to copy');
    }
  };

  return (
    <div className="relative group">
      <Highlight theme={customTheme} code={code.trim()} language={language}>
        {({ style, tokens, getLineProps, getTokenProps }) => (
          <pre
            className="rounded-lg p-4 overflow-x-auto text-sm border border-gray-800"
            style={style}
          >
            {tokens.map((line, i) => (
              <div key={i} {...getLineProps({ line })}>
                {line.map((token, key) => (
                  <span key={key} {...getTokenProps({ token })} />
                ))}
              </div>
            ))}
          </pre>
        )}
      </Highlight>
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-2 rounded-lg bg-gray-800/80 text-gray-400 hover:text-white hover:bg-gray-700 transition-colors opacity-0 group-hover:opacity-100"
        title="Copy to clipboard"
      >
        {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
      </button>
    </div>
  );
}

export default function ApiDocs() {
  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <SEOHead
        title="API Documentation"
        description="Upload Rocket League replays programmatically using BallCam's REST API with Personal Access Tokens. Complete documentation with code examples."
      />

      {/* Header */}
      <div className="flex items-center gap-3 mb-8">
        <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-violet-500 to-fuchsia-500 flex items-center justify-center">
          <Book className="w-6 h-6 text-white" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-white">API Documentation</h1>
          <p className="text-gray-400 text-sm">
            Upload replays programmatically using Personal Access Tokens
          </p>
        </div>
      </div>

      {/* Quick Links */}
      <div className="flex flex-wrap gap-3 mb-8">
        <Link
          to="/settings/tokens"
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-violet-500/20 text-violet-400 hover:bg-violet-500/30 transition-colors"
        >
          <Key className="w-4 h-4" />
          Manage Tokens
        </Link>
      </div>

      {/* Authentication Section */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Key className="w-5 h-5 text-violet-400" />
          Authentication
        </h2>

        <p className="text-gray-400 mb-4">
          All API requests require authentication using a Personal Access Token (PAT).
          Include your token in the <code className="text-violet-400 bg-violet-500/10 px-1.5 py-0.5 rounded">Authorization</code> header:
        </p>

        <CodeBlock code="Authorization: Bearer pat_your_token_here" />

        <div className="mt-4 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
          <p className="text-amber-400 text-sm">
            <strong>Security:</strong> Never share your tokens or commit them to version control.
            Tokens have the same permissions as your account.
          </p>
        </div>
      </AuthCard>

      {/* Endpoints Section */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-6 flex items-center gap-2">
          <Terminal className="w-5 h-5 text-violet-400" />
          Endpoints
        </h2>

        {/* Upload Replay */}
        <div className="border border-gray-800 rounded-lg overflow-hidden">
          <div className="bg-gray-900/50 px-4 py-3 flex items-center gap-3 border-b border-gray-800">
            <span className="px-2 py-1 rounded text-xs font-semibold bg-green-500/20 text-green-400">
              POST
            </span>
            <code className="text-white font-mono">/api/replays</code>
            <span className="ml-auto text-gray-500 text-sm flex items-center gap-1">
              <Upload className="w-4 h-4" />
              Upload Replay
            </span>
          </div>

          <div className="p-4 space-y-4">
            <p className="text-gray-400">
              Upload a Rocket League replay file (.replay) to be processed and made available for viewing.
            </p>

            {/* Request Format */}
            <div>
              <h4 className="text-sm font-medium text-gray-300 mb-2">Request</h4>
              <p className="text-gray-500 text-sm mb-2">
                Content-Type: <code className="text-gray-400">multipart/form-data</code>
              </p>

              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-800">
                      <th className="text-left py-2 text-gray-400 font-medium">Field</th>
                      <th className="text-left py-2 text-gray-400 font-medium">Type</th>
                      <th className="text-left py-2 text-gray-400 font-medium">Required</th>
                      <th className="text-left py-2 text-gray-400 font-medium">Description</th>
                    </tr>
                  </thead>
                  <tbody className="text-gray-300">
                    <tr className="border-b border-gray-800/50">
                      <td className="py-2 font-mono text-violet-400">file</td>
                      <td className="py-2">File</td>
                      <td className="py-2">
                        <span className="text-green-400">Yes</span>
                      </td>
                      <td className="py-2 text-gray-500">The .replay file (max 50MB)</td>
                    </tr>
                    <tr className="border-b border-gray-800/50">
                      <td className="py-2 font-mono text-violet-400">title</td>
                      <td className="py-2">String</td>
                      <td className="py-2">
                        <span className="text-gray-500">No</span>
                      </td>
                      <td className="py-2 text-gray-500">Custom title (max 100 chars). Auto-generated if not provided.</td>
                    </tr>
                    <tr>
                      <td className="py-2 font-mono text-violet-400">visibility</td>
                      <td className="py-2">String</td>
                      <td className="py-2">
                        <span className="text-gray-500">No</span>
                      </td>
                      <td className="py-2 text-gray-500">
                        <code className="text-gray-400">"public"</code> (default) or <code className="text-gray-400">"unlisted"</code>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            {/* Example */}
            <div>
              <h4 className="text-sm font-medium text-gray-300 mb-2">Example (cURL)</h4>
              <CodeBlock
                code={`curl -X POST https://api.ballcam.tv/replays \\
  -H "Authorization: Bearer pat_your_token_here" \\
  -F "file=@match.replay" \\
  -F "title=Grand Final - Team A vs Team B" \\
  -F "visibility=public"`}
              />
            </div>

            {/* Response */}
            <div>
              <h4 className="text-sm font-medium text-gray-300 mb-2">Response (201 Created)</h4>
              <CodeBlock
                language="json"
                code={`{
  "success": true,
  "replay": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Grand Final - Team A vs Team B",
    "status": "processing",
    "visibility": "public",
    "mapName": "DFH Stadium",
    "team0Score": 3,
    "team1Score": 2,
    "durationSeconds": 420,
    "createdAt": "2026-01-03T12:00:00.000Z",
    "players": [...]
  }
}`}
              />
            </div>

            {/* Processing Status */}
            <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
              <p className="text-blue-400 text-sm">
                <strong>Note:</strong> The replay is processed asynchronously. Initial status will be{' '}
                <code className="bg-blue-500/20 px-1 rounded">"processing"</code>.
                Poll <code className="bg-blue-500/20 px-1 rounded">GET /api/replays/:id</code> to check when it becomes{' '}
                <code className="bg-blue-500/20 px-1 rounded">"ready"</code>.
              </p>
            </div>
          </div>
        </div>
      </AuthCard>

      {/* Error Codes */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4">Error Responses</h2>

        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="text-left py-2 text-gray-400 font-medium">Status</th>
                <th className="text-left py-2 text-gray-400 font-medium">Description</th>
              </tr>
            </thead>
            <tbody className="text-gray-300">
              <tr className="border-b border-gray-800/50">
                <td className="py-2">
                  <span className="px-2 py-1 rounded text-xs bg-red-500/20 text-red-400">400</span>
                </td>
                <td className="py-2 text-gray-500">Bad request (invalid file, missing fields, etc.)</td>
              </tr>
              <tr className="border-b border-gray-800/50">
                <td className="py-2">
                  <span className="px-2 py-1 rounded text-xs bg-red-500/20 text-red-400">401</span>
                </td>
                <td className="py-2 text-gray-500">Invalid or missing authentication token</td>
              </tr>
              <tr className="border-b border-gray-800/50">
                <td className="py-2">
                  <span className="px-2 py-1 rounded text-xs bg-red-500/20 text-red-400">403</span>
                </td>
                <td className="py-2 text-gray-500">Email verification required</td>
              </tr>
              <tr>
                <td className="py-2">
                  <span className="px-2 py-1 rounded text-xs bg-red-500/20 text-red-400">500</span>
                </td>
                <td className="py-2 text-gray-500">Server error during upload processing</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div className="mt-4">
          <h4 className="text-sm font-medium text-gray-300 mb-2">Error Response Format</h4>
          <CodeBlock
            language="json"
            code={`{
  "error": "Bad Request",
  "message": "File must be a .replay file"
}`}
          />
        </div>
      </AuthCard>

      {/* Code Examples */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4">Code Examples</h2>

        {/* JavaScript/Node.js */}
        <div className="mb-6">
          <h3 className="text-lg font-medium text-gray-300 mb-3 flex items-center gap-2">
            <span className="w-6 h-6 rounded bg-yellow-500/20 flex items-center justify-center text-xs text-yellow-400">JS</span>
            JavaScript / Node.js
          </h3>
          <CodeBlock
            language="javascript"
            code={`const fs = require('fs');
const FormData = require('form-data');

async function uploadReplay(filePath, token, options = {}) {
  const form = new FormData();
  form.append('file', fs.createReadStream(filePath));

  if (options.title) form.append('title', options.title);
  if (options.visibility) form.append('visibility', options.visibility);

  const response = await fetch('https://api.ballcam.tv/replays', {
    method: 'POST',
    headers: {
      'Authorization': \`Bearer \${token}\`,
    },
    body: form,
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  return response.json();
}

// Usage
const result = await uploadReplay('./match.replay', 'pat_xxx', {
  title: 'Tournament Final',
  visibility: 'public',
});
console.log('Uploaded:', result.replay.id);`}
          />
        </div>

        {/* Python */}
        <div>
          <h3 className="text-lg font-medium text-gray-300 mb-3 flex items-center gap-2">
            <span className="w-6 h-6 rounded bg-blue-500/20 flex items-center justify-center text-xs text-blue-400">PY</span>
            Python
          </h3>
          <CodeBlock
            language="python"
            code={`import requests

def upload_replay(file_path, token, title=None, visibility='public'):
    url = 'https://api.ballcam.tv/replays'
    headers = {'Authorization': f'Bearer {token}'}

    with open(file_path, 'rb') as f:
        files = {'file': f}
        data = {'visibility': visibility}
        if title:
            data['title'] = title

        response = requests.post(url, headers=headers, files=files, data=data)

    response.raise_for_status()
    return response.json()

# Usage
result = upload_replay(
    './match.replay',
    'pat_xxx',
    title='Tournament Final',
    visibility='public'
)
print(f"Uploaded: {result['replay']['id']}")`}
          />
        </div>
      </AuthCard>

      {/* Rate Limits */}
      <AuthCard>
        <h2 className="text-xl font-semibold text-white mb-4">Rate Limits</h2>
        <p className="text-gray-400 mb-4">
          API requests are subject to rate limiting to ensure fair usage:
        </p>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>Upload endpoint: <span className="text-white">10 requests per minute</span> per token</li>
          <li>File size limit: <span className="text-white">50 MB</span> per replay</li>
        </ul>
      </AuthCard>
    </div>
  );
}
