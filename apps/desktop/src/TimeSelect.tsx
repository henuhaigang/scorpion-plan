import { useState, useRef, useEffect } from "react";

interface TimeSelectProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

const TIME_SLOTS: string[] = [];
for (let h = 0; h < 24; h++) {
  TIME_SLOTS.push(`${String(h).padStart(2, "0")}:00`);
  TIME_SLOTS.push(`${String(h).padStart(2, "0")}:30`);
}

export function TimeSelect({ value, onChange, placeholder }: TimeSelectProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState(value);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setQuery(value);
  }, [value]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const filtered = query
    ? TIME_SLOTS.filter((t) => t.startsWith(query.replace(/[^0-9]/g, "").slice(0, 4)))
    : TIME_SLOTS;

  const handleSelect = (time: string) => {
    onChange(time);
    setOpen(false);
  };

  const handleInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);
    onChange(val);
    if (val === "") setOpen(true);
  };

  return (
    <div className="time-select" ref={ref}>
      <input
        type="text"
        className="time-select-input"
        value={query}
        placeholder={placeholder || "HH:MM"}
        onChange={handleInput}
        onFocus={() => setOpen(true)}
        autoComplete="off"
      />
      {open && (
        <ul className="time-select-dropdown" role="listbox">
          {filtered.length === 0 ? (
            <li className="time-select-empty">无匹配时间</li>
          ) : (
            filtered.map((time) => (
              <li
                key={time}
                role="option"
                aria-selected={time === value}
                className={`time-select-option${time === value ? " time-select-option--selected" : ""}`}
                onClick={() => handleSelect(time)}
              >
                {time}
              </li>
            ))
          )}
        </ul>
      )}
    </div>
  );
}
