import { useState, useCallback } from 'react';

const STORAGE_KEY = 'playground-panel-sizes';

interface PanelSizes {
  leftWidth: number;
  centerWidth: number;
  codeHeight: number;
  resultsHeight: number;
}

const DEFAULTS: PanelSizes = {
  leftWidth: 280,
  centerWidth: 240,
  codeHeight: 180,
  resultsHeight: 200,
};

const LIMITS = {
  leftMin: 180,
  leftMax: 500,
  centerMin: 160,
  centerMax: 400,
  codeMin: 80,
  codeMax: 400,
  resultsMin: 60,
  resultsMax: 500,
};

function loadSizes(): PanelSizes {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<PanelSizes>;
      return {
        leftWidth: clamp(parsed.leftWidth ?? DEFAULTS.leftWidth, LIMITS.leftMin, LIMITS.leftMax),
        centerWidth: clamp(parsed.centerWidth ?? DEFAULTS.centerWidth, LIMITS.centerMin, LIMITS.centerMax),
        codeHeight: clamp(parsed.codeHeight ?? DEFAULTS.codeHeight, LIMITS.codeMin, LIMITS.codeMax),
        resultsHeight: clamp(parsed.resultsHeight ?? DEFAULTS.resultsHeight, LIMITS.resultsMin, LIMITS.resultsMax),
      };
    }
  } catch { /* ignore */ }
  return { ...DEFAULTS };
}

function saveSizes(sizes: PanelSizes): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sizes));
  } catch { /* ignore */ }
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

export function useResizablePanels() {
  const [sizes, setSizes] = useState<PanelSizes>(loadSizes);

  const resizeLeft = useCallback((delta: number) => {
    setSizes((prev) => ({
      ...prev,
      leftWidth: clamp(prev.leftWidth + delta, LIMITS.leftMin, LIMITS.leftMax),
    }));
  }, []);

  const resizeCenter = useCallback((delta: number) => {
    setSizes((prev) => ({
      ...prev,
      centerWidth: clamp(prev.centerWidth + delta, LIMITS.centerMin, LIMITS.centerMax),
    }));
  }, []);

  const resizeCode = useCallback((delta: number) => {
    setSizes((prev) => ({
      ...prev,
      codeHeight: clamp(prev.codeHeight - delta, LIMITS.codeMin, LIMITS.codeMax),
    }));
  }, []);

  const resizeResults = useCallback((delta: number) => {
    setSizes((prev) => ({
      ...prev,
      resultsHeight: clamp(prev.resultsHeight - delta, LIMITS.resultsMin, LIMITS.resultsMax),
    }));
  }, []);

  const persistSizes = useCallback(() => {
    setSizes((current) => {
      saveSizes(current);
      return current;
    });
  }, []);

  return { sizes, resizeLeft, resizeCenter, resizeCode, resizeResults, persistSizes };
}
