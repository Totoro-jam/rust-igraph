window.BENCHMARK_DATA = {
  "lastUpdate": 1780980635016,
  "repoUrl": "https://github.com/Totoro-jam/rust-igraph",
  "entries": {
    "rust-igraph Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "8a9fae751fbdce1280aa0c5dd41e6013f0c1ec9e",
          "message": "feat(algo-tr): ALGO-TR-048 irregularity indices (Albertson / sigma / total / variance)\n\nFour irregularity measures that quantify deviation from regularity:\n- albertson_index: Σ|d_u-d_v| over edges\n- sigma_index: Σ(d_u-d_v)² over edges (Gutman)\n- total_irregularity: ½Σ|d_u-d_v| over all vertex pairs\n- degree_variance: Var(degree sequence)\n\n57 unit tests + 4 doctests, all pass. Clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T10:51:09+08:00",
          "tree_id": "5148a9fbcd9e778f6c0ec6073cc227080813ab60",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/8a9fae751fbdce1280aa0c5dd41e6013f0c1ec9e"
        },
        "date": 1780974153161,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 682,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1614,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 16747,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 391107,
            "range": "± 2683",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 14129,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 102913,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1114990,
            "range": "± 10871",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 8223,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 36664,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 186966,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 14789,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 102674,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1097711,
            "range": "± 5347",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 15012,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 111977,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1089063,
            "range": "± 15552",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 30848,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 235495,
            "range": "± 2891",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2217206,
            "range": "± 5382",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 30337,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 229488,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2243352,
            "range": "± 4940",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 1664,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 2873,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 5591,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 716,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1079,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 10752,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 104110,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 696,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1147,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 11166,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 111076,
            "range": "± 5472",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 11145,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 112621,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1231925,
            "range": "± 8311",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 133482,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 5367185,
            "range": "± 6010",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 21649702,
            "range": "± 85921",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 13509,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 131134,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1506108,
            "range": "± 15167",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 10828,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 9029,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 9947,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 20889,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 499,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 5023,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 23888,
            "range": "± 514",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 110251,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 1803,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 1888,
            "range": "± 34",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "179e08ebf4b3c9b62e0d33bd1e2faab7d3d36968",
          "message": "feat(algo-tr): ALGO-TR-049 forgotten index / reduced second Zagreb / modified first Zagreb\n\nThree more degree-based molecular descriptors:\n- forgotten_index: Σ(d_u²+d_v²) = Σd_v³ (Furtula-Gutman 2015)\n- reduced_second_zagreb: Σ(d_u-1)(d_v-1)\n- modified_first_zagreb: Σ 1/d_v² over non-isolated vertices\n\n46 unit tests + 3 doctests, all pass. Clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T10:58:24+08:00",
          "tree_id": "a3af50346de881711a0e3b971da28ad6074246da",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/179e08ebf4b3c9b62e0d33bd1e2faab7d3d36968"
        },
        "date": 1780974641103,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 782,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1879,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 18040,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 483588,
            "range": "± 18485",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18993,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 153805,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1478237,
            "range": "± 43022",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10757,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 42648,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 254352,
            "range": "± 6894",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21882,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 160324,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1460928,
            "range": "± 29738",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21479,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 156521,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1465412,
            "range": "± 28150",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 43523,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 310703,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2941809,
            "range": "± 65863",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 43956,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 306139,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3021033,
            "range": "± 51244",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2371,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4066,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7803,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 786,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1202,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 11684,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 116787,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 761,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1344,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12580,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 127274,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14533,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146400,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1599619,
            "range": "± 7081",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 178062,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 5919425,
            "range": "± 25916",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25631156,
            "range": "± 558067",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16717,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 156086,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1821303,
            "range": "± 13559",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 11416,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 9916,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 11069,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 24015,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 604,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6614,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30264,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 135999,
            "range": "± 1751",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2039,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2412,
            "range": "± 5",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "50aefa9501c0ab279ebbbc9d4c762afa21e37c82",
          "message": "feat(algo-tr): ALGO-TR-050 general Randić / general sum-connectivity / reciprocal Randić\n\nParameterised degree-based topological indices:\n- general_randic_index(G, α): Σ(d_u·d_v)^α (Bollobás-Erdős 1998)\n- general_sum_connectivity_index(G, α): Σ(d_u+d_v)^α\n- reciprocal_randic_index: Σ√(d_u·d_v) = R_{+½}\n\n32 unit tests + 3 doctests, all pass. Clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T11:06:16+08:00",
          "tree_id": "6599558a6bc6b9e8b5ea5e8329c199d12d9248e0",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/50aefa9501c0ab279ebbbc9d4c762afa21e37c82"
        },
        "date": 1780975114813,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 881,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2083,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21551,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 510754,
            "range": "± 3134",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18420,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 144426,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1433155,
            "range": "± 14884",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10918,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 47128,
            "range": "± 1665",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 235499,
            "range": "± 2377",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19182,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 131788,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1403533,
            "range": "± 32023",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19114,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 131432,
            "range": "± 840",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1421535,
            "range": "± 13591",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38426,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 280833,
            "range": "± 906",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2840039,
            "range": "± 8865",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39953,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 285147,
            "range": "± 3924",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2880638,
            "range": "± 11774",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2045,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3680,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7099,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 922,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1397,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13440,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 133553,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 905,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1481,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15548,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144825,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14582,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 147570,
            "range": "± 1525",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1606868,
            "range": "± 13180",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 175945,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6955266,
            "range": "± 14226",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27081491,
            "range": "± 91063",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17218,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167417,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1945917,
            "range": "± 17744",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 14031,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11976,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 13050,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26735,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 626,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6434,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30358,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 141924,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2339,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2359,
            "range": "± 10",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "0f39c0f47ba0e38aba33a108b3dffecdc6761f87",
          "message": "feat(algo-tr): ALGO-TR-051 Zagreb connection indices\n\nAdd first/second/modified-first Zagreb connection indices based on\nsecond-neighbour (distance-2) counts. 46 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T11:19:33+08:00",
          "tree_id": "6b0699d72edc5fbc98a66b87beb0679488042886",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/0f39c0f47ba0e38aba33a108b3dffecdc6761f87"
        },
        "date": 1780975917507,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 899,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2110,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21757,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 512019,
            "range": "± 5832",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18484,
            "range": "± 1891",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 142144,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1432942,
            "range": "± 12546",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10547,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45221,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 239000,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19426,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 131758,
            "range": "± 601",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1433023,
            "range": "± 22885",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19295,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 130774,
            "range": "± 7140",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1417358,
            "range": "± 22509",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 40038,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 283820,
            "range": "± 4694",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2830372,
            "range": "± 26885",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39358,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 260067,
            "range": "± 2514",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2883740,
            "range": "± 7429",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2076,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3683,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7098,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 919,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1397,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13397,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134073,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1488,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14182,
            "range": "± 607",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144041,
            "range": "± 4154",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14401,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 148027,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1603314,
            "range": "± 18814",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174620,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7052241,
            "range": "± 120692",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27453878,
            "range": "± 110674",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17204,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 168646,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1935077,
            "range": "± 12039",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13695,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11286,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12525,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26656,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 626,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6282,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31646,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 144170,
            "range": "± 3875",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2302,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2364,
            "range": "± 28",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "5c01dab40b1f5d1b023e8ea047e0e3fe4d5d8810",
          "message": "feat(algo-tr): ALGO-TR-052 Narumi-Katayama & multiplicative Zagreb indices\n\nAdd narumi_katayama_index, first/second_multiplicative_zagreb —\nproduct-based degree descriptors. 42 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T11:30:47+08:00",
          "tree_id": "8c8560dd4eabe535dc49322d49d822e88f300956",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/5c01dab40b1f5d1b023e8ea047e0e3fe4d5d8810"
        },
        "date": 1780976587027,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2096,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21546,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 512743,
            "range": "± 5239",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18667,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 132336,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1428206,
            "range": "± 27875",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10793,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 46329,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 244535,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19874,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 141258,
            "range": "± 1601",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1409937,
            "range": "± 25621",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19654,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 133762,
            "range": "± 1778",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1425085,
            "range": "± 18157",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 39248,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 268021,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2833281,
            "range": "± 17301",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39118,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 276338,
            "range": "± 1281",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2890512,
            "range": "± 8286",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2133,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3734,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7100,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 918,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1396,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13701,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134132,
            "range": "± 625",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 894,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1450,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14142,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144409,
            "range": "± 723",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14333,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145390,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1592399,
            "range": "± 8324",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 172168,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7002845,
            "range": "± 99647",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27809722,
            "range": "± 601466",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17370,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 169180,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1961715,
            "range": "± 10992",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13525,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11232,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12531,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26695,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 641,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6441,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30829,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 145564,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2302,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2355,
            "range": "± 11",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "c1709ee9b018c87d8bd357170f40558df7987f03",
          "message": "feat(algo-tr): ALGO-TR-053 Schultz index (degree-distance index)\n\nAdd schultz_index — Σ (d(u)+d(v))·dist(u,v) over all vertex pairs.\nUses all-pairs BFS. 22 unit tests + 1 doctest.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T11:41:07+08:00",
          "tree_id": "7c7c32c366a940d9a4bc587fb674079f53197c40",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/c1709ee9b018c87d8bd357170f40558df7987f03"
        },
        "date": 1780977201614,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2109,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21509,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 525707,
            "range": "± 10821",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21677,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 162350,
            "range": "± 1336",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1536066,
            "range": "± 11970",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12179,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50124,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 248541,
            "range": "± 4794",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22951,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164480,
            "range": "± 895",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1547149,
            "range": "± 7188",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22931,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 160510,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1567270,
            "range": "± 33003",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45699,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 328982,
            "range": "± 1813",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3052712,
            "range": "± 15732",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45515,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 322722,
            "range": "± 1754",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3074991,
            "range": "± 12357",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2048,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3673,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7496,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 878,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13780,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 139652,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 888,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1415,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14201,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137755,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14017,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 137941,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1535802,
            "range": "± 6921",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166980,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6344342,
            "range": "± 12277",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24560022,
            "range": "± 32835",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15769,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152620,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1770775,
            "range": "± 13226",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13429,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11077,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12600,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26658,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 633,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6790,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32769,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154709,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2195,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2354,
            "range": "± 9",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "f447e06e9dc1e78da930c2e3b5beffb04e811725",
          "message": "feat(algo-tr): ALGO-TR-054 reformulated Zagreb & third Zagreb index\n\nAdd first/second reformulated Zagreb (edge-degree based) and third\nZagreb index (edge irregularity). 37 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T11:52:35+08:00",
          "tree_id": "42ad8aee3b4c7329354ae29758e444e3d5372cb3",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/f447e06e9dc1e78da930c2e3b5beffb04e811725"
        },
        "date": 1780977881610,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 883,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2079,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 24568,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 518065,
            "range": "± 4544",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18342,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 132621,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1413668,
            "range": "± 7341",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10468,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45114,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 236529,
            "range": "± 5167",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 18878,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 136541,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1415486,
            "range": "± 15556",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19229,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 130504,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1403201,
            "range": "± 9479",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38399,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 294292,
            "range": "± 1737",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2870334,
            "range": "± 7578",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38419,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 283390,
            "range": "± 1117",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2869205,
            "range": "± 10245",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2054,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3788,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7161,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 923,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1404,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13394,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134555,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 890,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1537,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14137,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143658,
            "range": "± 1156",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14300,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145866,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1591294,
            "range": "± 18673",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174652,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7013523,
            "range": "± 15122",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 26927199,
            "range": "± 473309",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17209,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 169638,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1960664,
            "range": "± 10412",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 14256,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11776,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12935,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27050,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 625,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6293,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30883,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 145858,
            "range": "± 1134",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2280,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2379,
            "range": "± 8",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "84edd6639fcbc228551dfe451b8d81346d1a036c",
          "message": "feat(algo-tr): ALGO-TR-055 fourth ABC / fifth GA / degree-sum index\n\nAdd eccentricity-based ABC₄ and GA₅ indices, plus degree-sum index.\n38 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T12:16:30+08:00",
          "tree_id": "e937eb2dd2e986f16000b3d0b6ef746bc8daa2d6",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/84edd6639fcbc228551dfe451b8d81346d1a036c"
        },
        "date": 1780979328294,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 772,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1870,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 18196,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 484804,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19006,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 155617,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1462596,
            "range": "± 34359",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10843,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 42831,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 257693,
            "range": "± 6618",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21870,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162635,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1463309,
            "range": "± 33478",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22088,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 154359,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1460364,
            "range": "± 27717",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44359,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 319846,
            "range": "± 1196",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2895873,
            "range": "± 48899",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44147,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 310677,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2907195,
            "range": "± 56041",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2300,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4055,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7886,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 936,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1212,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 11782,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 117798,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 760,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1377,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12588,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 130044,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14537,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146216,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1601253,
            "range": "± 5237",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174038,
            "range": "± 1312",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 5948419,
            "range": "± 11196",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24779991,
            "range": "± 97441",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16703,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 155190,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1803576,
            "range": "± 13357",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 11426,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 10148,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 10934,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 23897,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 606,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6533,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30746,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 132892,
            "range": "± 1637",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2024,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2411,
            "range": "± 3",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "b952d59e5decbc0e41372cd5f54c21299a0c4457",
          "message": "feat(algo-tr): ALGO-TR-056 inverse degree index + Zagreb coindices\n\nAdd three topological index functions:\n- inverse_degree_index: Σ 1/d(v) (zeroth-order Randić index)\n- first_zagreb_coindex: Σ_{non-edges} (d(u)+d(v)) via identity 2m(n-1)-M₁\n- second_zagreb_coindex: Σ_{non-edges} d(u)·d(v) via algebraic identity\n\n37 unit tests + 3 doctests, all passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T12:30:02+08:00",
          "tree_id": "8e1a26bb1d0db10c21cbca2743e3402d4374dfeb",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/b952d59e5decbc0e41372cd5f54c21299a0c4457"
        },
        "date": 1780980146034,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2133,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21623,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 506305,
            "range": "± 2302",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18325,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 130090,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1419874,
            "range": "± 8236",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10356,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45242,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 238105,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19236,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 133672,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1421676,
            "range": "± 16350",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19801,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 131444,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1417327,
            "range": "± 33801",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45560,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 276073,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2852581,
            "range": "± 9206",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38712,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 266422,
            "range": "± 1457",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2863995,
            "range": "± 10742",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2078,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3770,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7076,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 915,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1400,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13427,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134532,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 890,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1474,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14136,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144038,
            "range": "± 986",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14525,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 148746,
            "range": "± 1409",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1611545,
            "range": "± 19693",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174284,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7061137,
            "range": "± 15457",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 28166919,
            "range": "± 65258",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17272,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167672,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1947489,
            "range": "± 50920",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13527,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11622,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 13171,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26620,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 624,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6748,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30280,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 144761,
            "range": "± 1053",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2295,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2399,
            "range": "± 9",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "committer": {
            "email": "moqiuchen66@gmail.com",
            "name": "Totoro-jam",
            "username": "Totoro-jam"
          },
          "distinct": true,
          "id": "20f451cc3dbf363f8d72a236d41a17a735d83aa0",
          "message": "feat(algo-tr): ALGO-TR-057 Sombor index variants\n\nAdd three Sombor-family topological indices (Gutman 2021):\n- sombor_index: Σ √(d(u)² + d(v)²) over edges\n- reduced_sombor_index: Σ √((d(u)-1)² + (d(v)-1)²) over edges\n- average_sombor_index: SO(G) / m\n\n33 unit tests + 3 doctests, all passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T12:38:20+08:00",
          "tree_id": "5ee75aaae22c429cd488a7ed467fb21f07b71ef6",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/20f451cc3dbf363f8d72a236d41a17a735d83aa0"
        },
        "date": 1780980634000,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 872,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2178,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21632,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 518658,
            "range": "± 3332",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21808,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 165997,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1561988,
            "range": "± 28913",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12149,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 53204,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249043,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22627,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 165834,
            "range": "± 2149",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1547051,
            "range": "± 18376",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22215,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164115,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1530619,
            "range": "± 25580",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 46712,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 330305,
            "range": "± 2071",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3084551,
            "range": "± 15449",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44959,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 325456,
            "range": "± 3577",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3072133,
            "range": "± 25263",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2029,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3695,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 8231,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 857,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1414,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13752,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 140527,
            "range": "± 1116",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 884,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1418,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13455,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 146079,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14341,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138707,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1547467,
            "range": "± 64184",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165169,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6297350,
            "range": "± 24394",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24864889,
            "range": "± 384765",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15959,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153018,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1779520,
            "range": "± 28680",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13553,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11775,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12744,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27184,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 623,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6799,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32539,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157121,
            "range": "± 2123",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2207,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2296,
            "range": "± 13",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}