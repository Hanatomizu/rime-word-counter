/** 自绘下拉选择：原生 `<select>` 在深色 WebView 里无法统一风格 */

import { useEffect, useRef, useState } from 'react';

interface Option<T extends string> {
  value: T;
  label: string;
}

interface SelectProps<T extends string> {
  value: T;
  options: Array<Option<T>>;
  onChange: (value: T) => void;
  label: string;
}

export function Select<T extends string>({ value, options, onChange, label }: SelectProps<T>) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };

    document.addEventListener('mousedown', handlePointerDown);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handlePointerDown);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [open]);

  const current = options.find((option) => option.value === value);

  return (
    <div className="select" ref={rootRef}>
      <button
        type="button"
        className="select__button"
        aria-label={label}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((previous) => !previous)}
      >
        <span>{current?.label ?? value}</span>
        <svg
          width="10"
          height="10"
          viewBox="0 0 10 10"
          aria-hidden="true"
          style={{
            transform: open ? 'rotate(180deg)' : 'none',
            transition: 'transform 140ms cubic-bezier(0.32, 0.72, 0, 1)',
          }}
        >
          <path
            d="M2 3.8 5 6.8l3-3"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {open && (
        <div className="select__menu" role="listbox" aria-label={label}>
          {options.map((option) => (
            <button
              key={option.value}
              type="button"
              role="option"
              aria-selected={option.value === value}
              className="select__option"
              onClick={() => {
                onChange(option.value);
                setOpen(false);
              }}
            >
              <span>{option.label}</span>
              {option.value === value && (
                <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                  <path
                    d="M1.5 5.2 3.8 7.5 8.5 2.8"
                    fill="none"
                    stroke="var(--primary-hover)"
                    strokeWidth="1.4"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
