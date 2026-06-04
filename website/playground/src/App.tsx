import { useState, useCallback, useEffect, useRef } from 'react';
import { Header } from './components/Header';
import { GraphEditor } from './components/GraphEditor';
import { AlgoPanel } from './components/AlgoPanel';
import { Canvas } from './components/Canvas';
import { CodeEditor } from './components/CodeEditor';
import { Resizer } from './components/Resizer';
import { useTheme } from './hooks/useTheme';
import { useI18n } from './hooks/useI18n';
import { useWasm } from './hooks/useWasm';
import { useResizablePanels } from './hooks/useResizablePanels';
import { PRESETS } from './presets';
import type { AlgoId, AlgoParams, AlgoResult, Edge, AlgoResultScores, AlgoResultMembership, AlgoResultOrder, RunResult } from './types';
import './App.css';

function edgesFromPreset(id: string): string {
  const preset = PRESETS[id];
  if (!preset) return '';
  return preset.edges.map(([u, v]) => `${u} ${v}`).join('\n');
}

function parseEdges(text: string): Edge[] {
  const edges: Edge[] = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('//')) continue;
    const parts = trimmed.split(/[\s,;]+/).map(Number);
    if (parts.length >= 2 && Number.isFinite(parts[0]) && Number.isFinite(parts[1])) {
      edges.push([parts[0]!, parts[1]!]);
    }
  }
  return edges;
}

function getVcount(edges: Edge[]): number {
  let max = -1;
  for (const [u, v] of edges) {
    if (u > max) max = u;
    if (v > max) max = v;
  }
  return max + 1;
}

function formatOutput(
  algo: AlgoId,
  result: AlgoResult,
  t: (key: string) => string,
): string {
  const lines: string[] = [];

  if (algo === 'pagerank' && 'scores' in result) {
    const r = result as AlgoResultScores;
    lines.push(t('output.pagerank'));
    r.scores.forEach((s, i) => lines.push(`  vertex ${i}: ${s.toFixed(6)}`));
  } else if (algo === 'louvain' && 'membership' in result) {
    const r = result as AlgoResultMembership;
    lines.push(t('output.louvain').replace('{mod}', (r.modularity ?? 0).toFixed(4)));
    r.membership.forEach((c, i) => lines.push(`  vertex ${i}: community ${c}`));
  } else if (algo === 'betweenness' && 'scores' in result) {
    const r = result as AlgoResultScores;
    lines.push(t('output.betweenness'));
    r.scores.forEach((s, i) => lines.push(`  vertex ${i}: ${s.toFixed(4)}`));
  } else if (algo === 'bfs' && 'order' in result) {
    const r = result as AlgoResultOrder;
    lines.push(t('output.bfs'));
    lines.push('  ' + r.order.join(' → '));
  } else if (algo === 'components' && 'membership' in result) {
    const r = result as AlgoResultMembership;
    lines.push(t('output.components').replace('{count}', String(r.count ?? 0)));
    r.membership.forEach((c, i) => lines.push(`  vertex ${i}: component ${c}`));
  } else if (algo === 'infomap' && 'membership' in result) {
    const r = result as AlgoResultMembership;
    lines.push(t('output.infomap').replace('{cl}', (r.codelength ?? 0).toFixed(4)));
    r.membership.forEach((c, i) => lines.push(`  vertex ${i}: community ${c}`));
  } else if (algo === 'spinglass' && 'membership' in result) {
    const r = result as AlgoResultMembership;
    lines.push(
      t('output.spinglass')
        .replace('{mod}', (r.modularity ?? 0).toFixed(4))
        .replace('{k}', String(r.nb_clusters ?? '?')),
    );
    r.membership.forEach((c, i) => lines.push(`  vertex ${i}: community ${c}`));
  }

  return lines.join('\n');
}

