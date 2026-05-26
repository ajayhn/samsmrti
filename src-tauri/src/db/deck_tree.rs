use rusqlite::Connection;
use std::collections::HashMap;

/// Deck id plus all descendant deck ids (includes `root_id`).
pub fn deck_scope_ids(conn: &Connection, root_id: &str) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE scope AS (
            SELECT id FROM decks WHERE id = ?1
            UNION ALL
            SELECT d.id FROM decks d
            INNER JOIN scope s ON d.parent_id = s.id
         )
         SELECT id FROM scope",
    )?;
    let ids = stmt
        .query_map([root_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

pub fn is_ancestor_of(
    conn: &Connection,
    ancestor_id: &str,
    deck_id: &str,
) -> Result<bool, rusqlite::Error> {
    if ancestor_id == deck_id {
        return Ok(true);
    }
    let scope = deck_scope_ids(conn, ancestor_id)?;
    Ok(scope.iter().any(|id| id == deck_id))
}

/// Direct card counts keyed by deck id (notes in that deck only), for one profile.
pub fn direct_deck_counts(
    conn: &Connection,
    profile_id: &str,
    now: i64,
) -> Result<HashMap<String, (i64, i64, i64)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT n.deck_id,
                COUNT(c.id) AS total,
                SUM(CASE WHEN cp.state IN ('review','relearning') AND cp.due_at <= ?2
                    AND (cp.buried_until IS NULL OR cp.buried_until <= ?2) THEN 1 ELSE 0 END) AS due,
                SUM(CASE WHEN cp.state = 'new'
                    AND (cp.buried_until IS NULL OR cp.buried_until <= ?2) THEN 1 ELSE 0 END) AS new_count
         FROM notes n
         JOIN cards c ON c.note_id = n.id
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?1
         GROUP BY n.deck_id",
    )?;
    let mut map = HashMap::new();
    let rows = stmt.query_map((profile_id, now), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;
    for row in rows {
        let (id, total, due, new_count) = row?;
        map.insert(id, (total, due, new_count));
    }
    Ok(map)
}

pub fn rollup_deck_counts(
    deck_ids: &[String],
    parent_of: &HashMap<String, Option<String>>,
    direct: &HashMap<String, (i64, i64, i64)>,
) -> HashMap<String, (i64, i64, i64)> {
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    for id in deck_ids {
        children_of.entry(id.clone()).or_default();
    }
    for id in deck_ids {
        if let Some(Some(parent)) = parent_of.get(id) {
            children_of
                .entry(parent.clone())
                .or_default()
                .push(id.clone());
        }
    }

    let mut memo: HashMap<String, (i64, i64, i64)> = HashMap::new();

    fn sum_deck(
        id: &str,
        children_of: &HashMap<String, Vec<String>>,
        direct: &HashMap<String, (i64, i64, i64)>,
        memo: &mut HashMap<String, (i64, i64, i64)>,
    ) -> (i64, i64, i64) {
        if let Some(v) = memo.get(id) {
            return *v;
        }
        let mut total = direct.get(id).copied().unwrap_or((0, 0, 0));
        if let Some(children) = children_of.get(id) {
            for child in children {
                let c = sum_deck(child, children_of, direct, memo);
                total.0 += c.0;
                total.1 += c.1;
                total.2 += c.2;
            }
        }
        memo.insert(id.to_string(), total);
        total
    }

    for id in deck_ids {
        sum_deck(id, &children_of, direct, &mut memo);
    }
    memo
}
