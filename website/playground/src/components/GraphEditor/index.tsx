import { useState, useCallback } from 'react';
import { PresetPicker } from '../PresetPicker';
import { GeneratorDialog } from '../GeneratorDialog';
import type { GeneratorId, GeneratorParams } from '../../types';
import styles from './index.module.css';

interface GraphEditorProps {
  edgeText: string;
  presetId: string;
  wasmAvailable: boolean;
  onEdgeTextChange: (text: string) => void;
  onPresetChange: (id: string) => void;
  onGenerate: (generator: GeneratorId, params: GeneratorParams) => void;
  t: (key: string) => string;
}

export function GraphEditor({
  edgeText,
  presetId,
  wasmAvailable,
  onEdgeTextChange,
  onPresetChange,
  onGenerate,
  t,
}: GraphEditorProps) {
  const [genOpen, setGenOpen] = useState(false);

  const handleOpenGen = useCallback(() => setGenOpen(true), []);
  const handleCloseGen = useCallback(() => setGenOpen(false), []);

  return (
    <>
      <div className={styles.topBar}>
        <PresetPicker value={presetId} onChange={onPresetChange} t={t} />
        <button
          className={styles.generateBtn}
          onClick={handleOpenGen}
          disabled={!wasmAvailable}
          title={t('gen.title')}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
          </svg>
          {t('gen.btn')}
        </button>
      </div>
      <textarea
        className={styles.edgeInput}
        value={edgeText}
        onChange={(e) => onEdgeTextChange(e.target.value)}
        spellCheck={false}
        placeholder={t('edgePlaceholder')}
      />
      <div className={styles.editorHint}>{t('ctrlEnter')}</div>
      <GeneratorDialog
        open={genOpen}
        wasmAvailable={wasmAvailable}
        onGenerate={onGenerate}
        onClose={handleCloseGen}
        t={t}
      />
    </>
  );
}
