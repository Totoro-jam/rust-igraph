window.BENCHMARK_DATA = {
  "lastUpdate": 1781017710509,
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
          "id": "c62d8f4fb25fff3274a5157018337ac9a36c94b9",
          "message": "feat(algo-tr): ALGO-TR-058 degree-eccentricity indices\n\nAdd three degree×eccentricity topological indices:\n- lanzhou_index: Σ d(v)² · ε(v)\n- degree_eccentricity_index: Σ d(v) · ε(v)\n- eccentric_distance_sum: Σ ε(v) · D(v)\n\n40 unit tests + 3 doctests, all passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T12:48:14+08:00",
          "tree_id": "a00e48019b6c7ee1896cf46e984e0fa929868b2a",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/c62d8f4fb25fff3274a5157018337ac9a36c94b9"
        },
        "date": 1780981229709,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 886,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2087,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21770,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 527571,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19650,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 133201,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1420761,
            "range": "± 14901",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11146,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45559,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 236063,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19455,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 133727,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1426815,
            "range": "± 12464",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 18994,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 129118,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1402464,
            "range": "± 44475",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38680,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 281786,
            "range": "± 8835",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2849529,
            "range": "± 9835",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38779,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 275649,
            "range": "± 1150",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2845382,
            "range": "± 8007",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2094,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3734,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7137,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 916,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1401,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13557,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134004,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 897,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1461,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14390,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143399,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14309,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145639,
            "range": "± 1375",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1594269,
            "range": "± 8479",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 171855,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6906388,
            "range": "± 11368",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27151867,
            "range": "± 94168",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17093,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167559,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1934990,
            "range": "± 43066",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13698,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11523,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12585,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27113,
            "range": "± 1260",
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
            "value": 6419,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30596,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 142892,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2292,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2386,
            "range": "± 6",
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
          "id": "6df0979f91b9dfdaa8c4951f29485b3db519ed8e",
          "message": "feat(algo-tr): ALGO-TR-059 edge-Szeged / Graovac-Ghorbani indices\n\nAdd three Szeged-family topological indices:\n- edge_szeged_index: Σ m_u(e)·m_v(e) — edge proximity product\n- edge_pi_index: Σ [m_u(e)+m_v(e)] — edge-PI\n- graovac_ghorbani_index: Σ ln(n_u·n_v)/ln(n_u+n_v) — log-ratio index\n\n30 unit tests + 3 doctests, all passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:04:24+08:00",
          "tree_id": "c8ee001ff31678cf63b5d9560adede2c2e4b4e72",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/6df0979f91b9dfdaa8c4951f29485b3db519ed8e"
        },
        "date": 1780982195175,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2134,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21785,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 517726,
            "range": "± 9999",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21600,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163461,
            "range": "± 2494",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1526600,
            "range": "± 17592",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12086,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 49726,
            "range": "± 1223",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250260,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22650,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 160644,
            "range": "± 2017",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1519329,
            "range": "± 26197",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22488,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 168041,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1534735,
            "range": "± 44740",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45721,
            "range": "± 1049",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 327500,
            "range": "± 5335",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3077776,
            "range": "± 19382",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 46195,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 329223,
            "range": "± 1422",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3063738,
            "range": "± 98995",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2061,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3727,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7522,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 878,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1417,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13798,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 143829,
            "range": "± 2537",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1414,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15191,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137362,
            "range": "± 2185",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13979,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 140461,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1561768,
            "range": "± 14812",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166691,
            "range": "± 5740",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6432413,
            "range": "± 39092",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24716523,
            "range": "± 104577",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15877,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152913,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1784032,
            "range": "± 14661",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13687,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11397,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12559,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27387,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 636,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6702,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33287,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157671,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2196,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2275,
            "range": "± 59",
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
          "id": "7a08fdcd36e65a8e0fe4704cdf7149dc541fef28",
          "message": "feat(algo-tr): ALGO-TR-060 hyper-Zagreb and redefined Zagreb indices\n\nAdd first_hyper_zagreb (Σ(d(u)+d(v))²), second_hyper_zagreb\n(Σ(d(u)·d(v))²), and first_redefined_zagreb (Σ(d(u)+d(v))/(d(u)·d(v)))\ntopological indices. Includes identity proof: ReZG₁ equals the count\nof non-isolated vertices. 35 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:14:56+08:00",
          "tree_id": "9a03e579dcba1864ff821c5f08a804363cd46799",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/7a08fdcd36e65a8e0fe4704cdf7149dc541fef28"
        },
        "date": 1780982831671,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 834,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1970,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 20676,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 484718,
            "range": "± 11758",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 20380,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 151470,
            "range": "± 3244",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1462311,
            "range": "± 87233",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11456,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 49335,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 242471,
            "range": "± 5686",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21215,
            "range": "± 518",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 152899,
            "range": "± 15711",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1467029,
            "range": "± 31402",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21108,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 157430,
            "range": "± 4569",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1456199,
            "range": "± 33959",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 43127,
            "range": "± 1057",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 319042,
            "range": "± 7012",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2900119,
            "range": "± 80240",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 42941,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 310496,
            "range": "± 7087",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2920463,
            "range": "± 61362",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 1951,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3497,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7140,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 819,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1338,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13182,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134393,
            "range": "± 2870",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 864,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1354,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12760,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 130118,
            "range": "± 3020",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13161,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 137498,
            "range": "± 2864",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1539317,
            "range": "± 15790",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 164457,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6350752,
            "range": "± 26175",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24670894,
            "range": "± 147418",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16088,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153729,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1804254,
            "range": "± 8487",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13527,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11064,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12397,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26602,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 638,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6656,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32264,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156816,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2217,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2232,
            "range": "± 17",
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
          "id": "134f9f77fedf92cf4566e4db97ade80a1430f224",
          "message": "feat(algo-tr): ALGO-TR-061 Platt, Gordon-Scantlebury, Bertz edge-degree indices\n\nAdd platt_index (Σ(d(u)+d(v)-2)), gordon_scantlebury_index (P₂ path\ncount = Σ C(d(v),2)), and bertz_complexity_index (Σ C(d(u)+d(v)-2,2))\nedge-degree based topological indices. Includes identity proofs:\nPlatt = M₁ - 2m, GS = Platt/2. 42 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:23:14+08:00",
          "tree_id": "909e73b1c896e86dd699aaaca533d0d540871f5d",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/134f9f77fedf92cf4566e4db97ade80a1430f224"
        },
        "date": 1780983326340,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 875,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2127,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 24010,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 515886,
            "range": "± 5264",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21759,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161403,
            "range": "± 3330",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1533561,
            "range": "± 15811",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11898,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50830,
            "range": "± 1320",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249320,
            "range": "± 2445",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22582,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 166677,
            "range": "± 1182",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1545413,
            "range": "± 17596",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22383,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 167061,
            "range": "± 652",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1520694,
            "range": "± 12813",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 46103,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 332089,
            "range": "± 3518",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3075493,
            "range": "± 20009",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 47324,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 322821,
            "range": "± 7607",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3039166,
            "range": "± 18753",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2062,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3706,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7515,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 874,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1410,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13785,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 139663,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1391,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15065,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137890,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13857,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138543,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1548026,
            "range": "± 21505",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165014,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6365519,
            "range": "± 173098",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24837473,
            "range": "± 330115",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15906,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153010,
            "range": "± 1405",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1785492,
            "range": "± 18394",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13458,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11522,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12629,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26840,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 635,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6566,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32862,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155784,
            "range": "± 2643",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2230,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2286,
            "range": "± 15",
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
          "id": "fc05722db7565fbefa663ae021da8beaea3ee5cf",
          "message": "feat(algo-tr): ALGO-TR-062 neighborhood Zagreb indices\n\nAdd first_neighborhood_zagreb (Σ S(v)²), second_neighborhood_zagreb\n(Σ S(u)·S(v) over edges), and neighborhood_forgotten_index (Σ S(v)³)\nwhere S(v) is the sum of neighbor degrees. Includes regular-graph\nformulas and cross-consistency checks. 44 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:32:03+08:00",
          "tree_id": "572c6236ea7ce4525c9265d262c64de56e020ba9",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/fc05722db7565fbefa663ae021da8beaea3ee5cf"
        },
        "date": 1780983859852,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2226,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21792,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 534491,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21518,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161999,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1546805,
            "range": "± 14202",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12175,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 51159,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250998,
            "range": "± 1188",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22107,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163426,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1527561,
            "range": "± 10124",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22749,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 159864,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1536544,
            "range": "± 13911",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45498,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 341151,
            "range": "± 1799",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3061320,
            "range": "± 18228",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44748,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 320582,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3065704,
            "range": "± 16805",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2114,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3897,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7513,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 867,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1407,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 15165,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 136889,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1416,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13531,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 134591,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13802,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139049,
            "range": "± 1261",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540689,
            "range": "± 20322",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165833,
            "range": "± 556",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6334864,
            "range": "± 13730",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24537765,
            "range": "± 39786",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16181,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153086,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1772349,
            "range": "± 62646",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13541,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11455,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 13050,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26738,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 644,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6785,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32953,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 159004,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2263,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2276,
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
          "id": "d0b7260e68cc299b8ee35b405eaf1af93fe10698",
          "message": "feat(algo-tr): ALGO-TR-063 Gourava indices\n\nAdd first_gourava_index (Σ[d(u)+d(v)+d(u)·d(v)]), second_gourava_index\n(Σ(d(u)+d(v))·(d(u)·d(v))), and first_hyper_gourava_index (squared\nfirst Gourava). Includes identity GO₁ = M₁+M₂ and Cauchy-Schwarz\nbound HGO₁·m ≥ GO₁². 42 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:39:17+08:00",
          "tree_id": "493a54cf98680ee7501212523b74fc2c5b41c966",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/d0b7260e68cc299b8ee35b405eaf1af93fe10698"
        },
        "date": 1780984303472,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2093,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21697,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 522515,
            "range": "± 7205",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18702,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 138058,
            "range": "± 2339",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1406679,
            "range": "± 18054",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10712,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45250,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 236236,
            "range": "± 4650",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19424,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 128780,
            "range": "± 1256",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1411751,
            "range": "± 24592",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19160,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 130275,
            "range": "± 1065",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1387653,
            "range": "± 47134",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 39290,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 273669,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2818634,
            "range": "± 8935",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38933,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 294025,
            "range": "± 1303",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2814071,
            "range": "± 8700",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2060,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3711,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7063,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 923,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1391,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13418,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134366,
            "range": "± 2531",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 894,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1477,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14136,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143137,
            "range": "± 922",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14430,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146088,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1589896,
            "range": "± 10002",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174052,
            "range": "± 2628",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7008362,
            "range": "± 230335",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27376193,
            "range": "± 142805",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17189,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 169111,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1947803,
            "range": "± 15701",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13636,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11497,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12665,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26865,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 625,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6335,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30736,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 145981,
            "range": "± 1539",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2292,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2356,
            "range": "± 13",
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
          "id": "a0702e910d18674953f264b010c82738071bfaa2",
          "message": "feat(algo-tr): ALGO-TR-064 Nirmala indices\n\nAdd nirmala_index (Σ√(d(u)+d(v))), first_inverse_nirmala (Σ1/√(d(u)+d(v))),\nand second_inverse_nirmala (Σ1/√(d(u)·d(v))) — the latter equals the\nRandić index. Includes Cauchy-Schwarz bound N·IN₁ ≥ m² and regular-graph\nformulas. 38 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T13:46:51+08:00",
          "tree_id": "82d79b14743b551251cd71d19ed90601752e9d2c",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/a0702e910d18674953f264b010c82738071bfaa2"
        },
        "date": 1780984748645,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2074,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21902,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 543089,
            "range": "± 7672",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21605,
            "range": "± 1416",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163359,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1517259,
            "range": "± 45774",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12112,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 49989,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 248032,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22362,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163701,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1521200,
            "range": "± 14640",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22711,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 161580,
            "range": "± 4969",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1512464,
            "range": "± 13768",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45997,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 321293,
            "range": "± 1617",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3032876,
            "range": "± 15094",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45477,
            "range": "± 607",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 326773,
            "range": "± 3155",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3045754,
            "range": "± 10158",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2079,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3693,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7534,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 871,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1408,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13772,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137211,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1450,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13478,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 138970,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13824,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139051,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1547906,
            "range": "± 15410",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167049,
            "range": "± 1457",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6356116,
            "range": "± 19676",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24983609,
            "range": "± 113352",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16165,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154242,
            "range": "± 1442",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1793620,
            "range": "± 14519",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13481,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11210,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12511,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26607,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 624,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6655,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32564,
            "range": "± 1007",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157413,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2208,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2283,
            "range": "± 19",
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
          "id": "b62490f46dffb023136a0795b260c1374355d8bb",
          "message": "feat(algo-tr): ALGO-TR-065 transmission Zagreb indices\n\nAdd first_transmission_zagreb (Σ σ(v)²), second_transmission_zagreb\n(Σ σ(u)·σ(v) over edges), and reciprocal_transmission_index (Σ 1/σ(v))\nwhere σ(v) is vertex transmission (distance sum). O(V²) BFS for all\ntransmissions. 40 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T14:01:46+08:00",
          "tree_id": "8fbeca1596bddaef915c1c7987a23c8022409d25",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/b62490f46dffb023136a0795b260c1374355d8bb"
        },
        "date": 1780985642010,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 787,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1872,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 18318,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 485243,
            "range": "± 1266",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19056,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 153794,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1475168,
            "range": "± 42735",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10938,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 42718,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 256120,
            "range": "± 8413",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21769,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 152950,
            "range": "± 2425",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1520855,
            "range": "± 63350",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21606,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 156108,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1508010,
            "range": "± 36298",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 43686,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 318277,
            "range": "± 3245",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2931254,
            "range": "± 40633",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 47856,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 309967,
            "range": "± 5272",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3053128,
            "range": "± 61367",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2293,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4034,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7716,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 784,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1202,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 11692,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 117187,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 762,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1376,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12808,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 127640,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14542,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146330,
            "range": "± 1391",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1602679,
            "range": "± 8861",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 179126,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 5884875,
            "range": "± 15038",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24900766,
            "range": "± 234893",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16698,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 155024,
            "range": "± 719",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1799281,
            "range": "± 17008",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 11436,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 9821,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 10937,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 24298,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 605,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6598,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30766,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 132996,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2031,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2409,
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
          "id": "e553d2665c3d4483cd5d5021748d7c2f5ea63c77",
          "message": "fix(algo-tr): clippy comparison_chain + cast_lossless + unnecessary_wraps\n\nRewrite three if/else-if chains in szeged_edge.rs to match/cmp patterns\nto satisfy clippy::comparison_chain on Rust 1.85 CI. Also fix cast_lossless\nin transmission_zagreb.rs test and allow unnecessary_wraps for the private\nvertex_transmissions helper.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T14:16:06+08:00",
          "tree_id": "a025d56943bbf5a6791587a03fb3f840dbfb1919",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/e553d2665c3d4483cd5d5021748d7c2f5ea63c77"
        },
        "date": 1780986508388,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 909,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2170,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22610,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 524119,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18334,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 132550,
            "range": "± 2317",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1413337,
            "range": "± 26247",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10354,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45298,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 233612,
            "range": "± 1025",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19342,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 132456,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1409080,
            "range": "± 23814",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19229,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 134464,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1416842,
            "range": "± 16641",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 39217,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 285092,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2828067,
            "range": "± 28964",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 40058,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 265276,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2851936,
            "range": "± 9311",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2046,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3712,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7079,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 954,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1394,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13382,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134091,
            "range": "± 2196",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 894,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1474,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14165,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 145851,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14486,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 147532,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1598857,
            "range": "± 11165",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 173688,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6962761,
            "range": "± 14336",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 28293174,
            "range": "± 97600",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17051,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 169797,
            "range": "± 1015",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1931685,
            "range": "± 14440",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13580,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11545,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12962,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27156,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 629,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6448,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30271,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 142856,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2301,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2431,
            "range": "± 7",
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
          "id": "bf4b24b908068480984a5d392c2b72806156ab61",
          "message": "feat(algo-tr): ALGO-TR-066 leap Zagreb indices (first/second/third)\n\nd₂(v)-based indices using the count of vertices at distance exactly 2.\nLM₁ = Σ d₂(v)², LM₂ = Σ_{edges} d₂(u)·d₂(v), LM₃ = Σ d(v)·d₂(v).\n42 unit tests + 3 doctests covering paths, cycles, stars, Petersen, etc.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T14:29:37+08:00",
          "tree_id": "788dcab1964518c87bf6663ee687328b4722a8b1",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/bf4b24b908068480984a5d392c2b72806156ab61"
        },
        "date": 1780987316145,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 883,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2140,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21533,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 504731,
            "range": "± 1370",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19224,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 141118,
            "range": "± 12474",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1420780,
            "range": "± 17570",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10681,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45828,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 233117,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 20157,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 136465,
            "range": "± 2067",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1433080,
            "range": "± 11234",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19390,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 135736,
            "range": "± 1277",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1430965,
            "range": "± 19294",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 39606,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 265023,
            "range": "± 2009",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2958607,
            "range": "± 8426",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38984,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 279534,
            "range": "± 2985",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2863277,
            "range": "± 10987",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2065,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3726,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7108,
            "range": "± 26",
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
            "value": 1399,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13416,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134510,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 895,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1478,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15125,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144278,
            "range": "± 1148",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14288,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146513,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1595583,
            "range": "± 20941",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174036,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6945477,
            "range": "± 26359",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27353630,
            "range": "± 86446",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17062,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167283,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1944003,
            "range": "± 16676",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13540,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11359,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12633,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26901,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 670,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6490,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31127,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 144785,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2277,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2412,
            "range": "± 51",
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
          "id": "40452a73e564f4aa9849651bb6165fe451e65b63",
          "message": "feat(algo-tr): ALGO-TR-067 reciprocal distance-degree indices\n\nRDD (additively weighted Harary), H_M (multiplicatively weighted Harary),\nand terminal Wiener index (pendant-only distances). 38 unit tests + 3\ndoctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T14:50:57+08:00",
          "tree_id": "0792b1bb5139770fddcb30d258d3adba45c4f933",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/40452a73e564f4aa9849651bb6165fe451e65b63"
        },
        "date": 1780988600774,
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
            "value": 2088,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21610,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 518721,
            "range": "± 6273",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 20964,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 147889,
            "range": "± 628",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1440489,
            "range": "± 10424",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11151,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 47402,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 236341,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22435,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 135870,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1451332,
            "range": "± 13409",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22288,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 138406,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1435414,
            "range": "± 15213",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 40732,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 278057,
            "range": "± 1674",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2842821,
            "range": "± 8246",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39498,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 281360,
            "range": "± 1485",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2814404,
            "range": "± 7570",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2798,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4902,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 9145,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 920,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1398,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13444,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 135057,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 888,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1475,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14523,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 145128,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14319,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146911,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1592190,
            "range": "± 6549",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174315,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6903694,
            "range": "± 11103",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27202124,
            "range": "± 89398",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17353,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167162,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1948738,
            "range": "± 7133",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13703,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11626,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12749,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26695,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 719,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6403,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30543,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 147242,
            "range": "± 991",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2368,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2410,
            "range": "± 7",
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
          "id": "50793e1daf1628ffed3dc422d640fe79c1408ecc",
          "message": "feat(algo-tr): ALGO-TR-068 ve-degree Zagreb indices (alpha/beta/second)\n\nVertex-edge degree d_ve(v) = d²-2d+S(v) based indices. 40 unit tests +\n3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T15:12:26+08:00",
          "tree_id": "1ac11829b571b47ec704b0044ceb361fc98e163a",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/50793e1daf1628ffed3dc422d640fe79c1408ecc"
        },
        "date": 1780989876351,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 873,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2089,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21416,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 542035,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21972,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161083,
            "range": "± 1402",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1536241,
            "range": "± 15364",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11979,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50397,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249909,
            "range": "± 1229",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22639,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 165777,
            "range": "± 1044",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1528212,
            "range": "± 7286",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22172,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164212,
            "range": "± 1945",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1522210,
            "range": "± 10650",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45139,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 325541,
            "range": "± 2266",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3049687,
            "range": "± 25234",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45141,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 323278,
            "range": "± 8411",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3063310,
            "range": "± 11228",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2109,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3974,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7594,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 884,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1410,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13760,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137037,
            "range": "± 1108",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 888,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1417,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13446,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 138577,
            "range": "± 842",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13851,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138389,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1543535,
            "range": "± 38508",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166149,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6326436,
            "range": "± 32885",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24677169,
            "range": "± 91233",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16000,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152341,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1785007,
            "range": "± 12438",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13565,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11592,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12687,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26990,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 640,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6824,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32961,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156564,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2204,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2265,
            "range": "± 14",
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
          "id": "255e6082614092caebaa9e5544a2ec3e04ed6e75",
          "message": "feat(algo-tr): ALGO-TR-069 ev-degree indices (first/second Zagreb + Randić)\n\nEdge-vertex degree d_ev(e) = d(u)+d(v)-2 based topological indices:\n- first_ev_degree_zagreb: sum of squared ev-degrees\n- second_ev_degree_zagreb: product over adjacent edge pairs\n- ev_degree_randic: Randić-like index over adjacent edge pairs\n\n38 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T15:44:20+08:00",
          "tree_id": "f93c13be5c92661e2775cd47bdf087e9f3a8d90b",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/255e6082614092caebaa9e5544a2ec3e04ed6e75"
        },
        "date": 1780991799017,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 878,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2080,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22231,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 512998,
            "range": "± 5069",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21467,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 160006,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1529746,
            "range": "± 8706",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12090,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50804,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249437,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22474,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164310,
            "range": "± 980",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1524747,
            "range": "± 10927",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22192,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162061,
            "range": "± 5689",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1528381,
            "range": "± 21014",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45644,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 327131,
            "range": "± 1367",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3060492,
            "range": "± 17907",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44638,
            "range": "± 938",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 332777,
            "range": "± 2178",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3048333,
            "range": "± 6838",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2113,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3879,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7525,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 877,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1413,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13784,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137422,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1443,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13438,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 133914,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13848,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138609,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1539888,
            "range": "± 16962",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 164801,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6359000,
            "range": "± 24529",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24486889,
            "range": "± 573316",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15962,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152455,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1774907,
            "range": "± 23533",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13492,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11207,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12468,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26723,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 637,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6758,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33642,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157543,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2205,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2263,
            "range": "± 21",
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
          "id": "3061b8d2903cfb94fcf1c93b6567f05e6a4f19e5",
          "message": "feat(algo-tr): ALGO-TR-070 Sombor variants (elliptic, modified, coindex)\n\nThree extensions of the Sombor index family:\n- elliptic_sombor_index: (du+dv)√(du²+dv²) over edges\n- modified_sombor_index: 1/√(du²+dv²) over edges\n- sombor_coindex: √(du²+dv²) over non-adjacent pairs\n\n35 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T15:53:11+08:00",
          "tree_id": "986836e28b5e088f4ba2baa5e09e7584f49a543c",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/3061b8d2903cfb94fcf1c93b6567f05e6a4f19e5"
        },
        "date": 1780992329804,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2263,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21963,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 544167,
            "range": "± 2243",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21203,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161200,
            "range": "± 6639",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1544074,
            "range": "± 11516",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12160,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 51102,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250926,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22748,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 160123,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1531330,
            "range": "± 6829",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22201,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 161224,
            "range": "± 3643",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1551153,
            "range": "± 18051",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45532,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 319187,
            "range": "± 1805",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3076184,
            "range": "± 10824",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44764,
            "range": "± 3529",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 331398,
            "range": "± 1885",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3061849,
            "range": "± 9234",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2116,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3892,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7535,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 866,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1414,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 14526,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 141434,
            "range": "± 2216",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 885,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1415,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13459,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 146834,
            "range": "± 1005",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13987,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 140891,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1538193,
            "range": "± 15391",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165901,
            "range": "± 818",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6324818,
            "range": "± 22444",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24563016,
            "range": "± 54111",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16015,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153606,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1774537,
            "range": "± 55546",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13445,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11302,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12551,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26456,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 623,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6644,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32420,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 153741,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2224,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2266,
            "range": "± 13",
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
          "id": "473d61e8ab14d4c42e13f777932b9f3cb87cbfdd",
          "message": "feat(algo-tr): ALGO-TR-071 forgotten/hyper-Zagreb coindices\n\nComplement-edge coindices summing over non-adjacent vertex pairs:\n- forgotten_coindex: Σ [d(u)²+d(v)²] over non-edges\n- first_hyper_zagreb_coindex: Σ [d(u)+d(v)]² over non-edges\n- second_hyper_zagreb_coindex: Σ [d(u)·d(v)]² over non-edges\n\n37 unit tests + 3 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:03:23+08:00",
          "tree_id": "57a548502ff5d5364601e8d50c75554a0071b881",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/473d61e8ab14d4c42e13f777932b9f3cb87cbfdd"
        },
        "date": 1780992947252,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 874,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2083,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21695,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 530524,
            "range": "± 10184",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21706,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163536,
            "range": "± 843",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1545641,
            "range": "± 45098",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12097,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50963,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 251072,
            "range": "± 18090",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21851,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164680,
            "range": "± 3195",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1533432,
            "range": "± 32790",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22169,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 159933,
            "range": "± 6370",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1542301,
            "range": "± 25946",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44482,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 328947,
            "range": "± 3504",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3056584,
            "range": "± 22348",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44924,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 323709,
            "range": "± 2578",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3079155,
            "range": "± 25870",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2028,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3700,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7614,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 858,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1413,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13770,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 139801,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 886,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1414,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13497,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 148696,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13879,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138051,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540956,
            "range": "± 9207",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165958,
            "range": "± 604",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6332194,
            "range": "± 144142",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24628974,
            "range": "± 223272",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15983,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152295,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1781953,
            "range": "± 24772",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13425,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11148,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12624,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26492,
            "range": "± 719",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 651,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6718,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33546,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 158899,
            "range": "± 2756",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2213,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2294,
            "range": "± 16",
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
          "id": "668e2161c578a32830ea253bf64faa7b0acc7d26",
          "message": "feat(algo-tr): ALGO-TR-072 arithmetic-geometric index, sigma coindex, albertson coindex\n\nBond-additive degree-sum variants over edges and complement edges:\n- arithmetic_geometric_index: AM/GM ratio of endpoint degrees\n- sigma_coindex: squared degree differences over non-edges\n- albertson_coindex: absolute degree differences over non-edges\n\n36 unit tests + 3 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:19:26+08:00",
          "tree_id": "7eb15e593e8429845251889c5cbbe17ad3968d85",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/668e2161c578a32830ea253bf64faa7b0acc7d26"
        },
        "date": 1780993911860,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 883,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2108,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21737,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 520647,
            "range": "± 1999",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18613,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 128523,
            "range": "± 1700",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1425837,
            "range": "± 16333",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10441,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45430,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 233812,
            "range": "± 879",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19788,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 134099,
            "range": "± 2295",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1407168,
            "range": "± 14111",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19159,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 131477,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1409362,
            "range": "± 12692",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38800,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 285113,
            "range": "± 1873",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2819659,
            "range": "± 14286",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39124,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 282855,
            "range": "± 1671",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2860090,
            "range": "± 6937",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2062,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3704,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7113,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 909,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1395,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13404,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 132883,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 932,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1963,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15412,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144745,
            "range": "± 2718",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14433,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 147994,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1612359,
            "range": "± 20249",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 173559,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6951348,
            "range": "± 35202",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 28283859,
            "range": "± 150644",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16878,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167798,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1929408,
            "range": "± 8981",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13435,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11267,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12551,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26746,
            "range": "± 88",
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
            "value": 6539,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31389,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 143311,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2281,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2387,
            "range": "± 36",
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
          "id": "6ea00b6a378f2777c43507fd1cee535e06a486ee",
          "message": "feat(algo-tr): ALGO-TR-073 exponential degree-based indices\n\nExponential versions of classical bond-additive topological indices:\n- exponential_augmented_zagreb: exp(du·dv/(du+dv-2)) over edges\n- exponential_randic: exp(1/√(du·dv)) over edges\n- exponential_abc: exp(√((du+dv-2)/(du·dv))) over edges\n- exponential_ga: exp(2√(du·dv)/(du+dv)) over edges\n\n44 unit tests + 4 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:29:52+08:00",
          "tree_id": "5a8ccee3c50368ab7e2519ecfd1c8f9f9ddb765a",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/6ea00b6a378f2777c43507fd1cee535e06a486ee"
        },
        "date": 1780994532476,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 869,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2155,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 24618,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 537505,
            "range": "± 5948",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21550,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 159778,
            "range": "± 3731",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1545412,
            "range": "± 13772",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12147,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 51385,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249028,
            "range": "± 3418",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22395,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 161035,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1552804,
            "range": "± 23118",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22032,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 165175,
            "range": "± 2006",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1540604,
            "range": "± 17190",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45771,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 322076,
            "range": "± 2516",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3078421,
            "range": "± 33446",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45708,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 340315,
            "range": "± 8070",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3077084,
            "range": "± 64755",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2112,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3893,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7508,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 857,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1413,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13746,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 142342,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1416,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13478,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 140716,
            "range": "± 2528",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13830,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138499,
            "range": "± 652",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1539222,
            "range": "± 31718",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166399,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6338197,
            "range": "± 46485",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24449335,
            "range": "± 434703",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16061,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154162,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1792621,
            "range": "± 28953",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13588,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11473,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12721,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26916,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 635,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6787,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32654,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156609,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2267,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2257,
            "range": "± 14",
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
          "id": "cfe0cd19e64953266dca1c0cef0756e674466716",
          "message": "feat(algo-tr): ALGO-TR-074 multiplicative connectivity indices\n\nProduct-based versions of classical bond-additive indices using\nlog-sum accumulation to avoid overflow:\n- multiplicative_sum_connectivity: Π 1/√(du+dv) over edges\n- multiplicative_randic: Π 1/√(du·dv) over edges\n- multiplicative_abc: Π √((du+dv-2)/(du·dv)) over edges\n- multiplicative_ga: Π 2√(du·dv)/(du+dv) over edges\n\n40 unit tests + 4 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:38:56+08:00",
          "tree_id": "3882e1034e7ee0d2d4c6a9521e4d9922db558f5e",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/cfe0cd19e64953266dca1c0cef0756e674466716"
        },
        "date": 1780995071513,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 890,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2129,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22173,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 506293,
            "range": "± 6095",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21223,
            "range": "± 912",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163922,
            "range": "± 6701",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1532045,
            "range": "± 33035",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12300,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50815,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249101,
            "range": "± 1416",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22704,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 161496,
            "range": "± 5764",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1557881,
            "range": "± 102522",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22297,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164109,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1518798,
            "range": "± 33462",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45037,
            "range": "± 2845",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 323997,
            "range": "± 4078",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3073498,
            "range": "± 20866",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44952,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 323585,
            "range": "± 7143",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3064279,
            "range": "± 6974",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2087,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3697,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7574,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 859,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1414,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13849,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137103,
            "range": "± 3756",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1391,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13438,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 134626,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13975,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139796,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1557671,
            "range": "± 37599",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167720,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6401221,
            "range": "± 157664",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24977175,
            "range": "± 105482",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16047,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153728,
            "range": "± 1376",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1781781,
            "range": "± 15662",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13482,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11282,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12655,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26690,
            "range": "± 1224",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 625,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6542,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32022,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155962,
            "range": "± 1359",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2200,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2299,
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
          "id": "18d4bd6d25405f9ed321fe3ee6dc18d22d41a736",
          "message": "feat(algo-tr): ALGO-TR-075 reduced degree-based indices\n\nIndices using (d(v)-1) reduced degrees instead of d(v):\n- reduced_reciprocal_randic: 1/√((du-1)(dv-1)) over edges\n- reduced_sum_connectivity: 1/√(du+dv-2) over edges\n- reduced_first_zagreb: Σ(d(v)-1)² over vertices\n- reduced_forgotten_index: Σ(d(v)-1)³ over vertices\n\n41 unit tests + 4 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:48:52+08:00",
          "tree_id": "f72d684dee789332b3ac821ca928510a0c9165d3",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/18d4bd6d25405f9ed321fe3ee6dc18d22d41a736"
        },
        "date": 1780995672176,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2071,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21769,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 526152,
            "range": "± 1256",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21945,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 166903,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1544651,
            "range": "± 13403",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11987,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50780,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249137,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22781,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 160507,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1535641,
            "range": "± 7688",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22449,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162809,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1523797,
            "range": "± 5922",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45058,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 330547,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3066078,
            "range": "± 10038",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45078,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 335019,
            "range": "± 4126",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3058752,
            "range": "± 20145",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2032,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3704,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7499,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 879,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1411,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13753,
            "range": "± 601",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 144263,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1414,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13431,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 146990,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13933,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138988,
            "range": "± 4607",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1554488,
            "range": "± 17364",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 169454,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6327179,
            "range": "± 14044",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24742745,
            "range": "± 35281",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16015,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 163656,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1817646,
            "range": "± 14978",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13478,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11246,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12544,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26674,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 651,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6748,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32427,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155122,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2204,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2261,
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
          "id": "7956b5f5fa5dae1c43408efba97f338a974980ea",
          "message": "feat(algo-tr): ALGO-TR-076 entropy-based topological indices\n\nShannon entropy of edge-weight distributions induced by classical\ntopological indices:\n- first_zagreb_entropy: H of (du+dv) weights\n- second_zagreb_entropy: H of (du·dv) weights\n- randic_entropy: H of 1/√(du·dv) weights\n- abc_entropy: H of √((du+dv-2)/(du·dv)) weights\n\n37 unit tests + 4 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T16:59:16+08:00",
          "tree_id": "0c6688c68b4161f9035599a419ede5a9dddc7ba3",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/7956b5f5fa5dae1c43408efba97f338a974980ea"
        },
        "date": 1780996297280,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 886,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2161,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21624,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 537440,
            "range": "± 10780",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21540,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161729,
            "range": "± 6913",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1517615,
            "range": "± 17790",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12300,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50722,
            "range": "± 1507",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249396,
            "range": "± 9941",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22447,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163867,
            "range": "± 2083",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1516313,
            "range": "± 18872",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22058,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162736,
            "range": "± 3709",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1539273,
            "range": "± 32231",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45599,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 328179,
            "range": "± 3636",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3033349,
            "range": "± 124839",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44847,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 325723,
            "range": "± 4675",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3066911,
            "range": "± 155958",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2046,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3735,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7514,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 871,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13805,
            "range": "± 858",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 138138,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 890,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1425,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13449,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 140632,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13827,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 137669,
            "range": "± 2095",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1546791,
            "range": "± 29069",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165071,
            "range": "± 16826",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6374557,
            "range": "± 17225",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24768780,
            "range": "± 555178",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16055,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152168,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1807225,
            "range": "± 29177",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13392,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11314,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12632,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26878,
            "range": "± 1143",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 632,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 7065,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32629,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 159257,
            "range": "± 2566",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2274,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2285,
            "range": "± 63",
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
          "id": "d52bbc54c02357316de6ca3a08cc9133f3675a5e",
          "message": "feat(algo-tr): ALGO-TR-077 degree-power indices (general zeroth-order Randić, variable sum exdeg, inverse degree power, variable first Zagreb)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T17:08:44+08:00",
          "tree_id": "30c84b79386a67d35affc467372e839949fe060b",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/d52bbc54c02357316de6ca3a08cc9133f3675a5e"
        },
        "date": 1780996853596,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2171,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21522,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 528010,
            "range": "± 7020",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21882,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 164646,
            "range": "± 1675",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1524962,
            "range": "± 21147",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12056,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 49900,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 252141,
            "range": "± 2519",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22534,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162283,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1546607,
            "range": "± 18228",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22396,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164490,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1533809,
            "range": "± 18541",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45662,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 331402,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3071933,
            "range": "± 13740",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45427,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 332237,
            "range": "± 5206",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3065048,
            "range": "± 9603",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2034,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3705,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7538,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 880,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1410,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 14598,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 136920,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 894,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1418,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13506,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 135176,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13822,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139115,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540707,
            "range": "± 12652",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165572,
            "range": "± 2230",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6324506,
            "range": "± 20301",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24612819,
            "range": "± 71261",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15913,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152608,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1774124,
            "range": "± 12718",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13467,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11431,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12652,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26547,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 639,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6648,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32332,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156559,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2215,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2269,
            "range": "± 15",
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
          "id": "0bb6928b953e2b32d67baa1f68e896b5009d3543",
          "message": "feat(algo-tr): ALGO-TR-078 extended irregularity (Bell, Collatz-Sinogowitz, IRL, IRLU, degree CV)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T17:16:39+08:00",
          "tree_id": "1c1f4f7bc987a5e270682bc63862d38172035998",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/0bb6928b953e2b32d67baa1f68e896b5009d3543"
        },
        "date": 1780997340533,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2073,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22493,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 533703,
            "range": "± 2498",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21576,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 164608,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1514024,
            "range": "± 22220",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11942,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50726,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 256695,
            "range": "± 7736",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22181,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164317,
            "range": "± 783",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1522479,
            "range": "± 13547",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22728,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163366,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1531866,
            "range": "± 28747",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45301,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 329046,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3044136,
            "range": "± 16568",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45848,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 328974,
            "range": "± 2560",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3056242,
            "range": "± 76187",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2100,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3879,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7543,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 877,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1410,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13734,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 140879,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 885,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1416,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13451,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 138218,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14128,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 141485,
            "range": "± 1347",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1566943,
            "range": "± 30577",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 170199,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6373911,
            "range": "± 19551",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25505917,
            "range": "± 191775",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16245,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 157673,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1799066,
            "range": "± 12161",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13365,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11395,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12522,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26757,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 631,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6495,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33361,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154429,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2225,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2241,
            "range": "± 12",
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
          "id": "232b10b54180907398c88b3e4365050d653fcb97",
          "message": "feat(algo-tr): ALGO-TR-079 edge irregularity indices (IRD, IRA, IRB, IRGA)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T17:27:27+08:00",
          "tree_id": "d9f11a484598f5fff88805e9b73462ddbe5dc3fc",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/232b10b54180907398c88b3e4365050d653fcb97"
        },
        "date": 1780997990117,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 876,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2066,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22020,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 525158,
            "range": "± 3113",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21796,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 168327,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1539211,
            "range": "± 15165",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12138,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 51245,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 248948,
            "range": "± 4062",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22484,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 161911,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1531508,
            "range": "± 7877",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22797,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164653,
            "range": "± 1424",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1535670,
            "range": "± 17232",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45316,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 333477,
            "range": "± 2292",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3082400,
            "range": "± 74206",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45450,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 329605,
            "range": "± 1917",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3076557,
            "range": "± 71104",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2105,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3885,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7551,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 867,
            "range": "± 27",
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
            "value": 13762,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137626,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 886,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1418,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13484,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 134516,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13868,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139014,
            "range": "± 1767",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1542250,
            "range": "± 28925",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165382,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6374379,
            "range": "± 14893",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24614247,
            "range": "± 47557",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16098,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153086,
            "range": "± 4605",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1788732,
            "range": "± 7341",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13568,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11047,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12402,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26910,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 690,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6884,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32337,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157106,
            "range": "± 1206",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2204,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2246,
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
          "id": "ef12b13aee0e8585faf4d0a7b90bb0929578e5ba",
          "message": "fix(algo-tr): fix clippy similar_names and dead_code warnings in TR-077/078/079\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T17:39:45+08:00",
          "tree_id": "364c0642d8132bd6b40f949a643b93c78e1f3f4c",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/ef12b13aee0e8585faf4d0a7b90bb0929578e5ba"
        },
        "date": 1780998735040,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 881,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2113,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21957,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 506517,
            "range": "± 4397",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18406,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 133971,
            "range": "± 1914",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1408404,
            "range": "± 23060",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10434,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45648,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 237446,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19610,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 133883,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1432172,
            "range": "± 10743",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19039,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 130980,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1460539,
            "range": "± 8102",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38763,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 296684,
            "range": "± 1504",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2830531,
            "range": "± 8590",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39147,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 262348,
            "range": "± 1765",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2892601,
            "range": "± 15329",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2062,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3708,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7087,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 1099,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1393,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13422,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 135310,
            "range": "± 2427",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 891,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1474,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14145,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143819,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14301,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146984,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1599863,
            "range": "± 18144",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 173646,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6988553,
            "range": "± 19205",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27096021,
            "range": "± 130568",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17273,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 169040,
            "range": "± 2630",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1957883,
            "range": "± 14793",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13718,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11521,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12616,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26402,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 623,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6362,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31405,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 143874,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2300,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2358,
            "range": "± 20",
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
          "id": "6bcb68a42410c320bf6eafeec21eadebea32fffa",
          "message": "feat(algo-tr): ALGO-TR-080 exponential vertex indices (eM₁, eF, eID, eSC)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T17:52:36+08:00",
          "tree_id": "eca8340043d2a8f3fd08f89b64fb8453a75c0a44",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/6bcb68a42410c320bf6eafeec21eadebea32fffa"
        },
        "date": 1780999494875,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 885,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2096,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 24993,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 504335,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18375,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 131388,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1420027,
            "range": "± 30309",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10464,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45878,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 237125,
            "range": "± 11888",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19302,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 129335,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1424829,
            "range": "± 22361",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19131,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 129270,
            "range": "± 2247",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1455464,
            "range": "± 31856",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 39072,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 285389,
            "range": "± 8564",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2847320,
            "range": "± 52173",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38601,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 282578,
            "range": "± 1830",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2831614,
            "range": "± 14743",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2088,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3743,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7171,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 922,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1398,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13417,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134723,
            "range": "± 425",
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
            "value": 1480,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14143,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144767,
            "range": "± 1893",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14254,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145468,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1600899,
            "range": "± 30502",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 173172,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6992881,
            "range": "± 102940",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27215355,
            "range": "± 453657",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17029,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167587,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1947849,
            "range": "± 14563",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 14888,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 12248,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 14094,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 28288,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 648,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6489,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31114,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 143048,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2280,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2385,
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
          "id": "ee75aa136bd7fa18c44a5e8a8521da6f44335ce0",
          "message": "feat(algo-tr): ALGO-TR-081 degree-ratio indices (SR, min-max, DHM, DDC)\n\nAdd four bond-additive indices based on degree ratios:\n- symmetric_degree_ratio: Σ (d(u)/d(v) + d(v)/d(u))\n- minmax_degree_ratio: Σ min(d(u),d(v))/max(d(u),d(v))\n- degree_harmonic_mean_index: Σ 2·d(u)·d(v)/(d(u)+d(v))\n- degree_diff_connectivity: Σ 1/√(|d(u)-d(v)|+1)\n\n32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T18:05:05+08:00",
          "tree_id": "35105e78dd0d17330c0968998566b10443cfa4a0",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/ee75aa136bd7fa18c44a5e8a8521da6f44335ce0"
        },
        "date": 1781000242752,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2226,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21962,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 514719,
            "range": "± 4593",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21587,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 160369,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1527808,
            "range": "± 28756",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12003,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50492,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 256095,
            "range": "± 1583",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22451,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 159683,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1545153,
            "range": "± 29537",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22435,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 161222,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1533693,
            "range": "± 10038",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 46003,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 327693,
            "range": "± 1667",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3033506,
            "range": "± 19346",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45114,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 327587,
            "range": "± 1675",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3071579,
            "range": "± 23089",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2107,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3805,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7551,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 863,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1521,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13751,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 139643,
            "range": "± 909",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 886,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1420,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15139,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137427,
            "range": "± 1205",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14001,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 140328,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1560346,
            "range": "± 18679",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167218,
            "range": "± 2277",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6441053,
            "range": "± 25731",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25248044,
            "range": "± 162787",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15973,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154446,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1820111,
            "range": "± 20115",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 14263,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11127,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 13166,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27732,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 635,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6840,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32832,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157261,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2248,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2321,
            "range": "± 15",
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
          "id": "5c7bd879ef8afcd6b156321972773aa60761ce12",
          "message": "feat(algo-tr): ALGO-TR-082 degree distribution moments (skewness, kurtosis, Gini, max-dev)\n\nAdd four statistical measures of the degree sequence:\n- degree_skewness: third standardized moment (asymmetry)\n- degree_kurtosis: excess kurtosis (tailedness vs normal)\n- degree_gini: Gini coefficient of degree inequality\n- degree_max_deviation: max |d(v) - d̄|\n\n32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T18:16:27+08:00",
          "tree_id": "1269f37c7467aca90c202eeaedb09bceb9bfc08a",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/5c7bd879ef8afcd6b156321972773aa60761ce12"
        },
        "date": 1781000921269,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2079,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22373,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 525561,
            "range": "± 6293",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21619,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 165048,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1523618,
            "range": "± 18693",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12217,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50546,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 251108,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22395,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164354,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1517620,
            "range": "± 69124",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22046,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163538,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1531586,
            "range": "± 12698",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45678,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 319753,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3072979,
            "range": "± 7256",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45100,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 324512,
            "range": "± 5160",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3069888,
            "range": "± 19344",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2081,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3700,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7477,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 875,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1415,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 14588,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137038,
            "range": "± 8442",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 884,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1413,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14863,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137503,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13793,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138247,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1539880,
            "range": "± 17592",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166667,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6280954,
            "range": "± 12851",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25341584,
            "range": "± 107142",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16061,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152397,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1785246,
            "range": "± 22879",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13670,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11217,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12616,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26937,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 648,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6648,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32065,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155424,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2194,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2321,
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
          "id": "7a31591abe7d1b1c80881e504e4b79acf01d72b6",
          "message": "feat(algo-tr): ALGO-TR-083 degree spread measures (range, span ratio, median, IQR)\n\nAdd four descriptive statistics of the degree sequence:\n- degree_range: d_max - d_min\n- degree_span_ratio: range normalized by mean degree\n- degree_median: median of sorted degree sequence\n- degree_iqr: interquartile range (Q3 - Q1)\n\n34 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T18:29:36+08:00",
          "tree_id": "313a35e8c2f76dd593244e88c8194c9bbb674935",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/7a31591abe7d1b1c80881e504e4b79acf01d72b6"
        },
        "date": 1781001714203,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2114,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21608,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 507440,
            "range": "± 4034",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21572,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 164291,
            "range": "± 2769",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1538735,
            "range": "± 21052",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12139,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50180,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250435,
            "range": "± 991",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22362,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163252,
            "range": "± 1005",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1522876,
            "range": "± 77681",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22760,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162537,
            "range": "± 1955",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1525583,
            "range": "± 15455",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 46003,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 334801,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3037272,
            "range": "± 19798",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45197,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 331398,
            "range": "± 1491",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3059174,
            "range": "± 21945",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2055,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3695,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7484,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 873,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 14531,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137359,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 895,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1416,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15129,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 136610,
            "range": "± 1389",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13864,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138371,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540442,
            "range": "± 13193",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165720,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6357259,
            "range": "± 25729",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25361486,
            "range": "± 138906",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16044,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154030,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1788474,
            "range": "± 14280",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13456,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11274,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12639,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26817,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 632,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6632,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33357,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156231,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2300,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2317,
            "range": "± 16",
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
          "id": "6ad37d3cd2e48df901219fe94af54ed9e813f4e7",
          "message": "feat(algo-tr): ALGO-TR-084 degree deviation measures (MAD, MedAD, entropy)\n\nAdd four robust dispersion and entropy measures:\n- degree_mad: mean absolute deviation of degrees\n- degree_median_ad: median absolute deviation of degrees\n- degree_entropy_ln: Shannon entropy with natural log\n- degree_entropy_normalized: entropy normalized to [0,1]\n\n32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T18:43:35+08:00",
          "tree_id": "fadbe54dba102f3cbc0b61008ee4f6d228de6077",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/6ad37d3cd2e48df901219fe94af54ed9e813f4e7"
        },
        "date": 1781002555250,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 780,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1899,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 17873,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 485236,
            "range": "± 2039",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19055,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 154087,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1473966,
            "range": "± 42581",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10686,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 42974,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 258242,
            "range": "± 7245",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22188,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 155497,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1472859,
            "range": "± 41642",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21806,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 153093,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1449678,
            "range": "± 24331",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44166,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 321501,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2915152,
            "range": "± 42505",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 43320,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 322658,
            "range": "± 4109",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2937750,
            "range": "± 46924",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2415,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4166,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7888,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 787,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1255,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 12157,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 121966,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 765,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1376,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12646,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 128901,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14559,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145808,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1602298,
            "range": "± 23447",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 177806,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 5945664,
            "range": "± 12946",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25211483,
            "range": "± 350326",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16602,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154906,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1803575,
            "range": "± 34422",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 11407,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 9944,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 11130,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 23751,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 604,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6279,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30302,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 135306,
            "range": "± 2707",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2038,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2428,
            "range": "± 7",
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
          "id": "d2a51624662a3f7c64983f9daa80390030fce21e",
          "message": "feat(algo-tr): ALGO-TR-085 degree distribution shape (mode, concentration, diversity, hub dominance)\n\nAdd four shape descriptors of the degree distribution:\n- degree_mode: most frequent degree value\n- degree_concentration: fraction of vertices with mode degree\n- degree_diversity: count of distinct degree values\n- hub_dominance: max-degree share of total degree\n\n32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T19:13:52+08:00",
          "tree_id": "d67d4105b4f715c6a1ac709a190311db306ed4af",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/d2a51624662a3f7c64983f9daa80390030fce21e"
        },
        "date": 1781004368839,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 989,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2209,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 23754,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 542082,
            "range": "± 3286",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21691,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163460,
            "range": "± 1186",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1538441,
            "range": "± 9744",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12064,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 51761,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 248934,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22478,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162289,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1539512,
            "range": "± 24741",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22655,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 159084,
            "range": "± 2022",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1531938,
            "range": "± 16756",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45824,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 331595,
            "range": "± 1348",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3054361,
            "range": "± 7717",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45675,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 326454,
            "range": "± 2164",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3073378,
            "range": "± 8141",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2100,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3872,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7530,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 873,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13836,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 142489,
            "range": "± 888",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 885,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1419,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13434,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 134171,
            "range": "± 3817",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13878,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138156,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540675,
            "range": "± 26534",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167218,
            "range": "± 2229",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6286224,
            "range": "± 18183",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24605225,
            "range": "± 203088",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15890,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152920,
            "range": "± 1887",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1811173,
            "range": "± 6655",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13642,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11367,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12548,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26395,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 648,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 7006,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32940,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157182,
            "range": "± 447",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2245,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2325,
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
          "id": "051b373e1a47c14b20bf6810f4af2a4480c26a81",
          "message": "feat(algo-tr): ALGO-TR-086 edge degree-pair aggregates\n\nAdd edge_degree_min_sum, edge_degree_max_sum, edge_degree_log_product,\nedge_degree_mean_sum — simple edge-additive indices aggregating min, max,\nlog-product, and mean of endpoint degrees. 30 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T19:23:36+08:00",
          "tree_id": "18330ba31fccac2858100f03816c8aa72ade4842",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/051b373e1a47c14b20bf6810f4af2a4480c26a81"
        },
        "date": 1781004948021,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 891,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2136,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 23020,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 526840,
            "range": "± 1708",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21501,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161131,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1534892,
            "range": "± 30479",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12011,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50576,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249068,
            "range": "± 2022",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 23158,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163018,
            "range": "± 1808",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1525637,
            "range": "± 10345",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22460,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163835,
            "range": "± 948",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1527385,
            "range": "± 9387",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45716,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 340682,
            "range": "± 1394",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3100443,
            "range": "± 41648",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44947,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 328374,
            "range": "± 5125",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3055704,
            "range": "± 12587",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2047,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3711,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7508,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 860,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1410,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13768,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 143505,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1420,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13504,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 136025,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13919,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139791,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1553967,
            "range": "± 33150",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167633,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6347712,
            "range": "± 15650",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25713377,
            "range": "± 138834",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15928,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152615,
            "range": "± 827",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1788916,
            "range": "± 16374",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13472,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11559,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12868,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26571,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 635,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6859,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32457,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157038,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2207,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2271,
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
          "id": "f797e5ee0151f5d96f1e6f486609ffd6327c97f1",
          "message": "feat(algo-tr): ALGO-TR-087 edge degree mean-type indices\n\nAdd edge_degree_harmonic_sum, edge_degree_geometric_sum,\nedge_degree_ratio_sum, edge_degree_rms — harmonic, geometric, ratio,\nand RMS aggregates of endpoint degrees per edge. 32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T19:33:25+08:00",
          "tree_id": "5658297af3130114925636a40370b67a27bc1642",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/f797e5ee0151f5d96f1e6f486609ffd6327c97f1"
        },
        "date": 1781005542020,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 846,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 1954,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 18084,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 486265,
            "range": "± 1911",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18958,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 153948,
            "range": "± 691",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1458344,
            "range": "± 35977",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10760,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 42633,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 254438,
            "range": "± 5703",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22012,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 155736,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1469658,
            "range": "± 29147",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21416,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 154673,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1457628,
            "range": "± 29738",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 43834,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 309648,
            "range": "± 895",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2955545,
            "range": "± 80338",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45520,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 318911,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2914229,
            "range": "± 52620",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2295,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4023,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7669,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 845,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1197,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 11664,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 117274,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 766,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1355,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 12803,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 127354,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14587,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 142963,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1611285,
            "range": "± 27854",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 178093,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6097528,
            "range": "± 25073",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24690110,
            "range": "± 158372",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16619,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 155178,
            "range": "± 1368",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1798774,
            "range": "± 8187",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 11450,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 10111,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 11047,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 23759,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 603,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6436,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30850,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 135064,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2025,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2407,
            "range": "± 7",
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
          "id": "a2515f1d174c67e01ddfbc69191f9ce747d7bc6b",
          "message": "feat(algo-tr): ALGO-TR-088 degree vertex class ratios\n\nAdd degree_leaf_ratio, degree_isolated_ratio, degree_core_ratio,\ndegree_tail_ratio — fractions of vertex set in degree-based classes\n(pendants, isolates, above-mean core, heavy-tail). 30 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T19:42:29+08:00",
          "tree_id": "85331aa4742014940f9928060e9975253bd21f00",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/a2515f1d174c67e01ddfbc69191f9ce747d7bc6b"
        },
        "date": 1781006083206,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 868,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2154,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21769,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 537186,
            "range": "± 7700",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21533,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 165812,
            "range": "± 2437",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1525905,
            "range": "± 16997",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12066,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50773,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249105,
            "range": "± 6292",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22616,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162890,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1522013,
            "range": "± 9781",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22708,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 164644,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1521827,
            "range": "± 7340",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45331,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 326082,
            "range": "± 956",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3065773,
            "range": "± 6261",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44792,
            "range": "± 1346",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 331111,
            "range": "± 2076",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3078869,
            "range": "± 8681",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2044,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3711,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7505,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 872,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1421,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13768,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137354,
            "range": "± 1399",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 896,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1416,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13439,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 145586,
            "range": "± 927",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 15056,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139253,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1548024,
            "range": "± 27422",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165815,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6355665,
            "range": "± 16619",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24275766,
            "range": "± 89452",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16005,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153022,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1786901,
            "range": "± 17386",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13356,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11312,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12661,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26364,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 626,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6713,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32300,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154781,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2238,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2280,
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
          "id": "c65143a23c8dd64ef81d2e0dd557b0052c3038ef",
          "message": "feat(algo-tr): ALGO-TR-089 vertex neighbor degree statistics\n\nAdd degree_neighbor_max_sum, degree_neighbor_min_sum,\ndegree_neighbor_range_sum, degree_neighbor_variance_sum — per-vertex\naggregates of neighbor degree distributions. 32 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T19:51:02+08:00",
          "tree_id": "c3d080049442699c070d1ac78a62ef11646cad55",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/c65143a23c8dd64ef81d2e0dd557b0052c3038ef"
        },
        "date": 1781006618788,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 882,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2066,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21973,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 526888,
            "range": "± 1805",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21723,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 161570,
            "range": "± 5883",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1527317,
            "range": "± 16837",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12193,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50821,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 249476,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22184,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164305,
            "range": "± 4897",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1542492,
            "range": "± 30789",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22315,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163440,
            "range": "± 2249",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1539845,
            "range": "± 26165",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44339,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 332163,
            "range": "± 1489",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3080327,
            "range": "± 15285",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 46543,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 333237,
            "range": "± 1531",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3060894,
            "range": "± 18413",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2043,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3682,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7504,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 870,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1408,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13752,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 138208,
            "range": "± 1287",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 889,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1410,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13458,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 136558,
            "range": "± 1586",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13883,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 137803,
            "range": "± 1659",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1546963,
            "range": "± 18831",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165302,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6351964,
            "range": "± 150343",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24941288,
            "range": "± 155853",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16108,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 157329,
            "range": "± 2952",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1810913,
            "range": "± 22120",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13576,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 13384,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12753,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26890,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 654,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6963,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32828,
            "range": "± 582",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 156228,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2240,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2304,
            "range": "± 99",
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
          "id": "89cb06dd544024dfae45c843aae477e2ff601eb3",
          "message": "feat(algo-tr): ALGO-TR-090 edge degree normalized indices\n\nFour edge-level indices that normalize endpoint degrees:\n- edge_inverse_degree_sum: Σ 1/(d(u)+d(v))\n- edge_degree_diff_ratio: Σ |d(u)-d(v)|/(d(u)+d(v))\n- edge_degree_sorensen: Σ 2·min(d(u),d(v))/(d(u)+d(v))\n- edge_degree_product_ratio: Σ d(u)·d(v)/(d(u)+d(v))²\n\n30 unit tests + 4 doctests. Cross-consistency: sorensen + diff = m.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T20:02:49+08:00",
          "tree_id": "5575187deb6ded46c5091593a8ce6dd46dfe6dab",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/89cb06dd544024dfae45c843aae477e2ff601eb3"
        },
        "date": 1781007294792,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 1029,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2163,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21594,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 537940,
            "range": "± 4799",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 23386,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 167902,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1577360,
            "range": "± 55014",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12772,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 52445,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 258927,
            "range": "± 6239",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 24099,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 167139,
            "range": "± 2652",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1573607,
            "range": "± 25361",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 24095,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 170285,
            "range": "± 2278",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1571002,
            "range": "± 14022",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 47673,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 333433,
            "range": "± 2965",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3130611,
            "range": "± 40231",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 46610,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 339080,
            "range": "± 2503",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3151299,
            "range": "± 23703",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2097,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3843,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7763,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 879,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1440,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13748,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 141972,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 888,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1422,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 15054,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 140764,
            "range": "± 2178",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14270,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 141693,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1582543,
            "range": "± 15370",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 175006,
            "range": "± 1591",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6480521,
            "range": "± 39946",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25243970,
            "range": "± 133874",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16413,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 158331,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1818804,
            "range": "± 26979",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13867,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11451,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12459,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26788,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 638,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6701,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32268,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155698,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2337,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2254,
            "range": "± 14",
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
          "id": "13a9470e89c1fcbd5cdb6a04b2dc9fe44d1f2f09",
          "message": "feat(algo-tr): ALGO-TR-091 degree inequality indices\n\nFour degree inequality/concentration measures:\n- degree_herfindahl: Σ(d/Σd)² concentration index\n- degree_theil: (1/n)Σ(d/d̄)·ln(d/d̄) generalized entropy\n- degree_palma: top 10% / bottom 40% degree share ratio\n- degree_hoover: Σ|d-d̄|/(2·Σd) Robin Hood index\n\n33 unit tests + 4 doctests. All zero for regular graphs.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T20:19:54+08:00",
          "tree_id": "bc126c27debfedf8cc313e762cfb0832eb176f8d",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/13a9470e89c1fcbd5cdb6a04b2dc9fe44d1f2f09"
        },
        "date": 1781008337060,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2081,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21641,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 520517,
            "range": "± 3136",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21720,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 168934,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1512129,
            "range": "± 19635",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12283,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 49952,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250917,
            "range": "± 2584",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22279,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164075,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1517107,
            "range": "± 11164",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22420,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162823,
            "range": "± 1004",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1507857,
            "range": "± 18448",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44293,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 334760,
            "range": "± 9009",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3056216,
            "range": "± 19427",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45434,
            "range": "± 2293",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 332605,
            "range": "± 1313",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3033462,
            "range": "± 16187",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2104,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3893,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7561,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 872,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1418,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13761,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 139922,
            "range": "± 1111",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 942,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1418,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13430,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 138996,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13817,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138917,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1540613,
            "range": "± 17230",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 164614,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6333903,
            "range": "± 24315",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25241008,
            "range": "± 124421",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16052,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153835,
            "range": "± 2570",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1790414,
            "range": "± 17299",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13567,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11179,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12473,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26750,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 629,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6693,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32404,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154802,
            "range": "± 535",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2187,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2273,
            "range": "± 42",
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
          "id": "0e5a69037c9aaf7a81e75e1e9d903fa184c811fa",
          "message": "feat(algo-tr): ALGO-TR-092 edge neighborhood overlap aggregates\n\nFour edge-level neighborhood overlap indices summed over all edges:\n- edge_common_neighbor_sum: Σ|N(u)∩N(v)\\{u,v}|\n- edge_jaccard_sum: Σ|∩|/|∪| Jaccard similarity\n- edge_overlap_sum: Σ|∩|/min(d-1) overlap coefficient\n- edge_adamic_adar_sum: Σ Σ_{w∈CN} 1/ln(d(w))\n\n36 unit tests + 4 doctests. Perfect overlap (=m) for complete graphs.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T20:43:16+08:00",
          "tree_id": "c2cb54b7c9fa0ca9d1e2a52e9ad2613bc85c41d8",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/0e5a69037c9aaf7a81e75e1e9d903fa184c811fa"
        },
        "date": 1781009745353,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 878,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2079,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22147,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 544983,
            "range": "± 6278",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21719,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 164322,
            "range": "± 1039",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1529830,
            "range": "± 9408",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11985,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50382,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 247192,
            "range": "± 3762",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22645,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164162,
            "range": "± 830",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1520335,
            "range": "± 10495",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22386,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 162282,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1538936,
            "range": "± 19645",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44872,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 336956,
            "range": "± 2056",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3065189,
            "range": "± 58159",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44667,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 320043,
            "range": "± 1673",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3062778,
            "range": "± 23626",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2027,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3700,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7481,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 874,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1412,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13806,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 143272,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 892,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1422,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13559,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137653,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13809,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138317,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1547003,
            "range": "± 7571",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166037,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6381692,
            "range": "± 37604",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25043901,
            "range": "± 136315",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15941,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153000,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1787286,
            "range": "± 14543",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13479,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11359,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12657,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26685,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 626,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6763,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32169,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 155597,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2347,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2467,
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
          "id": "b7448382b5e9ce01787001c20cb9626179ea2d88",
          "message": "feat(algo-tr): ALGO-TR-093 edge degree correlation indices\n\nFour edge-level degree correlation measures:\n- edge_degree_covariance: Cov(d(u),d(v)) over edges\n- edge_degree_pearson: Pearson r of endpoint degree pairs\n- edge_degree_cosine: cosine similarity of degree vectors\n- edge_degree_discrepancy: Σ(d(u)-d(v))²/(4m) normalized\n\n29 unit tests + 4 doctests. All zero for regular graphs.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T20:57:29+08:00",
          "tree_id": "edf0c4bb5131bf843f6c02be9af9af1587631e9d",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/b7448382b5e9ce01787001c20cb9626179ea2d88"
        },
        "date": 1781010591645,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 878,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2100,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22089,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 500950,
            "range": "± 3614",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21538,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 162869,
            "range": "± 3773",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1527316,
            "range": "± 25654",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11942,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50892,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 252727,
            "range": "± 2199",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22173,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162421,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1544383,
            "range": "± 40743",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22724,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 167800,
            "range": "± 1886",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1529259,
            "range": "± 62795",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44828,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 328522,
            "range": "± 1780",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3045048,
            "range": "± 25460",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45325,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 324062,
            "range": "± 1971",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3078564,
            "range": "± 17971",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2146,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3915,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7550,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 858,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1421,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13771,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 142459,
            "range": "± 1527",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 975,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1429,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13489,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 135090,
            "range": "± 2401",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14063,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139646,
            "range": "± 1014",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1559714,
            "range": "± 24980",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 168196,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6475930,
            "range": "± 43763",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24667061,
            "range": "± 140670",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16001,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 155648,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1805661,
            "range": "± 20686",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13747,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11427,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12751,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27188,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 625,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6763,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33133,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157344,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2292,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2226,
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
          "id": "9a857093a1b6a363d5a1bb884df59db835c491d6",
          "message": "feat(algo-tr): ALGO-TR-094 graph density profile indices\n\nFour novel graph density measures: triangle_density (fraction of\npossible triangles), square_density (chordless 4-cycle density),\nedge_connectivity_ratio (2m/n(n-1)), degree_density (normalized\nsecond moment of degree). 37 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T21:19:18+08:00",
          "tree_id": "6ed3f8f9448f9c4b17de6221cfc74f15ee022d76",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/9a857093a1b6a363d5a1bb884df59db835c491d6"
        },
        "date": 1781011899499,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2087,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21912,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 520315,
            "range": "± 2900",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 22044,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 163831,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1530657,
            "range": "± 18365",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11977,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50056,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250512,
            "range": "± 690",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22946,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 162134,
            "range": "± 1072",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1530275,
            "range": "± 18111",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22688,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163052,
            "range": "± 1624",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1521204,
            "range": "± 63772",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45643,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 323455,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3059795,
            "range": "± 10418",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45666,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 330560,
            "range": "± 1103",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3062548,
            "range": "± 8063",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2038,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3719,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7619,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 859,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13750,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137124,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1415,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14167,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 153480,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13860,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138823,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1542683,
            "range": "± 6295",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 166221,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6298937,
            "range": "± 25490",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 25349596,
            "range": "± 141937",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16097,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 154137,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1795443,
            "range": "± 9983",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13479,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11278,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12558,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26835,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 693,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6913,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32744,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154747,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2209,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2313,
            "range": "± 33",
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
          "id": "df9cd369a7c4c8aa8d5022341e3cbb2d8e1b29dc",
          "message": "feat(algo-tr): ALGO-TR-095 walk diversity indices\n\nFour novel walk-based measures: walk_entropy (Shannon entropy of degree\ndistribution as walk probability), walk_regularity (1 - CV of degrees),\ndegree_laplacian_energy (normalized mean absolute deviation from mean\ndegree), avg_neighbor_connectivity (mean neighbor degree ratio per\nvertex). 42 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T21:28:51+08:00",
          "tree_id": "0388b89e88e0c84df20549c75f09899188433e23",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/df9cd369a7c4c8aa8d5022341e3cbb2d8e1b29dc"
        },
        "date": 1781012472371,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 870,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2096,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21730,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 521081,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21971,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 162487,
            "range": "± 2837",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1531343,
            "range": "± 22929",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12278,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50546,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250110,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 23254,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 159730,
            "range": "± 632",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1538889,
            "range": "± 17637",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22608,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 160235,
            "range": "± 762",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1529724,
            "range": "± 7468",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45149,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 329077,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3073310,
            "range": "± 64745",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45733,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 329903,
            "range": "± 2431",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3065887,
            "range": "± 27859",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2045,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3731,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7597,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 877,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1413,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13780,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137147,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 887,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1418,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13441,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144168,
            "range": "± 11279",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13838,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138361,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1542904,
            "range": "± 16201",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 167820,
            "range": "± 635",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6323711,
            "range": "± 30234",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24702437,
            "range": "± 78150",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15968,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153226,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1778163,
            "range": "± 9094",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13441,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11169,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12577,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26964,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 647,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6734,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32430,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 157749,
            "range": "± 3130",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2254,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2329,
            "range": "± 12",
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
          "id": "bb241d58198273447543a67d41cd7690bb8f14b3",
          "message": "feat(algo-tr): ALGO-TR-096 graph connectivity ratio indices\n\nFour novel connectivity measures: circuit_rank_ratio (cyclomatic number\nnormalized by edges), meshedness_coefficient (circuit rank / max planar),\nedge_surplus_ratio (surplus edges / max surplus), connectivity_index\n(avg degree / max possible). Uses BFS component counting for circuit\nrank computation. 44 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T21:49:42+08:00",
          "tree_id": "4ed95ac24cee5ece949d0b23a081306538f7a430",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/bb241d58198273447543a67d41cd7690bb8f14b3"
        },
        "date": 1781013728062,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 887,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2191,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 22965,
            "range": "± 1022",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 532680,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21554,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 162957,
            "range": "± 2903",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1534089,
            "range": "± 11583",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12002,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50598,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 252260,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22590,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 163476,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1534114,
            "range": "± 40033",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22461,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 159317,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1528736,
            "range": "± 27468",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 45748,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 329534,
            "range": "± 3338",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3051664,
            "range": "± 67031",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 45725,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 337879,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3086203,
            "range": "± 79170",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2040,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3725,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7501,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 871,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1413,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13748,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 141377,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 895,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1415,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13442,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 138588,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13838,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 138354,
            "range": "± 2660",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1544797,
            "range": "± 25576",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 165599,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6396848,
            "range": "± 39533",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 26115997,
            "range": "± 734178",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 16146,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 152119,
            "range": "± 1004",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1785840,
            "range": "± 24548",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13382,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11220,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12468,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26383,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 641,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6893,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 32484,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154896,
            "range": "± 6395",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2206,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2298,
            "range": "± 40",
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
          "id": "a7e9edb523880e552cfb9ecc6b9b82543c1d044e",
          "message": "feat(algo-tr): ALGO-TR-097 subgraph ratios (pendant, bridge, triangle participation, isolated)\n\nFour substructure density measures: pendant edge ratio (fraction of\nedges incident to degree-1 vertices), bridge ratio (Tarjan bridge-finding),\ntriangle participation (fraction of vertices in at least one triangle),\nand isolated vertex ratio. 40 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T22:12:52+08:00",
          "tree_id": "851bc78789ed53776ab5f0e6540a65a4e51e50fe",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/a7e9edb523880e552cfb9ecc6b9b82543c1d044e"
        },
        "date": 1781015118604,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 880,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2111,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21663,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 503036,
            "range": "± 6117",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18300,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 131265,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1425480,
            "range": "± 16156",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10357,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 46377,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 241492,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19247,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 129090,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1474802,
            "range": "± 5145",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19258,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 127650,
            "range": "± 1904",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1436860,
            "range": "± 19000",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38473,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 272578,
            "range": "± 2398",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2879584,
            "range": "± 13458",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39428,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 277974,
            "range": "± 3879",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2854623,
            "range": "± 30125",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2048,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3676,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7146,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 913,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1395,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13388,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134152,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 893,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1472,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14076,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143027,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14327,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 145753,
            "range": "± 1821",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1591429,
            "range": "± 27704",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 174265,
            "range": "± 2285",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 7026097,
            "range": "± 65256",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27937809,
            "range": "± 77430",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17242,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167357,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1930305,
            "range": "± 37803",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13674,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11395,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12596,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26759,
            "range": "± 148",
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
            "value": 6317,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30262,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 144444,
            "range": "± 1466",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2277,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2367,
            "range": "± 24",
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
          "id": "f470740b8d061c8f5997e843be9e03c61f5f4fe8",
          "message": "feat(algo-tr): ALGO-TR-098 edge density ratios (self-loop, multi-edge, reciprocity, clustering)\n\nFour edge-level density measures: self-loop ratio, multi-edge ratio,\nreciprocity ratio (directed), and average local clustering coefficient.\n38 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T22:23:40+08:00",
          "tree_id": "534ae3d30ff9092dcce1f3ff7a0a4516578822eb",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/f470740b8d061c8f5997e843be9e03c61f5f4fe8"
        },
        "date": 1781015768037,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 871,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2151,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21849,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 508606,
            "range": "± 2628",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 21578,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 162696,
            "range": "± 835",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1547421,
            "range": "± 22433",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 12084,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 50926,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 250821,
            "range": "± 585",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 22447,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 164053,
            "range": "± 591",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1531525,
            "range": "± 16810",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 22571,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 163085,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1539954,
            "range": "± 9266",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 44925,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 327342,
            "range": "± 1495",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 3065366,
            "range": "± 27903",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 44603,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 332315,
            "range": "± 3915",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 3057404,
            "range": "± 14153",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2084,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3906,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7514,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 869,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1420,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13769,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 137262,
            "range": "± 1426",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 889,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1420,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 13458,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 137339,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 13978,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 139890,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1550307,
            "range": "± 17436",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 170161,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6316791,
            "range": "± 15448",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 24346343,
            "range": "± 92534",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 15990,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 153745,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1785182,
            "range": "± 13556",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13382,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11273,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12596,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26416,
            "range": "± 69",
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
            "value": 6599,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 33136,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 154605,
            "range": "± 1015",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2252,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2308,
            "range": "± 21",
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
          "id": "31e35b880d64d2e2f9b9577076a72a48a5ce911a",
          "message": "feat(algo-tr): ALGO-TR-099 spectral ratios (gap estimate, variance, edge-vertex, cyclomatic)\n\nFour spectral-inspired degree-based measures: degree spectral gap\nestimate (max-second/max), degree variance ratio (normalized), edge-vertex\nratio (m/n), and cyclomatic density (circuit rank/n).\n44 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T22:35:57+08:00",
          "tree_id": "dc798a71ae26a92fdc115b853d7f22c82f007a93",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/31e35b880d64d2e2f9b9577076a72a48a5ce911a"
        },
        "date": 1781016504180,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 883,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2105,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21830,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 505185,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 20821,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 135161,
            "range": "± 1001",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1438552,
            "range": "± 9504",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 11025,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 46538,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 233937,
            "range": "± 590",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 21482,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 135170,
            "range": "± 942",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1428358,
            "range": "± 32626",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 21984,
            "range": "± 1944",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 143979,
            "range": "± 2033",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1478124,
            "range": "± 19809",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 41057,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 283978,
            "range": "± 5010",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2839037,
            "range": "± 68568",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 38341,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 262133,
            "range": "± 2025",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2854182,
            "range": "± 7717",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2554,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 4430,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 8155,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 920,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1409,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13476,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134550,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 897,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1476,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14171,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 143271,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14436,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 147195,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1590072,
            "range": "± 20227",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 173322,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6996334,
            "range": "± 109497",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27134912,
            "range": "± 147093",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17073,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167256,
            "range": "± 2543",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1933410,
            "range": "± 23721",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13733,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11654,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12667,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26794,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 624,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6263,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30740,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 146458,
            "range": "± 3467",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2292,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2367,
            "range": "± 6",
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
          "id": "d3f714254ae795af42145f50b4813993b7918ce8",
          "message": "feat(algo-tr): ALGO-TR-100 neighborhood density (degree ratio, hub, leaf-hub, centralization)\n\nFour neighborhood density measures: average neighbor degree ratio,\nhub ratio (fraction above average degree), leaf-to-hub ratio, and\nFreeman's degree centralization. 38 unit tests + 4 doctests.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T22:44:48+08:00",
          "tree_id": "fe31462de9c6d376787b4bc4b64d413b16566699",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/d3f714254ae795af42145f50b4813993b7918ce8"
        },
        "date": 1781017036839,
        "tool": "cargo",
        "benches": [
          {
            "name": "bfs/karate (34v 78e)",
            "value": 881,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/100",
            "value": 2122,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21554,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 510495,
            "range": "± 12275",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 19499,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 133210,
            "range": "± 3834",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1442756,
            "range": "± 16816",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10431,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45263,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 233035,
            "range": "± 1605",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19156,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 135747,
            "range": "± 2367",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1421869,
            "range": "± 31849",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19461,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 132170,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1435359,
            "range": "± 7305",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38765,
            "range": "± 2723",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 271756,
            "range": "± 5120",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2851677,
            "range": "± 19441",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39340,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 277531,
            "range": "± 3659",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2897971,
            "range": "± 14449",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2056,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3667,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7080,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 913,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1395,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13401,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 133715,
            "range": "± 3218",
            "unit": "ns/iter"
          },
          {
            "name": "distances/karate (34v 78e)",
            "value": 895,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/100",
            "value": 1479,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14142,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 159447,
            "range": "± 1313",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14441,
            "range": "± 1367",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146310,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1605522,
            "range": "± 35946",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 175978,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6950613,
            "range": "± 20625",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27279200,
            "range": "± 820453",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17126,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 167881,
            "range": "± 2860",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1946053,
            "range": "± 8971",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13682,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11308,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12667,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 27670,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 632,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6327,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 30475,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 142044,
            "range": "± 632",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2265,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2379,
            "range": "± 49",
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
          "id": "ee2d441bb47fdaf552fd57c5ab26cac287e62589",
          "message": "feat(algo-tr): ALGO-TR-101 path ratios (diameter_radius_ratio, avg_path_fraction, efficiency_ratio, graph_compactness)\n\nPath-based structural ratio indices computed via all-pairs BFS:\n- diameter_radius_ratio: diameter / radius for connected graphs\n- avg_path_fraction: average shortest-path length / diameter\n- efficiency_ratio: global efficiency (avg inverse distance)\n- graph_compactness: 1 - avg_dist / diameter\n\n37 unit tests + 4 doctests, clippy clean.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-09T22:55:52+08:00",
          "tree_id": "8325970c26d0ddb4220e828df33af2b3ff54a157",
          "url": "https://github.com/Totoro-jam/rust-igraph/commit/ee2d441bb47fdaf552fd57c5ab26cac287e62589"
        },
        "date": 1781017709516,
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
            "value": 2122,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/1000",
            "value": 21648,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "bfs/synthetic/10000",
            "value": 533514,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/8",
            "value": 18267,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/16",
            "value": 131432,
            "range": "± 802",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/cycle/32",
            "value": 1414067,
            "range": "± 16389",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/16",
            "value": 10383,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/32",
            "value": 45973,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "canonical_permutation/path/64",
            "value": 234720,
            "range": "± 985",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/8",
            "value": 19022,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/16",
            "value": 129700,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "count_automorphisms/cycle/32",
            "value": 1422436,
            "range": "± 11917",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/8",
            "value": 19397,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/16",
            "value": 136530,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "automorphism_group/cycle/32",
            "value": 1417421,
            "range": "± 19617",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/8",
            "value": 38545,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/16",
            "value": 289714,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic_bliss/cycle/32",
            "value": 2830078,
            "range": "± 55125",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/8",
            "value": 39101,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/16",
            "value": 289739,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "isomorphic/cycle/32",
            "value": 2928458,
            "range": "± 13944",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/8",
            "value": 2073,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/16",
            "value": 3698,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "subisomorphic/cycle_target/32",
            "value": 7091,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "cc/karate (34v 78e)",
            "value": 920,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/100",
            "value": 1396,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/1000",
            "value": 13436,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cc/synthetic-multi-component/10000",
            "value": 134730,
            "range": "± 415",
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
            "value": 1476,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/1000",
            "value": 14392,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "distances/path/10000",
            "value": 144788,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/100",
            "value": 14289,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/1000",
            "value": 146489,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/sparse_avg_deg_4/10000",
            "value": 1590664,
            "range": "± 18042",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/100",
            "value": 172173,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/500",
            "value": 6952425,
            "range": "± 74487",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnp/dense_p_0_5/1000",
            "value": 27609480,
            "range": "± 92259",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/100",
            "value": 17220,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/1000",
            "value": 168036,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "erdos_renyi_gnm/m_eq_2n/10000",
            "value": 1939203,
            "range": "± 9669",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate (34v 78e, unweighted)",
            "value": 13573,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate weighted (varied)",
            "value": 11261,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/karate fixed seed (deterministic)",
            "value": 12522,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "louvain/ring-of-cliques 8x10 (80v 368e)",
            "value": 26622,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/textbook (6v 10e directed)",
            "value": 622,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L4xW8",
            "value": 6396,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L6xW16",
            "value": 31035,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "max_flow_value/layered/L8xW32",
            "value": 147429,
            "range": "± 1904",
            "unit": "ns/iter"
          },
          {
            "name": "count_triangles/karate",
            "value": 2297,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "transitivity_undirected/karate",
            "value": 2384,
            "range": "± 5",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}