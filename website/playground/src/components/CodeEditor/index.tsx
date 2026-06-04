import { useRef, useEffect, useState, useCallback } from 'react';
import { EditorState } from '@codemirror/state';
import {
  EditorView,
  lineNumbers,
  highlightActiveLine,
  highlightSpecialChars,
} from '@codemirror/view';
import { bracketMatching, syntaxHighlighting, HighlightStyle } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import { rust } from '@codemirror/lang-rust';
import { oneDark } from '@codemirror/theme-one-dark';
import type { AlgoId, Edge } from '../../types';
import styles from './index.module.css';

interface CodeEditorProps {
  algo: AlgoId;
  edges: Edge[];
  directed: boolean;
  theme: 'dark' | 'light';
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
    case 'closeness':
      return `use rust_igraph::{Graph, closeness};\n\n${graphLine}\n\nlet scores = closeness(&g).unwrap();\nprintln!("Closeness: {:?}", scores);`;
    case 'eigenvector':
      return `use rust_igraph::{Graph, eigenvector_centrality};\n\n${graphLine}\n\nlet scores = eigenvector_centrality(&g).unwrap();\nprintln!("Eigenvector centrality: {:?}", scores);`;
    case 'bfs':
      return `use rust_igraph::{Graph, bfs};\n\n${graphLine}\n\nlet order = bfs(&g, 0).unwrap();\nprintln!("BFS order: {:?}", order);`;
    case 'dfs':
      return `use rust_igraph::{Graph, dfs};\n\n${graphLine}\n\nlet order = dfs(&g, 0).unwrap();\nprintln!("DFS order: {:?}", order);`;
    case 'components':
      return `use rust_igraph::{Graph, connected_components};\n\n${graphLine}\n\nlet result = connected_components(&g).unwrap();\nprintln!("Components: {}", result.count);\nprintln!("Membership: {:?}", result.membership);`;
    case 'infomap':
      return `use rust_igraph::{Graph, infomap};\n\n${graphLine}\n\nlet result = infomap(&g).unwrap();\nprintln!("Codelength: {:.4}", result.codelength);\nprintln!("Communities: {:?}", result.membership);`;
    case 'spinglass':
      return `use rust_igraph::{Graph, spinglass};\n\n${graphLine}\n\nlet result = spinglass(&g).unwrap();\nprintln!("Modularity: {:.4}", result.modularity);\nprintln!("Communities: {:?}", result.membership);`;
    case 'label_propagation':
      return `use rust_igraph::{Graph, label_propagation};\n\n${graphLine}\n\nlet result = label_propagation(&g).unwrap();\nprintln!("Clusters: {}", result.nb_clusters);\nprintln!("Communities: {:?}", result.membership);`;
    case 'walktrap':
      return `use rust_igraph::{Graph, walktrap};\n\n${graphLine}\n\nlet result = walktrap(&g).unwrap();\nprintln!("Clusters: {}", result.nb_clusters);\nprintln!("Communities: {:?}", result.membership);`;
    case 'leiden':
      return `use rust_igraph::{Graph, leiden};\n\n${graphLine}\n\nlet result = leiden(&g).unwrap();\nprintln!("Quality: {:.4}", result.quality);\nprintln!("Communities: {:?}", result.membership);`;
    case 'fast_greedy':
      return `use rust_igraph::{Graph, fast_greedy_modularity};\n\n${graphLine}\n\nlet result = fast_greedy_modularity(&g).unwrap();\nprintln!("Clusters: {}", result.nb_clusters);\nprintln!("Communities: {:?}", result.membership);`;
    case 'leading_eigenvector':
      return `use rust_igraph::{Graph, leading_eigenvector};\n\n${graphLine}\n\nlet result = leading_eigenvector(&g, None, None).unwrap();\nprintln!("Modularity: {:.4}", result.modularity);\nprintln!("Communities: {:?}", result.membership);`;
    case 'edge_betweenness':
      return `use rust_igraph::{Graph, edge_betweenness_community};\n\n${graphLine}\n\nlet result = edge_betweenness_community(&g).unwrap();\nprintln!("Clusters: {}", result.nb_clusters);\nprintln!("Communities: {:?}", result.membership);`;
    case 'fluid':
      return `use rust_igraph::{Graph, fluid_communities};\n\n${graphLine}\n\nlet result = fluid_communities(&g, 3).unwrap();\nprintln!("Clusters: {}", result.nb_clusters);\nprintln!("Communities: {:?}", result.membership);`;
    case 'harmonic':
      return `use rust_igraph::{Graph, harmonic_centrality};\n\n${graphLine}\n\nlet scores = harmonic_centrality(&g).unwrap();\nprintln!("Harmonic centrality: {:?}", scores);`;
    case 'hits':
      return `use rust_igraph::{Graph, hub_and_authority_scores};\n\n${graphLine}\n\nlet result = hub_and_authority_scores(&g).unwrap();\nprintln!("Hub scores: {:?}", result.hub);\nprintln!("Authority scores: {:?}", result.authority);`;
    case 'katz':
      return `use rust_igraph::{Graph, katz_centrality};\n\n${graphLine}\n\nlet scores = katz_centrality(&g, 0.01, 1.0, None, None).unwrap();\nprintln!("Katz centrality: {:?}", scores);`;
    case 'dijkstra':
      return `use rust_igraph::{Graph, dijkstra_distances};\n\n${graphLine}\n\nlet weights = vec![1.0; g.ecount()];\nlet dists = dijkstra_distances(&g, 0, &weights).unwrap();\nprintln!("Distances from 0: {:?}", dists);`;
    case 'graph_stats':
      return `use rust_igraph::{Graph, is_connected, girth, count_triangles, is_bipartite};\n\n${graphLine}\n\nprintln!("Vertices: {}", g.vcount());\nprintln!("Edges: {}", g.ecount());\nprintln!("Diameter: {}", g.diameter().unwrap());\nprintln!("Girth: {}", girth(&g).unwrap());\nprintln!("Triangles: {}", count_triangles(&g).unwrap());`;
    case 'max_flow':
      return `use rust_igraph::{Graph, max_flow_value};\n\n${graphLine}\n\nlet value = max_flow_value(&g, 0, 1, None).unwrap();\nprintln!("Max flow: {:.4}", value);`;
    case 'articulation_points':
      return `use rust_igraph::{Graph, articulation_points};\n\n${graphLine}\n\nlet points = articulation_points(&g).unwrap();\nprintln!("Articulation points: {:?}", points);`;
    case 'degree_sequence':
      return `use rust_igraph::Graph;\n\n${graphLine}\n\nlet degrees = g.degree_sequence().unwrap();\nprintln!("Degree sequence: {:?}", degrees);`;
    default:
      return `use rust_igraph::Graph;\n\n${graphLine}`;
  }
}

