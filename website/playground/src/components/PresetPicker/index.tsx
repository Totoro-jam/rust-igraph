import { PRESET_ORDER } from '../../presets';
import styles from './index.module.css';

interface PresetPickerProps {
  value: string;
  onChange: (id: string) => void;
  t: (key: string) => string;
}

export function PresetPicker({ value, onChange, t }: PresetPickerProps) {
  return (
    <div className={styles.presetPicker}>
      <label className={styles.presetLabel}>{t('preset')}</label>
      <select
        className={styles.presetSelect}
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
