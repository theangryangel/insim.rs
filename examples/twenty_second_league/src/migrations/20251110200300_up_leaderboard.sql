CREATE VIEW leaderboard AS
SELECT
  r.event_id, r.player_id, p.pname, p.uname,
  SUM(r.points) AS total_points,
  ROW_NUMBER() OVER (PARTITION BY r.event_id ORDER BY SUM(r.points) DESC) as position
FROM result AS r
INNER JOIN player AS p ON p.id = r.player_id
GROUP BY r.event_id, r.player_id, p.pname, p.uname;

CREATE INDEX idx_result_event_player_points on result(event_id, player_id, points);
