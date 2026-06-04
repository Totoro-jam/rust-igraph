import { PRESET_ORDER } from '../presets';

interface PresetPickerProps {
  value: string;
  onChange: (id: string) => void;
  t: (key: string) => string;
}

export function PresetPicker({ value, onChange, t }: PresetPickerProps) {
  return (
    <div className="preset-picker">
      <label className="preset-label">{t('preset')}</label>
      <select
        className="preset-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {PRESET_ORDER.map((id) => (
          <option key={id} value={id}>
            {t(`preset.${id}`)}
          </option>
        ))}
      </select>
    </div>
  );
}