export function App() {
  const { theme, toggleTheme } = useTheme();
  const { lang, toggleLang, t } = useI18n();
  const { sizes, resizeLeft, resizeCenter, resizeCode, persistSizes } = useResizablePanels();

  const [presetId, setPresetId] = useState('karate');
  const [edgeText, setEdgeText] = useState(edgesFromPreset('karate'));
  const [directed, setDirected] = useState(false);
  const [algo, setAlgo] = useState<AlgoId>('pagerank');
  const [params, setParams] = useState<AlgoParams>({ damping: 0.85, source: 0 });

  const [coords, setCoords] = useState<[number, number][] | null>(null);
  const [result, setResult] = useState<AlgoResult | null>(null);
  const [output, setOutput] = useState('');
  const [elapsed, setElapsed] = useState<number | null>(null);

  const tRef = useRef(t);
  tRef.current = t;
  const algoRef = useRef(algo);
  algoRef.current = algo;

  const applyRunResult = useCallback((runResult: RunResult) => {
    setCoords(runResult.coords);
    setResult(runResult.result);
    setElapsed(runResult.elapsed_ms);
    setOutput(formatOutput(runResult.algo, runResult.result, tRef.current));
  }, []);

  const { status, wasmAvailable, run } = useWasm(applyRunResult);

  const edges = parseEdges(edgeText);
  const vcount = getVcount(edges);

  const handlePresetChange = useCallback(
    (id: string) => {
      setPresetId(id);
      const preset = PRESETS[id];
      if (preset) {
        setEdgeText(edgesFromPreset(id));
        setDirected(preset.directed);
      }
    },
    [],
  );

  const handleRun = useCallback(() => {
    const runResult = run(algo, edges, directed, params);
    if (runResult) {
      applyRunResult(runResult);
    }
  }, [algo, edges, directed, params, run, applyRunResult]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        handleRun();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [handleRun]);

  useEffect(() => {
    if (status === 'ready') {
      handleRun();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  const statusText = status === 'running'
    ? t('status.running')
    : wasmAvailable
      ? t('status.ready')
      : t('status.demo');

  return (
    <div className="app">
      <Header
        theme={theme}
        lang={lang}
        onToggleTheme={toggleTheme}
        onToggleLang={toggleLang}
        t={t}
      />

      <div className="workspace">
        <div className="workspace-top">
          <div className="panel panel-left" style={{ width: sizes.leftWidth }}>
            <GraphEditor
              edgeText={edgeText}
              directed={directed}
              presetId={presetId}
              onEdgeTextChange={setEdgeText}
              onDirectedChange={setDirected}
              onPresetChange={handlePresetChange}
              t={t}
            />
          </div>

          <Resizer direction="horizontal" onResize={resizeLeft} onResizeEnd={persistSizes} />

          <div className="panel panel-center" style={{ width: sizes.centerWidth }}>
            <AlgoPanel
              algo={algo}
              params={params}
              running={status === 'running'}
              onAlgoChange={setAlgo}
              onParamsChange={setParams}
              onRun={handleRun}
              t={t}
            />
          </div>

          <Resizer direction="horizontal" onResize={resizeCenter} onResizeEnd={persistSizes} />

          <div className="panel panel-right">
            <div className="panel-header">
              <h2>{t('results')}</h2>
              <div className="stats">
                {vcount > 0 && (
                  <>
                    <span>{t('nodes')}: {vcount}</span>
                    <span>{t('edges')}: {edges.length}</span>
                    {elapsed != null && <span>{t('time')}: {elapsed.toFixed(1)}ms</span>}
                  </>
                )}
                <span className={`status ${status}`}>{statusText}</span>
              </div>
            </div>

            <Canvas
              coords={coords}
              edges={edges}
              vcount={vcount}
              result={result}
              algo={algo}
              directed={directed}
              theme={theme}
              t={t}
            />

            <pre className="output">{output}</pre>
          </div>
        </div>

        <Resizer direction="vertical" onResize={resizeCode} onResizeEnd={persistSizes} />

        <div className="code-section" style={{ height: sizes.codeHeight }}>
          <CodeEditor
            algo={algo}
            edges={edges}
            directed={directed}
            theme={theme}
            t={t}
          />
        </div>
      </div>
    </div>
  );
}
