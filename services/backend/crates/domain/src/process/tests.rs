//! プロセス Aggregate の単体テストおよび性質ベーステスト
//!
//! 対応 §: ロードマップ §13.1 §13.2 §13.4.1

use super::*;
use crate::value_object::CompletionCriteria;

fn step(id: &str, label: &str) -> Step {
    let sid = StepId::new(id).expect("valid step id");
    Step::new(sid, label, 60, CompletionCriteria::Manual).expect("valid step")
}

fn edge(from: &str, to: &str) -> ProcessEdge {
    ProcessEdge::new(
        StepId::new(from).expect("valid"),
        StepId::new(to).expect("valid"),
        None,
    )
}

fn pid() -> ProcessId {
    ProcessId::new("p-1").expect("valid")
}

#[test]
fn linear_three_step_dag_is_accepted() {
    let p = Process::create(
        pid(),
        "linear",
        vec![step("a", "A"), step("b", "B"), step("c", "C")],
        vec![edge("a", "b"), edge("b", "c")],
        1,
    );
    assert!(p.is_ok());
    let p = p.unwrap();
    assert_eq!(p.start_steps().len(), 1);
    assert_eq!(p.end_steps().len(), 1);
}

#[test]
fn empty_name_is_rejected() {
    let r = Process::create(pid(), "", vec![step("a", "A")], vec![], 1);
    assert_eq!(r, Err(ProcessError::EmptyName));
}

#[test]
fn duplicate_step_id_is_rejected() {
    let r = Process::create(
        pid(),
        "dup",
        vec![step("a", "A1"), step("a", "A2")],
        vec![],
        1,
    );
    assert!(matches!(r, Err(ProcessError::DuplicateStepId(_))));
}

#[test]
fn unknown_edge_endpoint_is_rejected() {
    let r = Process::create(
        pid(),
        "unknown",
        vec![step("a", "A")],
        vec![edge("a", "ghost")],
        1,
    );
    assert!(matches!(r, Err(ProcessError::EdgeReferencesUnknownStep(_))));
}

#[test]
fn cycle_is_rejected() {
    let r = Process::create(
        pid(),
        "cycle",
        vec![step("a", "A"), step("b", "B")],
        vec![edge("a", "b"), edge("b", "a")],
        1,
    );
    assert_eq!(r, Err(ProcessError::Cyclic));
}

#[test]
fn no_start_is_rejected() {
    // 単一ステップの自己ループ → 巡回 OR 入次数全あり
    let r = Process::create(
        pid(),
        "no-start",
        vec![step("a", "A")],
        vec![edge("a", "a")],
        1,
    );
    // 自己ループは巡回として先に検出される
    assert_eq!(r, Err(ProcessError::Cyclic));
}

#[test]
fn disconnected_components_are_accepted_when_each_has_a_start() {
    // 切り離された 2 つの DAG: {a→b} と {c→d}。両方 valid な工程として受理される。
    let p = Process::create(
        pid(),
        "two-islands",
        vec![step("a", "A"), step("b", "B"), step("c", "C"), step("d", "D")],
        vec![edge("a", "b"), edge("c", "d")],
        1,
    );
    assert!(p.is_ok());
}

#[test]
fn diamond_dag_is_accepted() {
    let p = Process::create(
        pid(),
        "diamond",
        vec![step("a", "A"), step("b", "B"), step("c", "C"), step("d", "D")],
        vec![edge("a", "b"), edge("a", "c"), edge("b", "d"), edge("c", "d")],
        1,
    );
    assert!(p.is_ok());
}

#[test]
fn start_and_end_detection_for_diamond() {
    let p = Process::create(
        pid(),
        "diamond",
        vec![step("a", "A"), step("b", "B"), step("c", "C"), step("d", "D")],
        vec![edge("a", "b"), edge("a", "c"), edge("b", "d"), edge("c", "d")],
        1,
    )
    .expect("valid");
    let starts: Vec<&str> = p.start_steps().iter().map(|s| s.as_str()).collect();
    let ends: Vec<&str> = p.end_steps().iter().map(|s| s.as_str()).collect();
    assert_eq!(starts, vec!["a"]);
    assert_eq!(ends, vec!["d"]);
}

#[test]
fn process_error_display_renders_useful_messages() {
    assert!(format!("{}", ProcessError::EmptyName).contains("名称"));
    assert!(format!("{}", ProcessError::Cyclic).contains("巡回"));
    assert!(
        format!("{}", ProcessError::DuplicateStepId("x".into())).contains("重複")
    );
    assert!(format!("{}", ProcessError::NoStartStep).contains("入次数"));
    assert!(format!("{}", ProcessError::NoEndStep).contains("出次数"));
}

// =====================================================================
// 性質ベーステスト（§13.2）
// =====================================================================

mod props {
    use super::*;
    use proptest::prelude::*;

    /// `n` 個の線形ステップ a0→a1→...→a(n-1) を生成し、Process が常に妥当となる
    fn linear_chain(n: usize) -> (Vec<Step>, Vec<ProcessEdge>) {
        let steps: Vec<Step> = (0..n)
            .map(|i| step(&format!("a{i}"), &format!("Step {i}")))
            .collect();
        let edges: Vec<ProcessEdge> = (0..n.saturating_sub(1))
            .map(|i| edge(&format!("a{i}"), &format!("a{}", i + 1)))
            .collect();
        (steps, edges)
    }

    proptest! {
        #[test]
        fn arbitrary_linear_chain_is_always_valid(n in 1usize..32) {
            let (steps, edges) = linear_chain(n);
            let r = Process::create(pid(), "chain", steps, edges, 1);
            prop_assert!(r.is_ok());
        }

        #[test]
        fn back_edge_in_linear_chain_always_creates_cycle(
            n in 2usize..32,
            back_pair in (0usize..32, 0usize..32)
        ) {
            // 線形チェーン上に逆向き辺 a(j) → a(i) (j > i) を入れたら必ず巡回
            let (i, j) = (back_pair.0 % n, back_pair.1 % n);
            prop_assume!(j > i);
            let (steps, mut edges) = linear_chain(n);
            edges.push(edge(&format!("a{j}"), &format!("a{i}")));
            let r = Process::create(pid(), "withback", steps, edges, 1);
            prop_assert_eq!(r, Err(ProcessError::Cyclic));
        }
    }
}
