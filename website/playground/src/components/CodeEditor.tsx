import { useRef, useEffect } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView, lineNumbers, highlightActiveLine } from '@codemirror/view';
import { rust } from '@codemirror/lang-rust';
import { oneDark } from '@codemirror/theme-one-dark';
import type { AlgoId, Edge } from '../types';

interface CodeEditorProps {
  algo: AlgoId;
  edges: Edge[];
  directed: boolean;
  theme: 'dark' | 'light';
  t: (key: string) => string;
}

function generateRustCode(algo: AlgoId, edges: Edge[], directed: boolean): string {
  const edgeStr = edges
    .slice(0, 10)
    .map(([u, v]) => `(${u},${v})`)
    .join(', ');
  const suffix = edges.length > 10 ? ', ...' : '';
  const dirStr = directed ? 'true' : 'false';

  const graphLine = `let g = Graph::from_edges(\n    &[${edgeStr}${suffix}],\n    ${dirStr}, None\n).unwrap();`;

  switch (algo) {
    case 'pagerank':
      return `use rust_igraph::{Graph, pagerank};\n\n${graphLine}\n\nlet pr = pagerank(&g).unwrap();\nprintln!("PageRank: {:?}", pr.scores);`;
    case 'louvain':
      return `use rust_igraph::{Graph, louvain};\n\n${graphLine}\n\nlet result = louvain(&g).unwrap();\nprintln!("Modularity: {:.4}", result.modularity);\nprintln!("Communities: {:?}", result.membership);`;
    case 'betweenness':
      return `use rust_igraph::{Graph, betweenness};\n\n${graphLine}\n\nlet scores = betweenness(&g).unwrap();\nprintln!("Betweenness: {:?}", scores);`;
    case 'bfs':
      return `use rust_igraph::{Graph, bfs};\n\n${graphLine}\n\nlet order = bfs(&g, 0).unwrap();\nprintln!("BFS order: {:?}", order);`;
    case 'components':
      return `use rust_igraph::{Graph, connected_components};\n\n${graphLine}\n\nlet result = connected_components(&g).unwrap();\nprintln!("Components: {}", result.count);\nprintln!("Membership: {:?}", result.membership);`;
    case 'infomap':
      return `use rust_igraph::{Graph, infomap};\n\n${graphLine}\n\nlet result = infomap(&g).unwrap();\nprintln!("Codelength: {:.4}", result.codelength);\nprintln!("Communities: {:?}", result.membership);`;
    case 'spinglass':
      return `use rust_igraph::{Graph, spinglass};\n\n${graphLine}\n\nlet result = spinglass(&g).unwrap();\nprintln!("Modularity: {:.4}", result.modularity);\nprintln!("Communities: {:?}", result.membership);`;
    default:
      return `use rust_igraph::Graph;\n\n${graphLine}`;
  }
}

const lightTheme = EditorView.theme({
  '&': { backgroundColor: '#f6f8fa', color: '#24292f' },
  '.cm-gutters': { backgroundColor: '#f6f8fa', borderRight: '1px solid #d0d7de' },
  '.cm-activeLineGutter': { backgroundColor: '#eaeef2' },
  '.cm-activeLine': { backgroundColor: '#eaeef2' },
});

export function CodeEditor({ algo, edges, directed, theme, t }: CodeEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  const code = generateRustCode(algo, edges, directed);

  useEffect(() => {
    if (!editorRef.current) return;

    const extensions = [
      rust(),
      lineNumbers(),
      highlightActiveLine(),
      EditorView.editable.of(false),
      EditorState.readOnly.of(true),
      ...(theme === 'dark' ? [oneDark] : [lightTheme]),
    ];

    if (viewRef.current) {
      viewRef.current.destroy();
    }

    const state = EditorState.create({
      doc: code,
      extensions,
    });

    viewRef.current = new EditorView({
      state,
      parent: editorRef.current,
    });

    return () => {
      viewRef.current?.destroy();
      viewRef.current = null;
    };
  }, [code, theme]);

  return (
    <>
      <div className="code-panel-header">
        <h3>{t('code')}</h3>
      </div>
      <div className="code-editor" ref={editorRef} />
    </>
  );
}
