window.BENCHMARK_DATA = {
  "lastUpdate": 1780974641979,
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
      }
    ]
  }
}