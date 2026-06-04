import type { Edge } from './types';

function downloadFile(filename: string, content: string, mime: string): void {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.download = filename;
  link.href = url;
  link.click();
  URL.revokeObjectURL(url);
}

export function exportGml(edges: Edge[], vcount: number, directed: boolean): void {
  let gml = 'graph [\n';
  gml += `  directed ${directed ? 1 : 0}\n`;
  for (let i = 0; i < vcount; i++) {
    gml += `  node [\n    id ${i}\n    label "${i}"\n  ]\n`;
  }
  for (const [u, v] of edges) {
    gml += `  edge [\n    source ${u}\n    target ${v}\n  ]\n`;
  }
  gml += ']\n';
  downloadFile('graph.gml', gml, 'text/plain');
}

export function exportDot(edges: Edge[], vcount: number, directed: boolean): void {
  const keyword = directed ? 'digraph' : 'graph';
  const arrow = directed ? ' -> ' : ' -- ';
  let dot = `${keyword} G {\n`;
  for (let i = 0; i < vcount; i++) {
    dot += `  ${i};\n`;
  }
  for (const [u, v] of edges) {
    dot += `  ${u}${arrow}${v};\n`;
  }
  dot += '}\n';
  downloadFile('graph.dot', dot, 'text/plain');
}

export function exportGraphml(edges: Edge[], vcount: number, directed: boolean): void {
  let xml = '<?xml version="1.0" encoding="UTF-8"?>\n';
  xml += '<graphml xmlns="http://graphml.graphstruct.org/graphml">\n';
  xml += `  <graph id="G" edgedefault="${directed ? 'directed' : 'undirected'}">\n`;
  for (let i = 0; i < vcount; i++) {
    xml += `    <node id="n${i}"/>\n`;
  }
  for (let i = 0; i < edges.length; i++) {
    xml += `    <edge id="e${i}" source="n${edges[i]![0]}" target="n${edges[i]![1]}"/>\n`;
  }
  xml += '  </graph>\n';
  xml += '</graphml>\n';
  downloadFile('graph.graphml', xml, 'application/xml');
}

export function exportEdgeList(edges: Edge[], _vcount: number, _directed: boolean): void {
  const lines = edges.map(([u, v]) => `${u} ${v}`).join('\n');
  downloadFile('graph.txt', lines + '\n', 'text/plain');
}

export type ExportFormat = 'png' | 'gml' | 'dot' | 'graphml' | 'edgelist';
