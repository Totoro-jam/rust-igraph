import { PresetPicker } from './PresetPicker';

interface GraphEditorProps {
  edgeText: string;
  presetId: string;
  onEdgeTextChange: (text: string) => void;
  onPresetChange: (id: string) => void;
  t: (key: string) => string;
}

export function GraphEditor({
  edgeText,
  presetId,
  onEdgeTextChange,
  onPresetChange,
  t,
}: GraphEditorProps) {
  return (
    <>
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
