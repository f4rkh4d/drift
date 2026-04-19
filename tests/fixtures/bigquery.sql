-- bigquery-shaped queries.
SELECT user_id, COUNT(*) AS hits
FROM `proj.ds.events`
WHERE _PARTITIONDATE BETWEEN '2025-10-01' AND '2025-10-31'
GROUP BY user_id
ORDER BY hits DESC
LIMIT 100;
