//! Multi-run consensus validator for divergence detection and operator aggregation

use crate::forensic::validation::{detect_all_divergences, DivergenceDomain};
use crate::telemetry_reader::ExportTickData;

/// Aggregated operator attribution across N runs
#[derive(Debug, Clone)]
pub struct OperatorConsensus {
    pub lie_avg: f32,
    pub diss_avg: f32,
    pub back_avg: f32,
    pub ema_avg: f32,
}

impl OperatorConsensus {
    /// Compute agreement strength: variance-inverse of operator averages [0,1]
    pub fn agreement_strength(&self) -> f32 {
        let ops = [self.lie_avg, self.diss_avg, self.back_avg, self.ema_avg];
        let mean = ops.iter().sum::<f32>() / 4.0;
        let variance = ops.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / 4.0;
        if variance < 1e-6 {
            1.0
        } else {
            (-variance).exp()
        }
    }
}

/// Consensus metrics for a divergence cluster
#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub cluster_id: usize,
    pub start_tick: usize,
    pub end_tick: usize,
    pub domain_set: Vec<DivergenceDomain>,
    pub avg_agreement: f32,
    pub tick_count: usize,
}

/// A divergence cluster: contiguous region of divergence in same or multiple domains
#[derive(Debug, Clone)]
pub struct DivergenceCluster {
    pub start_tick: usize,
    pub end_tick: usize,
    pub domains: Vec<DivergenceDomain>,
    pub operator_intensity: Vec<f32>, // [lie, diss, back, ema] avg intensity for heatmap
}

/// Consensus analysis result for multiple runs
#[derive(Debug, Clone)]
pub struct ConsensusAnalysis {
    pub runs: Vec<Vec<ExportTickData>>,
    pub divergence_clusters: Vec<DivergenceCluster>,
    pub operator_consensus: Vec<OperatorConsensus>, // per tick
    pub cluster_metrics: Vec<ClusterMetrics>,
}

impl ConsensusAnalysis {
    /// Build full consensus analysis from multiple runs with cluster merging
    pub fn new(runs: Vec<Vec<ExportTickData>>) -> Self {
        let mut divergence_clusters = Vec::new();
        let mut operator_consensus = Vec::new();

        if runs.is_empty() {
            return ConsensusAnalysis {
                runs,
                divergence_clusters,
                operator_consensus,
                cluster_metrics: Vec::new(),
            };
        }

        let run_len = runs[0].len();

        // Compute operator consensus per tick
        for tick in 0..run_len {
            let mut lie_sum = 0.0;
            let mut diss_sum = 0.0;
            let mut back_sum = 0.0;
            let mut ema_sum = 0.0;
            let n = runs.len() as f32;

            for run in &runs {
                if tick < run.len() {
                    let contrib = crate::forensics::operator_contribution(run, tick);
                    lie_sum += contrib.lie;
                    diss_sum += contrib.diss;
                    back_sum += contrib.back;
                    ema_sum += contrib.ema;
                }
            }

            operator_consensus.push(OperatorConsensus {
                lie_avg: lie_sum / n,
                diss_avg: diss_sum / n,
                back_avg: back_sum / n,
                ema_avg: ema_sum / n,
            });
        }

        // Detect pairwise divergences (first two runs)
        let mut raw_divergences = Vec::new();
        if runs.len() >= 2 {
            raw_divergences = detect_all_divergences(&runs[0], &runs[1]);
        }

        // Merge contiguous divergences by domain
        divergence_clusters = Self::merge_divergences(&raw_divergences, &operator_consensus);

        // Compute cluster metrics
        let cluster_metrics = Self::compute_metrics(
            &divergence_clusters,
            &operator_consensus,
        );

        ConsensusAnalysis {
            runs,
            divergence_clusters,
            operator_consensus,
            cluster_metrics,
        }
    }

