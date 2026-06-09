window.BENCHMARK_DATA = {
  "lastUpdate": 1780976587494,
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
      }
    ]
  }
}