http://127.0.0.1:8001/polymarket_snapshot?sql=SELECT+round%28%28ph.timestamp+-+%28strftime%28%27%25s%27%2C+e.start_time%29+*+1000%29%29+%2F+60000.0%2C+0%29+as+offset_min%2C+round%28avg%28ph.close%29%2C+4%29+as+avg_draw_price%2C+count%28distinct+ph.market_id%29+as+market_count%2C+round%28min%28ph.close%29%2C+4%29+as+min_draw%2C+round%28max%28ph.close%29%2C+4%29+as+max_draw+FROM+price_history+ph+JOIN+polymarket_markets+pm+ON+ph.market_id+%3D+pm.id+JOIN+polymarket_events+e+ON+pm.event_id+%3D+e.id+WHERE+pm.derived_type+%3D+%27draw%27+AND+e.start_time+IS+NOT+NULL+AND+ph.close+%3C%3D+0.5+GROUP+BY+offset_min+HAVING+market_count+%3E%3D+5+ORDER+BY+offset_min#g.mark=line&g.x_column=offset_min&g.x_type=ordinal&g.y_column=avg_draw_price&g.y_type=quantitative

- avg draw price vs offset min (in minutes) for markets with derived_type = 'draw' and at least 5 markets in the same offset_min group, where the close price is less than or equal to 0.5. The offset_min is calculated as the difference between the timestamp of the price history and the start time of the event, converted to minutes. The average draw price is rounded to 4 decimal places, and the minimum and maximum draw prices are also calculated for each offset_min group. The results are ordered by offset_min.

WITH prices AS (
  SELECT ph.market_id,
    round((ph.timestamp - (strftime('%s', e.start_time) * 1000)) / 60000.0, 0) as offset_min,
    ph.close
  FROM price_history ph
  JOIN polymarket_markets pm ON ph.market_id = pm.id
  JOIN polymarket_events e ON pm.event_id = e.id
  WHERE pm.derived_type = 'draw' AND e.start_time IS NOT NULL AND ph.close <= 0.5
),
paired AS (
  SELECT market_id,
    MAX(CASE WHEN offset_min = -1 THEN close END) as pre_1,
    MAX(CASE WHEN offset_min = 0 THEN close END) as kickoff,
    MAX(CASE WHEN offset_min = 5 THEN close END) as post_5,
    MAX(CASE WHEN offset_min = 10 THEN close END) as post_10
  FROM prices
  WHERE offset_min IN (-1, 0, 5, 10)
  GROUP BY market_id
  HAVING pre_1 IS NOT NULL AND kickoff IS NOT NULL
)
SELECT
  COUNT(*) as n_markets,
  ROUND(AVG(kickoff - pre_1), 4) as mean_diff_kickoff_vs_pre,
  ROUND(AVG(post_5 - pre_1), 4) as mean_diff_5min_vs_pre,
  ROUND(AVG(post_10 - pre_1), 4) as mean_diff_10min_vs_pre,
  ROUND(SQRT(AVG((kickoff - pre_1)*(kickoff - pre_1)) - AVG(kickoff - pre_1)*AVG(kickoff - pre_1)), 4) as std_kickoff,
  ROUND(SQRT(AVG((post_5 - pre_1)*(post_5 - pre_1)) - AVG(post_5 - pre_1)*AVG(post_5 - pre_1)), 4) as std_5min,
  ROUND(SQRT(AVG((post_10 - pre_1)*(post_10 - pre_1)) - AVG(post_10 - pre_1)*AVG(post_10 - pre_1)), 4) as std_10min,
  ROUND(AVG(kickoff - pre_1) / (SQRT(AVG((kickoff - pre_1)*(kickoff - pre_1)) - AVG(kickoff - pre_1)*AVG(kickoff - pre_1)) / SQRT(COUNT(*))), 2) as t_stat_kickoff,
  ROUND(AVG(post_5 - pre_1) / (SQRT(AVG((post_5 - pre_1)*(post_5 - pre_1)) - AVG(post_5 - pre_1)*AVG(post_5 - pre_1)) / SQRT(COUNT(*))), 2) as t_stat_5min
FROM paired;

- Resultados com 1438 mercados:
Comparação
Kickoff vs -1min
+5min vs -1min
+10min vs -1min
Conclusão: O preço do empate sobe em média 7.4 pontos percentuais no kickoff — de ~24.3% para 31.7%. O t-statistic de 21.12 é extremamente significativo (p ≈ 0). O efeito dura pelo menos 10 minutos, com o preço ainda 3% acima do pré-jogo.
Isto é evidência estatística forte de que há um spike de preço no draw no momento do kickoff, consistente com a ideia de que a incerteza máxima (e portanto maior probabilidade implícita de empate) ocorre quando o jogo começa.