const githubLightHighlight = HighlightStyle.define([
  { tag: tags.keyword, color: '#cf222e' },
  { tag: tags.definition(tags.variableName), color: '#24292f' },
  { tag: tags.function(tags.variableName), color: '#8250df' },
  { tag: tags.typeName, color: '#0550ae' },
  { tag: tags.string, color: '#0a3069' },
  { tag: tags.number, color: '#0550ae' },
  { tag: tags.bool, color: '#cf222e' },
  { tag: tags.comment, color: '#6e7781', fontStyle: 'italic' },
  { tag: tags.macroName, color: '#8250df' },
  { tag: tags.operator, color: '#24292f' },
  { tag: tags.propertyName, color: '#0550ae' },
  { tag: tags.punctuation, color: '#24292f' },
  { tag: tags.self, color: '#cf222e' },
  { tag: tags.moduleKeyword, color: '#cf222e' },
  { tag: tags.attributeName, color: '#0550ae' },
]);

const lightTheme = EditorView.theme({
  '&': { backgroundColor: '#f6f8fa', color: '#24292f' },
  '.cm-gutters': {
    backgroundColor: '#f6f8fa',
    borderRight: '1px solid #d0d7de',
    color: '#8b949e',
  },
  '.cm-activeLineGutter': { backgroundColor: '#eaeef2' },
  '.cm-activeLine': { backgroundColor: 'rgba(234, 238, 242, 0.5)' },
  '.cm-matchingBracket': {
    backgroundColor: 'rgba(9, 105, 218, 0.15)',
    outline: '1px solid rgba(9, 105, 218, 0.3)',
  },
  '.cm-selectionMatch': { backgroundColor: 'rgba(9, 105, 218, 0.1)' },
  '.cm-cursor': { borderLeftColor: '#24292f' },
});

export function CodeEditor({ algo, edges, directed, theme }: CodeEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const [copied, setCopied] = useState(false);

  const code = generateRustCode(algo, edges, directed);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [code]);

  useEffect(() => {
    if (!editorRef.current) return;

    const extensions = [
      rust(),
      lineNumbers(),
      highlightActiveLine(),
      highlightSpecialChars(),
      bracketMatching(),
      EditorView.editable.of(false),
      EditorState.readOnly.of(true),
      ...(theme === 'dark'
        ? [oneDark]
        : [lightTheme, syntaxHighlighting(githubLightHighlight)]),
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
    <div className={styles.editorWrap}>
      <button
        className={styles.copyBtn}
        onClick={handleCopy}
        title="Copy to clipboard"
      >
        {copied ? (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
        )}
      </button>
      <div className={styles.editor} ref={editorRef} />
    </div>
  );
}
