//! プロセスグラフの不変条件検査
//!
//! 対応 §: ロードマップ §3.4 §10.2.1
//!
//! [`super::Process::create`] から呼ばれ、構造的妥当性を保証する。
//! ドメイン層なので外部 crate には依存しない。

use std::collections::{HashMap, HashSet};

use super::{ProcessEdge, ProcessError, Step, StepId};

/// 全ての不変条件を検査する
pub(super) fn validate(steps: &[Step], edges: &[ProcessEdge]) -> Result<(), ProcessError> {
    let mut id_set: HashSet<&StepId> = HashSet::with_capacity(steps.len());
    for s in steps {
        if !id_set.insert(s.id()) {
            return Err(ProcessError::DuplicateStepId(s.id().as_str().to_owned()));
        }
    }
    for e in edges {
        if !id_set.contains(e.from()) {
            return Err(ProcessError::EdgeReferencesUnknownStep(e.from().as_str().to_owned()));
        }
        if !id_set.contains(e.to()) {
            return Err(ProcessError::EdgeReferencesUnknownStep(e.to().as_str().to_owned()));
        }
    }
    if has_cycle(steps, edges) {
        return Err(ProcessError::Cyclic);
    }
    if find_starts(steps, edges).is_empty() {
        return Err(ProcessError::NoStartStep);
    }
    if find_ends(steps, edges).is_empty() {
        return Err(ProcessError::NoEndStep);
    }
    Ok(())
}

/// 入次数 0 の Step を返す
pub(super) fn find_starts<'a>(steps: &'a [Step], edges: &[ProcessEdge]) -> Vec<&'a StepId> {
    let mut has_in: HashSet<&StepId> = HashSet::new();
    for e in edges {
        has_in.insert(e.to());
    }
    steps.iter().map(Step::id).filter(|id| !has_in.contains(id)).collect()
}

/// 出次数 0 の Step を返す
pub(super) fn find_ends<'a>(steps: &'a [Step], edges: &[ProcessEdge]) -> Vec<&'a StepId> {
    let mut has_out: HashSet<&StepId> = HashSet::new();
    for e in edges {
        has_out.insert(e.from());
    }
    steps.iter().map(Step::id).filter(|id| !has_out.contains(id)).collect()
}

/// DFS の三色塗りで巡回を検出する。grey が再訪されたら巡回あり。
fn has_cycle(steps: &[Step], edges: &[ProcessEdge]) -> bool {
    let mut adj: HashMap<&StepId, Vec<&StepId>> = HashMap::with_capacity(steps.len());
    for s in steps {
        adj.entry(s.id()).or_default();
    }
    for e in edges {
        adj.entry(e.from()).or_default().push(e.to());
    }
    let mut color: HashMap<&StepId, Color> = adj.keys().map(|k| (*k, Color::White)).collect();
    let nodes: Vec<&StepId> = adj.keys().copied().collect();
    for n in nodes {
        if matches!(color.get(n).copied().unwrap_or(Color::White), Color::White)
            && dfs(n, &adj, &mut color)
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
enum Color {
    White,
    Grey,
    Black,
}

fn dfs<'a>(
    node: &'a StepId,
    adj: &HashMap<&'a StepId, Vec<&'a StepId>>,
    color: &mut HashMap<&'a StepId, Color>,
) -> bool {
    color.insert(node, Color::Grey);
    if let Some(nbrs) = adj.get(node) {
        for nb in nbrs {
            match color.get(nb).copied().unwrap_or(Color::White) {
                Color::Grey => return true,
                Color::White => {
                    if dfs(nb, adj, color) {
                        return true;
                    }
                }
                Color::Black => {}
            }
        }
    }
    color.insert(node, Color::Black);
    false
}
