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
    case 'scc':
      return `use rust_igraph::{Graph, strongly_connected_components};\n\n${graphLine}\n\nlet result = strongly_connected_components(&g).unwrap();\nprintln!("Components: {}", result.count);\nprintln!("Membership: {:?}", result.membership);`;
    case 'bridges':
      return `use rust_igraph::{Graph, bridges};\n\n${graphLine}\n\nlet result = bridges(&g).unwrap();\nprintln!("Bridge count: {}", result.count);\nprintln!("Bridges: {:?}", result.edges);`;
    case 'coloring':
      return `use rust_igraph::{Graph, vertex_coloring};\n\n${graphLine}\n\nlet result = vertex_coloring(&g).unwrap();\nprintln!("Chromatic number: {}", result.chromatic);\nprintln!("Colors: {:?}", result.colors);`;
    case 'topological_sort':
      return `use rust_igraph::{Graph, topological_sort};\n\n${graphLine}\n\nlet order = topological_sort(&g).unwrap();\nprintln!("Topological order: {:?}", order);`;
    case 'transitivity':
      return `use rust_igraph::{Graph, transitivity};\n\n${graphLine}\n\nlet value = transitivity(&g).unwrap();\nprintln!("Transitivity: {:.4}", value);`;
    case 'edge_betweenness_centrality':
      return `use rust_igraph::{Graph, edge_betweenness};\n\n${graphLine}\n\nlet scores = edge_betweenness(&g).unwrap();\nprintln!("Edge betweenness: {:?}", scores);`;
    case 'triad_census':
      return `use rust_igraph::{Graph, triad_census};\n\n${graphLine}\n\nlet tc = triad_census(&g).unwrap();\nprintln!("Triad census: {:?}", tc.counts);`;
    case 'canonical_permutation':
      return `use rust_igraph::{Graph, canonical_permutation};\n\n${graphLine}\n\nlet perm = canonical_permutation(&g, None).unwrap();\nprintln!("Canonical permutation: {:?}", perm);`;
    case 'count_automorphisms':
      return `use rust_igraph::{Graph, count_automorphisms};\n\n${graphLine}\n\nlet count = count_automorphisms(&g, None).unwrap();\nprintln!("|Aut(G)| = {}", count);`;
    case 'coreness':
      return `use rust_igraph::{Graph, coreness};\n\n${graphLine}\n\nlet cores = coreness(&g).unwrap();\nprintln!("K-cores: {:?}", cores);`;
    case 'eccentricity':
      return `use rust_igraph::{Graph, eccentricity};\n\n${graphLine}\n\nlet ecc = eccentricity(&g).unwrap();\nprintln!("Eccentricity: {:?}", ecc);`;
    case 'constraint':
      return `use rust_igraph::{Graph, constraint};\n\n${graphLine}\n\nlet scores = constraint(&g, None).unwrap();\nprintln!("Constraint: {:?}", scores);`;
    case 'diameter':
      return `use rust_igraph::Graph;\n\n${graphLine}\n\nlet d = g.diameter().unwrap();\nprintln!("Diameter: {:?}", d);`;
    case 'shortest_path':
      return `use rust_igraph::Graph;\n\n${graphLine}\n\nlet sp = g.shortest_path_to(0, 1, None).unwrap();\nprintln!("Path: {:?}", sp.vertices);`;
    case 'random_walk':
      return `use rust_igraph::{Graph, random_walk};\n\n${graphLine}\n\nlet (vertices, _edges) = random_walk(&g, None, 0, DijkstraMode::Out, 20, 42).unwrap();\nprintln!("Walk: {:?}", vertices);`;
    case 'isomorphism':
      return `use rust_igraph::{Graph, isomorphic_bliss};\n\nlet g1 = Graph::from_edges(\n    &[(0,1),(1,2),(2,3),(3,0)],\n    false, None\n).unwrap();\nlet g2 = Graph::from_edges(\n    &[(2,0),(0,3),(3,1),(1,2)],\n    false, None\n).unwrap();\n\nlet result = isomorphic_bliss(&g1, &g2, None, None).unwrap();\nprintln!("Isomorphic: {}", result.iso);\nprintln!("Mapping: {:?}", result.map12);`;
    case 'fundamental_cycles':
      return `use rust_igraph::{Graph, fundamental_cycles};\n\n${graphLine}\n\nlet cycles = fundamental_cycles(&g, None, None).unwrap();\nprintln!("Found {} fundamental cycles", cycles.len());\nfor (i, cycle) in cycles.iter().enumerate() {\n    println!("Cycle {}: {:?}", i, cycle);\n}`;
    case 'list_triangles':
      return `use rust_igraph::{Graph, list_triangles};\n\n${graphLine}\n\nlet tris = list_triangles(&g).unwrap();\nprintln!("Found {} triangles", tris.len());\nfor (a, b, c) in &tris {\n    println!("Triangle: {} - {} - {}", a, b, c);\n}`;
    case 'girth':
      return `use rust_igraph::{Graph, girth};\n\n${graphLine}\n\nlet g_val = girth(&g).unwrap();\nprintln!("Girth: {:?}", g_val);`;
    case 'trussness':
      return `use rust_igraph::{Graph, trussness};\n\n${graphLine}\n\nlet t = trussness(&g).unwrap();\nprintln!("Edge trussness: {:?}", t);`;
    case 'automorphism_group':
      return `use rust_igraph::{Graph, automorphism_group};\n\n${graphLine}\n\nlet gens = automorphism_group(&g, None).unwrap();\nprintln!("{} generators found", gens.len());\nfor gen in &gens {\n    println!("{:?}", gen);\n}`;
    case 'clique_number':
      return `use rust_igraph::{Graph, clique_number};\n\n${graphLine}\n\nlet omega = clique_number(&g).unwrap();\nprintln!("Clique number: {}", omega);`;
    case 'independence_number':
      return `use rust_igraph::{Graph, independence_number};\n\n${graphLine}\n\nlet alpha = independence_number(&g).unwrap();\nprintln!("Independence number: {}", alpha);`;
    case 'maximal_cliques':
      return `use rust_igraph::{Graph, maximal_cliques};\n\n${graphLine}\n\nlet cliques = maximal_cliques(&g, 0, 0).unwrap();\nprintln!("Found {} maximal cliques", cliques.len());\nfor c in &cliques {\n    println!("{:?}", c);\n}`;
    case 'vertex_connectivity':
      return `use rust_igraph::{Graph, vertex_connectivity};\n\n${graphLine}\n\nlet kappa = vertex_connectivity(&g).unwrap();\nprintln!("Vertex connectivity: {}", kappa);`;
    case 'edge_connectivity':
      return `use rust_igraph::{Graph, edge_connectivity};\n\n${graphLine}\n\nlet lambda = edge_connectivity(&g).unwrap();\nprintln!("Edge connectivity: {}", lambda);`;
    case 'minimum_spanning_tree':
      return `use rust_igraph::{Graph, minimum_spanning_tree};\n\n${graphLine}\n\nlet mst = minimum_spanning_tree(&g, None).unwrap();\nprintln!("MST edges: {}", mst.ecount());\nprintln!("MST: {:?}", mst);`;
    case 'bellman_ford':
      return `use rust_igraph::{Graph, bellman_ford};\n\n${graphLine}\n\nlet weights = vec![1.0; g.ecount()];\nlet dists = bellman_ford(&g, 0, &weights).unwrap();\nprintln!("Distances from 0: {:?}", dists);`;
    case 'degree_distribution':
      return `use rust_igraph::{Graph, degree_distribution, DegreeMode};\n\n${graphLine}\n\nlet degrees = degree_distribution(&g, DegreeMode::All).unwrap();\nprintln!("Degree per vertex: {:?}", degrees);`;
    case 'feedback_arc_set':
      return `use rust_igraph::{Graph, feedback_arc_set, FasAlgorithm};\n\n${graphLine}\n\nlet fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();\nprintln!("Feedback arc set ({} edges): {:?}", fas.len(), fas);`;
    case 'minimum_cycle_basis':
      return `use rust_igraph::{Graph, minimum_cycle_basis};\n\n${graphLine}\n\nlet cycles = minimum_cycle_basis(&g, None, true).unwrap();\nprintln!("{} cycles in minimum basis", cycles.len());\nfor c in &cycles {\n    println!("  {:?}", c);\n}`;
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
