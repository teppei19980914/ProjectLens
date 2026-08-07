import { cn } from "@/lib/utils";

export interface TabItem {
  key: string;
  label: string;
}

interface TabsProps {
  items: TabItem[];
  active: string;
  onChange: (key: string) => void;
}

export function Tabs({ items, active, onChange }: TabsProps) {
  return (
    <div className="flex gap-1 border-b border-gray-200 dark:border-gray-800">
      {items.map((item) => (
        <button
          key={item.key}
          onClick={() => onChange(item.key)}
          className={cn(
            "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
            active === item.key
              ? "border-blue-600 text-blue-600"
              : "border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
          )}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
