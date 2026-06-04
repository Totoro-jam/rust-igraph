import { useState, useCallback, useEffect, useRef } from 'react';
import type { GeneratorId, GeneratorParams } from '../../types';
import styles from './index.module.css';

interface GeneratorDialogProps {
  open: boolean;
  wasmAvailable: boolean;
  onGenerate: (generator: GeneratorId, params: GeneratorParams) => void;
  onClose: () => void;
  t: (key: string) => string;
}

const GENERATORS: GeneratorId[] = [
  'erdos_renyi',
  'barabasi_albert',
  'watts_strogatz',
  'complete',
  'cycle',
  'path',
  'star',
  'ring',
  'famous',
];

const FAMOUS_GRAPHS = [
  'Petersen', 'Bull', 'Chvatal', 'Coxeter', 'Cubical',
  'Diamond', 'Dodecahedral', 'Frucht', 'Grotzsch', 'Heawood',
  'Herschel', 'House', 'HouseX', 'Icosahedral', 'Krackhardt_Kite',
  'McGee', 'Meredith', 'Noperfectmatching', 'Nonline',
  'Octahedral', 'Pappus', 'Robertson', 'Smallestcyclicgroup',
  'Tetrahedral', 'Thomassen', 'Tutte',
];

export function GeneratorDialog({
  open,
  wasmAvailable,
  onGenerate,
  onClose,
  t,
}: GeneratorDialogProps) {
  const [generator, setGenerator] = useState<GeneratorId>('erdos_renyi');
  const [n, setN] = useState(50);
  const [p, setP] = useState(0.1);
  const [m, setM] = useState(2);
  const [k, setK] = useState(4);
  const [seed, setSeed] = useState(42);
  const [directed, setDirected] = useState(false);
  const [famousName, setFamousName] = useState('Petersen');
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [open, onClose]);

  const handleGenerate = useCallback(() => {
    const params: GeneratorParams = { seed };
    switch (generator) {
      case 'erdos_renyi':
        params.n = n;
        params.p = p;
        break;
      case 'barabasi_albert':
        params.n = n;
        params.m = m;
        break;
      case 'watts_strogatz':
        params.n = n;
        params.k = k;
        params.p = p;
        break;
      case 'complete':
      case 'cycle':
      case 'star':
      case 'ring':
        params.n = n;
        break;
      case 'path':
        params.n = n;
        params.directed = directed;
        break;
      case 'famous':
        params.name = famousName;
        break;
    }
    onGenerate(generator, params);
    onClose();
  }, [generator, n, p, m, k, seed, directed, famousName, onGenerate, onClose]);

  if (!open) return null;

  return (
    <div className={styles.overlay} onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className={styles.dialog} ref={dialogRef}>
        <h3 className={styles.dialogTitle}>{t('gen.title')}</h3>

        <div className={styles.field}>
          <label className={styles.fieldLabel}>{t('gen.model')}</label>
          <select
            className={styles.fieldSelect}
            value={generator}
            onChange={(e) => setGenerator(e.target.value as GeneratorId)}
          >
            {GENERATORS.map((g) => (
              <option key={g} value={g}>{t(`gen.${g}`)}</option>
            ))}
          </select>
        </div>

        {generator === 'famous' && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>{t('gen.famousName')}</label>
            <select
              className={styles.fieldSelect}
              value={famousName}
              onChange={(e) => setFamousName(e.target.value)}
            >
              {FAMOUS_GRAPHS.map((name) => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>
          </div>
        )}

        {generator !== 'famous' && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>
              {generator === 'star' ? t('gen.nStar') : t('gen.n')}
            </label>
            <input
              className={styles.fieldInput}
              type="number"
              min={2}
              max={500}
              value={n}
              onChange={(e) => setN(Math.max(2, Math.min(500, Number(e.target.value))))}
            />
            <div className={styles.fieldHint}>2 – 500</div>
          </div>
        )}

        {(generator === 'erdos_renyi' || generator === 'watts_strogatz') && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>{t('gen.p')}</label>
            <input
              className={styles.fieldInput}
              type="number"
              min={0}
              max={1}
              step={0.01}
              value={p}
              onChange={(e) => setP(Math.max(0, Math.min(1, Number(e.target.value))))}
            />
            <div className={styles.fieldHint}>0.0 – 1.0</div>
          </div>
        )}

        {generator === 'barabasi_albert' && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>{t('gen.m')}</label>
            <input
              className={styles.fieldInput}
              type="number"
              min={1}
              max={20}
              value={m}
              onChange={(e) => setM(Math.max(1, Math.min(20, Number(e.target.value))))}
            />
            <div className={styles.fieldHint}>{t('gen.mHint')}</div>
          </div>
        )}

        {generator === 'watts_strogatz' && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>{t('gen.k')}</label>
            <input
              className={styles.fieldInput}
              type="number"
              min={2}
              max={20}
              value={k}
              onChange={(e) => setK(Math.max(2, Math.min(20, Number(e.target.value))))}
            />
            <div className={styles.fieldHint}>{t('gen.kHint')}</div>
          </div>
        )}

        {generator === 'path' && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>
              <input
                type="checkbox"
                checked={directed}
                onChange={(e) => setDirected(e.target.checked)}
              />{' '}
              {t('directed')}
            </label>
          </div>
        )}

        {(generator === 'erdos_renyi' || generator === 'barabasi_albert' || generator === 'watts_strogatz') && (
          <div className={styles.field}>
            <label className={styles.fieldLabel}>{t('gen.seed')}</label>
            <input
              className={styles.fieldInput}
              type="number"
              min={0}
              value={seed}
              onChange={(e) => setSeed(Math.max(0, Number(e.target.value)))}
            />
          </div>
        )}

        <div className={styles.actions}>
          <button className={styles.btnCancel} onClick={onClose}>
            {t('gen.cancel')}
          </button>
          <button
            className={styles.btnGenerate}
            onClick={handleGenerate}
            disabled={!wasmAvailable}
          >
            {t('gen.generate')}
          </button>
        </div>
      </div>
    </div>
  );
}
