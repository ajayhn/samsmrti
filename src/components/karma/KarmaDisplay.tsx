import { useKarmaStore, formatKarmaDollars } from "../../stores/karmaStore";
import { useProfileStore } from "../../stores/profileStore";

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  return `${m}m`;
}

export function KarmaDisplay({ className = "" }: { className?: string }) {
  const active = useProfileStore((s) => s.active);
  const overview = useKarmaStore((s) => s.overview);
  const displayCents = useKarmaStore((s) => s.displayCents);
  const floatDelta = useKarmaStore((s) => s.floatDelta);

  if (!active) return null;

  if (active.is_admin) {
    return (
      <div
        className={`text-xs text-text-muted tabular-nums ${className}`}
        title="Admin profile does not earn Karma"
      >
        Admin · no karma
      </div>
    );
  }

  const streak = overview?.streak_days ?? 0;
  const activeSec = overview?.today_active_seconds ?? 0;
  const effective = overview?.today_effective_actions ?? 0;
  const qualified = overview?.qualified_today ?? false;

  const tooltip = [
    `${streak}-day streak`,
    qualified ? "Qualified today" : "Not qualified yet",
    `${formatDuration(activeSec)} / 10m active`,
    `${effective} / 15 effective actions`,
    streak > 0 && streak % 7 !== 0
      ? `${7 - (streak % 7)} days to +$5 bonus`
      : streak >= 7 && streak % 7 === 0
        ? "Weekly bonus earned!"
        : null,
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <div className={`relative flex items-center gap-2 ${className}`}>
      {floatDelta != null && floatDelta !== 0 && (
        <span
          className={`absolute -top-5 right-0 text-xs font-semibold tabular-nums animate-pulse ${
            floatDelta > 0 ? "text-success" : "text-warning"
          }`}
        >
          {floatDelta > 0 ? "+" : ""}
          {formatKarmaDollars(floatDelta)}
        </span>
      )}
      <div
        className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-surface-alt border border-border text-sm font-semibold text-text tabular-nums cursor-default"
        title={tooltip}
      >
        <span className="text-success" aria-hidden>
          $
        </span>
        <span>{formatKarmaDollars(displayCents).slice(1)}</span>
      </div>
    </div>
  );
}
