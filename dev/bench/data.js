window.BENCHMARK_DATA = {
  "lastUpdate": 1780988601230,
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
      }
    ]
  }
}