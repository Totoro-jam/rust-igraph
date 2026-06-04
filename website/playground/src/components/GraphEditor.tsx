import { PresetPicker } from './PresetPicker';

interface GraphEditorProps {
  edgeText: string;
  directed: boolean;
  presetId: string;
  onEdgeTextChange: (text: string) => void;
  onDirectedChange: (directed: boolean) => void;
  onPresetChange: (id: string) => void;
  t: (key: string) => string;
}

export function GraphEditor({
  edgeText,
  directed,
  presetId,
  onEdgeTextChange,
  onDirectedChange,
  onPresetChange,
  t,
}: GraphEditorProps) {
  return (
    <>
      <div className="panel-header">
        <h2>{t('graphEditor')}</h2>
        <label className="directed-label">
          <input
            type="checkbox"
            checked={directed}
            onChange={(e) => onDirectedChange(e.target.checked)}
          />
          {t('directed')}
        </label>
      </div>
      <PresetPicker value={presetId} onChange={onPresetChange} t={t} />
      <textarea
        className="edge-input"
        value={edgeText}
        onChange={(e) => onEdgeTextChange(e.target.value)}
        spellCheck={false}
        placeholder={t('edgePlaceholder')}
      />
      <div className="editor-hint">{t('ctrlEnter')}</div>
    </>
  );
}
