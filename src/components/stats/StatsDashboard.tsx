import { useEffect, useState } from "react";
import { api, type KarmaOverview } from "../../lib/tauri";
import { formatKarmaDollars } from "../../stores/karmaStore";
import { useProfileStore } from "../../stores/profileStore";

interface StatsOverview {
  total_cards: number;
  new_cards: number;
  learning_cards: number;
  review_cards: number;
  total_decks: number;
  total_reviews_today: number;
  streak_days: number;
  daily_reviews: DailyReview[];
}

interface DailyReview {
  date: string;
  count: number;
  again: number;
  hard: number;
  good: number;
  easy: number;
}

export function StatsDashboard() {
  const active = useProfileStore((s) => s.active);
  const [stats, setStats] = useState<StatsOverview | null>(null);
  const [karma, setKarma] = useState<KarmaOverview | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([api.getStatsOverview(), api.getKarmaOverview()])
      .then(([s, k]) => {
        setStats(s);
        setKarma(k);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [active?.id]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Loading stats...
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Failed to load stats.
      </div>
    );
  }

  const maxDaily = Math.max(...stats.daily_reviews.map((d) => d.count), 1);

  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <h2 className="text-xl font-bold text-text">Statistics</h2>

      {/* Summary cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard label="Total Cards" value={stats.total_cards} color="text" />
        <StatCard label="New" value={stats.new_cards} color="primary-500" />
        <StatCard
          label="Learning"
          value={stats.learning_cards}
          color="warning"
        />
        <StatCard label="Review" value={stats.review_cards} color="success" />
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        <StatCard
          label="Reviewed Today"
          value={stats.total_reviews_today}
          color="primary-500"
        />
        <StatCard
          label="Review Streak"
          value={`${stats.streak_days}d`}
          color="warning"
        />
        <StatCard label="Decks" value={stats.total_decks} color="text" />
      </div>

      {karma && (
        <div className="bg-surface-alt border border-border rounded-2xl p-6 space-y-4">
          <h3 className="text-sm font-semibold text-text-secondary">
            Karma — {active?.display_name ?? "Profile"}
          </h3>
          {karma.is_admin ? (
            <p className="text-sm text-text-muted">
              Admin profile does not earn Karma points.
            </p>
          ) : (
            <>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <StatCard
                  label="Balance"
                  value={formatKarmaDollars(karma.balance_cents)}
                  color="success"
                />
                <StatCard
                  label="Karma Streak"
                  value={`${karma.streak_days}d`}
                  color="warning"
                />
                <StatCard
                  label="Active Today"
                  value={`${Math.floor(karma.today_active_seconds / 60)}m`}
                  color="primary-500"
                />
                <StatCard
                  label="Effective Actions"
                  value={karma.today_effective_actions}
                  color="text"
                />
              </div>
              <p className="text-xs text-text-muted">
                Qualify each day with 10+ minutes active or 15+ effective actions
                (reviews + 2× cards added). Earn $0.10 per review, $0.20 per card
                added; +$5 every 7 qualifying days.
              </p>
              {karma.daily_qualified.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {karma.daily_qualified
                    .slice()
                    .reverse()
                    .slice(0, 14)
                    .map((d) => (
                      <span
                        key={d.day}
                        title={d.day}
                        className={`w-4 h-4 rounded-sm ${
                          d.qualified ? "bg-success" : "bg-surface border border-border"
                        }`}
                      />
                    ))}
                </div>
              )}
            </>
          )}
        </div>
      )}

      {/* Card state distribution */}
      <div className="bg-surface-alt border border-border rounded-2xl p-6">
        <h3 className="text-sm font-semibold text-text-secondary mb-4">
          Card Distribution
        </h3>
        <div className="flex h-6 rounded-full overflow-hidden bg-surface">
          {stats.total_cards > 0 && (
            <>
              <div
                className="bg-primary-500 transition-all"
                style={{
                  width: `${(stats.new_cards / stats.total_cards) * 100}%`,
                }}
                title={`New: ${stats.new_cards}`}
              />
              <div
                className="bg-orange-500 transition-all"
                style={{
                  width: `${(stats.learning_cards / stats.total_cards) * 100}%`,
                }}
                title={`Learning: ${stats.learning_cards}`}
              />
              <div
                className="bg-green-500 transition-all"
                style={{
                  width: `${(stats.review_cards / stats.total_cards) * 100}%`,
                }}
                title={`Review: ${stats.review_cards}`}
              />
            </>
          )}
        </div>
        <div className="flex gap-4 mt-2 text-xs text-text-muted">
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-primary-500" /> New
          </span>
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-orange-500" /> Learning
          </span>
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-green-500" /> Review
          </span>
        </div>
      </div>

      {/* Daily activity chart */}
      {stats.daily_reviews.length > 0 && (
        <div className="bg-surface-alt border border-border rounded-2xl p-6">
          <h3 className="text-sm font-semibold text-text-secondary mb-4">
            Last 30 Days Activity
          </h3>
          <div className="flex items-end gap-1 h-32">
            {stats.daily_reviews
              .slice()
              .reverse()
              .map((d) => {
                const height = (d.count / maxDaily) * 100;
                return (
                  <div
                    key={d.date}
                    className="flex-1 flex flex-col items-stretch justify-end gap-0"
                    title={`${d.date}: ${d.count} reviews`}
                  >
                    <div className="flex flex-col justify-end" style={{ height: `${height}%` }}>
                      <div
                        className="bg-red-400 rounded-t-sm"
                        style={{
                          height: `${d.count > 0 ? (d.again / d.count) * 100 : 0}%`,
                          minHeight: d.again > 0 ? "2px" : 0,
                        }}
                      />
                      <div
                        className="bg-orange-400"
                        style={{
                          height: `${d.count > 0 ? (d.hard / d.count) * 100 : 0}%`,
                          minHeight: d.hard > 0 ? "2px" : 0,
                        }}
                      />
                      <div
                        className="bg-green-400"
                        style={{
                          height: `${d.count > 0 ? (d.good / d.count) * 100 : 0}%`,
                          minHeight: d.good > 0 ? "2px" : 0,
                        }}
                      />
                      <div
                        className="bg-primary-400 rounded-b-sm"
                        style={{
                          height: `${d.count > 0 ? (d.easy / d.count) * 100 : 0}%`,
                          minHeight: d.easy > 0 ? "2px" : 0,
                        }}
                      />
                    </div>
                  </div>
                );
              })}
          </div>
          <div className="flex justify-between mt-2 text-xs text-text-muted">
            <span>30 days ago</span>
            <span>Today</span>
          </div>
        </div>
      )}
    </div>
  );
}

function StatCard({
  label,
  value,
  color,
}: {
  label: string;
  value: string | number;
  color: string;
}) {
  return (
    <div className="bg-surface-alt border border-border rounded-xl p-4 text-center">
      <p className={`text-2xl font-bold text-${color}`}>{value}</p>
      <p className="text-xs text-text-muted mt-1">{label}</p>
    </div>
  );
}
