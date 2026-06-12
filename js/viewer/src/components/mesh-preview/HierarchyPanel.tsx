import { useState } from 'react';
import { MeshPreviewEngine, MeshNode, MaterialInfo } from './MeshPreviewEngine';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ChevronRight, ChevronDown, Box, Folder, Circle } from 'lucide-react';
import { cn } from '@/lib/utils';

interface HierarchyPanelProps {
  hierarchy: MeshNode[];
  materials: MaterialInfo[];
  engine: MeshPreviewEngine | null;
}

interface TreeNodeProps {
  node: MeshNode;
  hierarchy: MeshNode[];
  engine: MeshPreviewEngine | null;
  level: number;
}

function TreeNode({ node, hierarchy, engine, level }: TreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(level < 2);
  const children = hierarchy.filter((n) => n.parentId === node.id);
  const hasChildren = children.length > 0;

  const handleClick = () => {
    if (engine && node.object) {
      engine.focusOnObject(node.object);
    }
  };

  const getIcon = () => {
    switch (node.type) {
      case 'mesh':
        return <Box className="h-3 w-3 text-violet-400" />;
      case 'group':
        return <Folder className="h-3 w-3 text-yellow-400" />;
      case 'bone':
        return <Circle className="h-3 w-3 text-green-400" />;
      default:
        return <Circle className="h-3 w-3 text-zinc-500" />;
    }
  };

  return (
    <div>
      <div
        className="flex items-center gap-1 py-0.5 hover:bg-zinc-800 rounded cursor-pointer group"
        style={{ paddingLeft: `${level * 12}px` }}
      >
        {hasChildren ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsExpanded(!isExpanded);
            }}
            className="p-0.5"
          >
            {isExpanded ? (
              <ChevronDown className="h-3 w-3 text-zinc-500" />
            ) : (
              <ChevronRight className="h-3 w-3 text-zinc-500" />
            )}
          </button>
        ) : (
          <span className="w-4" />
        )}
        <button
          onClick={handleClick}
          className="flex items-center gap-1.5 flex-1 text-left text-sm text-zinc-300 group-hover:text-white"
        >
          {getIcon()}
          <span className="truncate">{node.name}</span>
        </button>
      </div>
      {isExpanded &&
        children.map((child) => (
          <TreeNode
            key={child.id}
            node={child}
            hierarchy={hierarchy}
            engine={engine}
            level={level + 1}
          />
        ))}
    </div>
  );
}

export function HierarchyPanel({ hierarchy, materials, engine }: HierarchyPanelProps) {
  const rootNodes = hierarchy.filter((n) => n.parentId === null);

  return (
    <Tabs defaultValue="hierarchy" className="w-full">
      <TabsList className="w-full grid grid-cols-2">
        <TabsTrigger value="hierarchy">Hierarchy</TabsTrigger>
        <TabsTrigger value="materials">Materials</TabsTrigger>
      </TabsList>

      <TabsContent value="hierarchy" className="mt-4">
        {rootNodes.length === 0 ? (
          <div className="text-center py-8 text-zinc-500">
            <p>No mesh loaded</p>
            <p className="text-sm mt-1">Upload a mesh to see its hierarchy</p>
          </div>
        ) : (
          <div className="max-h-96 overflow-y-auto">
            {rootNodes.map((node) => (
              <TreeNode
                key={node.id}
                node={node}
                hierarchy={hierarchy}
                engine={engine}
                level={0}
              />
            ))}
          </div>
        )}
      </TabsContent>

      <TabsContent value="materials" className="mt-4">
        {materials.length === 0 ? (
          <div className="text-center py-8 text-zinc-500">
            <p>No materials found</p>
          </div>
        ) : (
          <div className="space-y-1">
            {materials.map((mat) => (
              <div
                key={mat.id}
                className={cn(
                  'flex items-center gap-2 px-2 py-1.5 rounded text-sm',
                  'hover:bg-zinc-800 text-zinc-300'
                )}
              >
                <div
                  className="w-4 h-4 rounded border border-zinc-600"
                  style={{ backgroundColor: mat.original.color }}
                />
                <span className="truncate flex-1">{mat.name}</span>
                <span className="text-xs text-zinc-500">{mat.meshName}</span>
              </div>
            ))}
          </div>
        )}
      </TabsContent>
    </Tabs>
  );
}
