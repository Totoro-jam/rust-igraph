import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import { Header } from './components/Header';
import { GraphEditor } from './components/GraphEditor';
import { AlgoPanel } from './components/AlgoPanel';
import { Canvas } from './components/Canvas';
import { CodeEditor } from './components/CodeEditor';
import { ResultsOutput } from './components/ResultsOutput';
import { Resizer } from './components/Resizer';
import { useTheme } from './hooks/useTheme';
import { useI18n } from './hooks/useI18n';
import { useWasm } from './hooks/useWasm';
import { useResizablePanels } from './hooks/useResizablePanels';
import { PRESETS } from './presets';
import type { AlgoId, AlgoParams, AlgoResult, Edge, RunResult } from './types';
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

export function App() {
  const { theme, toggleTheme } = useTheme();
  const { lang, toggleLang, t } = useI18n();
  const { sizes, resizeLeft, resizeCenter, resizeCode, persistSizes } = useResizablePanels();

  const [presetId, setPresetId] = useState('karate');
  const [edgeText, setEdgeText] = useState(edgesFromPreset('karate'));
  const [directed, setDirected] = useState(false);
  const [algo, setAlgo] = useState<AlgoId>('pagerank');
  const [params, setParams] = useState<AlgoParams>({ damping: 0.85, source: 0 });

  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [centerCollapsed, setCenterCollapsed] = useState(false);
  const [codeCollapsed, setCodeCollapsed] = useState(true);

  const [coords, setCoords] = useState<[number, number][] | null>(null);
  const [result, setResult] = useState<AlgoResult | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);

  const algoRef = useRef(algo);
  algoRef.current = algo;
  const initialRunDone = useRef(false);

  const applyRunResult = useCallback((runResult: RunResult) => {
    setCoords(runResult.coords);
    setResult(runResult.result);
    setElapsed(runResult.elapsed_ms);
  }, []);

  const { status, wasmAvailable, run } = useWasm(applyRunResult);

  const edges = useMemo(() => parseEdges(edgeText), [edgeText]);
  const vcount = useMemo(() => getVcount(edges), [edges]);

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
    if (status === 'ready' && !initialRunDone.current) {
      initialRunDone.current = true;
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
          <div
            className={`panel panel-left${leftCollapsed ? ' panel-collapsed' : ''}`}
            style={{ width: leftCollapsed ? undefined : sizes.leftWidth }}
            onClick={leftCollapsed ? () => setLeftCollapsed(false) : undefined}
          >
            <div className="panel-header">
              <h2>{t('graphEditor')}</h2>
              <div className="panel-header-actions">
                {!leftCollapsed && (
                  <label className="directed-label">
                    <input
                      type="checkbox"
                      checked={directed}
                      onChange={(e) => setDirected(e.target.checked)}
                    />
                    {t('directed')}
                  </label>
                )}
                <button
                  className="collapse-toggle"
                  onClick={() => setLeftCollapsed(!leftCollapsed)}
                  aria-label={leftCollapsed ? t('expand') : t('collapse')}
                  title={leftCollapsed ? t('expand') : t('collapse')}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    {leftCollapsed
                      ? <polyline points="9 18 15 12 9 6" />
                      : <polyline points="15 18 9 12 15 6" />
                    }
                  </svg>
                </button>
              </div>
            </div>
            {!leftCollapsed && (
              <GraphEditor
                edgeText={edgeText}
                presetId={presetId}
                onEdgeTextChange={setEdgeText}
                onPresetChange={handlePresetChange}
                t={t}
              />
            )}
          </div>

          {!leftCollapsed && (
            <Resizer direction="horizontal" onResize={resizeLeft} onResizeEnd={persistSizes} />
          )}

          <div
            className={`panel panel-center${centerCollapsed ? ' panel-collapsed' : ''}`}
            style={{ width: centerCollapsed ? undefined : sizes.centerWidth }}
            onClick={centerCollapsed ? () => setCenterCollapsed(false) : undefined}
          >
            <div className="panel-header">
              <h2>{t('algorithm')}</h2>
              <button
                className="collapse-toggle"
                onClick={() => setCenterCollapsed(!centerCollapsed)}
                aria-label={centerCollapsed ? t('expand') : t('collapse')}
                title={centerCollapsed ? t('expand') : t('collapse')}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  {centerCollapsed
                    ? <polyline points="9 18 15 12 9 6" />
                    : <polyline points="15 18 9 12 15 6" />
                  }
                </svg>
              </button>
            </div>
            {!centerCollapsed && (
              <AlgoPanel
                algo={algo}
                params={params}
                running={status === 'running'}
                onAlgoChange={setAlgo}
                onParamsChange={setParams}
                onRun={handleRun}
                t={t}
              />
            )}
          </div>

          {!centerCollapsed && (
            <Resizer direction="horizontal" onResize={resizeCenter} onResizeEnd={persistSizes} />
          )}

          <div className="panel panel-right">
            <div className="panel-header">
              <h2>{t('results')}</h2>
              <span className={`status ${status}`}>{statusText}</span>
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

            <ResultsOutput
              algo={algo}
              result={result}
              elapsed={elapsed}
              vcount={vcount}
              edgeCount={edges.length}
              t={t}
            />
          </div>
        </div>

        <Resizer direction="vertical" onResize={resizeCode} onResizeEnd={persistSizes} />

        <div
          className={`code-section${codeCollapsed ? ' code-collapsed' : ''}`}
          style={{ height: codeCollapsed ? undefined : sizes.codeHeight }}
        >
          <div className="code-panel-header">
            <h3>{t('code')}</h3>
            <button
              className="collapse-toggle"
              onClick={() => setCodeCollapsed(!codeCollapsed)}
              aria-label={codeCollapsed ? t('expand') : t('collapse')}
              title={codeCollapsed ? t('expand') : t('collapse')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                {codeCollapsed
                  ? <polyline points="6 9 12 15 18 9" />
                  : <polyline points="18 15 12 9 6 15" />
                }
              </svg>
            </button>
          </div>
          {!codeCollapsed && (
            <CodeEditor
              algo={algo}
              edges={edges}
              directed={directed}
              theme={theme}
            />
          )}
        </div>
      </div>
    </div>
  );
}