    /// Merge contiguous divergences in same domain into clusters
    fn merge_divergences(
        divs: &[crate::forensic::validation::DivergenceRecord],
        consensus: &[OperatorConsensus],
    ) -> Vec<DivergenceCluster> {
        if divs.is_empty() {
            return Vec::new();
        }

        let mut clusters = Vec::new();
        let mut current: Option<DivergenceCluster> = None;

        for d in divs {
            match &mut current {
                None => {
                    current = Some(DivergenceCluster {
                        start_tick: d.tick,
                        end_tick: d.tick,
                        domains: vec![d.domain],
                        operator_intensity: vec![0.0, 0.0, 0.0, 0.0],
                    });
                }
                Some(cluster) => {
                    // Merge if contiguous and same domain
                    if d.tick == cluster.end_tick + 1 && cluster.domains.contains(&d.domain) {
                        cluster.end_tick = d.tick;
                    } else if d.tick == cluster.end_tick + 1 && !cluster.domains.contains(&d.domain) {
                        // Extend cluster with new domain
                        cluster.end_tick = d.tick;
                        if !cluster.domains.contains(&d.domain) {
                            cluster.domains.push(d.domain);
                        }
                    } else {
                        // Start new cluster
                        clusters.push(cluster.clone());
                        current = Some(DivergenceCluster {
                            start_tick: d.tick,
                            end_tick: d.tick,
                            domains: vec![d.domain],
                            operator_intensity: vec![0.0, 0.0, 0.0, 0.0],
                        });
                    }
                }
            }
        }

        if let Some(cluster) = current {
            clusters.push(cluster);
        }

        // Compute operator intensity for each cluster
        for cluster in &mut clusters {
            let tick_count = cluster.end_tick - cluster.start_tick + 1;
            let mut lie_sum = 0.0;
            let mut diss_sum = 0.0;
            let mut back_sum = 0.0;
            let mut ema_sum = 0.0;

            for tick in cluster.start_tick..=cluster.end_tick {
                if tick < consensus.len() {
                    lie_sum += consensus[tick].lie_avg;
                    diss_sum += consensus[tick].diss_avg;
                    back_sum += consensus[tick].back_avg;
                    ema_sum += consensus[tick].ema_avg;
                }
            }

            cluster.operator_intensity = vec![
                lie_sum / tick_count as f32,
                diss_sum / tick_count as f32,
                back_sum / tick_count as f32,
                ema_sum / tick_count as f32,
            ];
        }

        clusters
    }

    /// Compute metrics for each cluster
    fn compute_metrics(
        clusters: &[DivergenceCluster],
        consensus: &[OperatorConsensus],
    ) -> Vec<ClusterMetrics> {
        clusters
            .iter()
            .enumerate()
            .map(|(id, cluster)| {
                let tick_count = cluster.end_tick - cluster.start_tick + 1;
                let mut agreement_sum = 0.0;

                for tick in cluster.start_tick..=cluster.end_tick {
                    if tick < consensus.len() {
                        agreement_sum += consensus[tick].agreement_strength();
                    }
                }

                let avg_agreement = agreement_sum / tick_count as f32;

                ClusterMetrics {
                    cluster_id: id,
                    start_tick: cluster.start_tick,
                    end_tick: cluster.end_tick,
                    domain_set: cluster.domains.clone(),
                    avg_agreement,
                    tick_count,
                }
            })
            .collect()
    }

    /// Get divergence count
    pub fn divergence_count(&self) -> usize {
        self.divergence_clusters.len()
    }

    /// Get consensus tick count
    pub fn consensus_tick_count(&self) -> usize {
        self.operator_consensus.len()
    }
}

/// Serialize consensus analysis to JSON
pub fn consensus_to_json(analysis: &ConsensusAnalysis) -> String {
    let mut cluster_jsons = Vec::new();
    for metric in &analysis.cluster_metrics {
        cluster_jsons.push(format!(
            r#"    {{
      "cluster_id": {},
      "start_tick": {},
      "end_tick": {},
      "tick_count": {},
      "avg_agreement": {:.4},
      "domains": {:?}
    }}"#,
            metric.cluster_id,
            metric.start_tick,
            metric.end_tick,
            metric.tick_count,
            metric.avg_agreement,
            metric.domain_set
        ));
    }

    format!(
        r#"{{
  "consensus_summary": {{
    "runs": {},
    "total_ticks": {},
    "divergence_clusters": {},
    "avg_cluster_agreement": {:.4}
  }},
  "clusters": [
{}
  ]
}}"#,
        analysis.runs.len(),
        analysis.operator_consensus.len(),
        analysis.divergence_clusters.len(),
        if analysis.cluster_metrics.is_empty() {
            1.0
        } else {
            analysis
                .cluster_metrics
                .iter()
                .map(|m| m.avg_agreement)
                .sum::<f32>()
                / analysis.cluster_metrics.len() as f32
        },
        cluster_jsons.join(",\n")
    )
}

/// Export consensus analysis to JSON file
pub fn write_consensus_json(path: &str, analysis: &ConsensusAnalysis) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path).expect("failed to create consensus json");
    let json = consensus_to_json(analysis);
    file.write_all(json.as_bytes())
        .expect("failed to write consensus json");
}

/// Export consensus analysis to CSV file
pub fn write_consensus_csv(path: &str, analysis: &ConsensusAnalysis) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path).expect("failed to create consensus csv");

    let header = "cluster_id,start_tick,end_tick,tick_count,avg_agreement,domains\n";
    file.write_all(header.as_bytes())
        .expect("failed to write consensus csv header");

    for metric in &analysis.cluster_metrics {
        let domain_str = format!("{:?}", metric.domain_set);
        let line = format!(
            "{},{},{},{},{:.4},{}\n",
            metric.cluster_id,
            metric.start_tick,
            metric.end_tick,
            metric.tick_count,
            metric.avg_agreement,
            domain_str
        );
        file.write_all(line.as_bytes())
            .expect("failed to write consensus csv line");
    }
}
