//! ProductionOrder の単体テスト・性質ベーステスト

use super::*;

fn fresh() -> ProductionOrder {
    ProductionOrder::create(
        OrderId::new("o-1").expect("valid"),
        ItemCode::new("ITEM-1").expect("valid"),
        Quantity::from_u64(5),
        IdempotencyKey::new("k-001").expect("valid"),
    )
    .expect("valid")
}

#[test]
fn order_id_rejects_empty() {
    let r = OrderId::new("");
    assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
}

#[test]
fn item_code_rejects_empty() {
    let r = ItemCode::new("");
    assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
}

#[test]
fn idempotency_key_rejects_empty() {
    let r = IdempotencyKey::new("");
    assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
}

#[test]
fn idempotency_key_rejects_non_ascii() {
    let r = IdempotencyKey::new("キー");
    assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
}

#[test]
fn production_order_rejects_zero_quantity() {
    let id = OrderId::new("o-1").expect("valid");
    let item = ItemCode::new("ITEM-1").expect("valid");
    let key = IdempotencyKey::new("k-001").expect("valid");
    let r = ProductionOrder::create(id, item, Quantity::from_u64(0), key);
    assert_eq!(r.unwrap_err(), ProductionOrderError::ZeroQuantity);
}

#[test]
fn production_order_creates_with_valid_inputs() {
    let o = fresh();
    assert_eq!(o.quantity().value(), 5);
    assert_eq!(o.state(), OrderState::Released);
}

// =====================================================================
// 状態機構（C1）
// =====================================================================

#[test]
fn released_can_start_to_in_progress() {
    let mut o = fresh();
    o.start().expect("ok");
    assert_eq!(o.state(), OrderState::InProgress);
}

#[test]
fn in_progress_can_complete_to_done() {
    let mut o = fresh();
    o.start().expect("ok");
    o.complete().expect("ok");
    assert_eq!(o.state(), OrderState::Done);
    assert!(o.state().is_terminal());
}

#[test]
fn released_can_cancel() {
    let mut o = fresh();
    o.cancel().expect("ok");
    assert_eq!(o.state(), OrderState::Cancelled);
    assert!(o.state().is_terminal());
}

#[test]
fn in_progress_can_cancel() {
    let mut o = fresh();
    o.start().expect("ok");
    o.cancel().expect("ok");
    assert_eq!(o.state(), OrderState::Cancelled);
}

#[test]
fn done_rejects_further_transitions() {
    let mut o = fresh();
    o.start().expect("ok");
    o.complete().expect("ok");
    assert!(matches!(
        o.start(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
    assert!(matches!(
        o.complete(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
    assert!(matches!(
        o.cancel(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
}

#[test]
fn cancelled_rejects_further_transitions() {
    let mut o = fresh();
    o.cancel().expect("ok");
    assert!(matches!(
        o.start(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
    assert!(matches!(
        o.complete(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
    assert!(matches!(
        o.cancel(),
        Err(DomainError::InvalidStateTransition { .. })
    ));
}

#[test]
fn complete_from_released_is_rejected() {
    let mut o = fresh();
    assert!(matches!(
        o.complete(),
        Err(DomainError::InvalidStateTransition { current: "Released", .. })
    ));
}

#[test]
fn rehydrate_preserves_state() {
    let o = ProductionOrder::rehydrate(
        OrderId::new("o-2").expect("valid"),
        ItemCode::new("ITEM-2").expect("valid"),
        Quantity::from_u64(10),
        IdempotencyKey::new("k-002").expect("valid"),
        OrderState::InProgress,
    );
    assert_eq!(o.state(), OrderState::InProgress);
    let mut o2 = o;
    o2.complete().expect("ok");
    assert_eq!(o2.state(), OrderState::Done);
}

#[test]
fn order_state_label_preserves_value() {
    assert_eq!(OrderState::Released.label(), "Released");
    assert_eq!(OrderState::InProgress.label(), "InProgress");
    assert_eq!(OrderState::Done.label(), "Done");
    assert_eq!(OrderState::Cancelled.label(), "Cancelled");
}

// mutation testing 検出強化

#[test]
fn order_id_as_str_preserves_value() {
    let id = OrderId::new("ORDER-XYZ").expect("valid");
    assert_eq!(id.as_str(), "ORDER-XYZ");
}

#[test]
fn order_id_boundary_256_ok_257_reject() {
    let ok = "x".repeat(256);
    assert!(OrderId::new(ok).is_ok());
    let ng = "x".repeat(257);
    assert!(OrderId::new(ng).is_err());
}

#[test]
fn item_code_as_str_preserves_value() {
    let c = ItemCode::new("ITEM-9").expect("valid");
    assert_eq!(c.as_str(), "ITEM-9");
}

#[test]
fn item_code_boundary_64_ok_65_reject() {
    let ok = "a".repeat(64);
    assert!(ItemCode::new(ok).is_ok());
    let ng = "a".repeat(65);
    assert!(ItemCode::new(ng).is_err());
}

#[test]
fn idempotency_key_boundary_128_ok_129_reject() {
    let ok = "a".repeat(128);
    assert!(IdempotencyKey::new(ok).is_ok());
    let ng = "a".repeat(129);
    assert!(IdempotencyKey::new(ng).is_err());
}

#[test]
fn idempotency_key_display_matches_as_str() {
    let k = IdempotencyKey::new("k-display").expect("valid");
    assert_eq!(k.as_str(), "k-display");
    assert_eq!(format!("{k}"), "k-display");
}

#[test]
fn production_order_error_display_renders() {
    let e1 = ProductionOrderError::ZeroQuantity;
    assert!(format!("{e1}").contains("数量"));
    let e2 = ProductionOrderError::DuplicateKey;
    assert!(format!("{e2}").contains("重複"));
}

// =====================================================================
// 性質ベーステスト
// =====================================================================

mod props {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, Clone, Copy)]
    enum Action {
        Start,
        Complete,
        Cancel,
    }

    fn action_strategy() -> impl Strategy<Value = Action> {
        prop_oneof![
            Just(Action::Start),
            Just(Action::Complete),
            Just(Action::Cancel),
        ]
    }

    proptest! {
        #[test]
        fn terminal_states_reject_all_actions(
            ops in proptest::collection::vec(action_strategy(), 0..16)
        ) {
            // 任意の操作列の後、終端 (Done / Cancelled) に達したら以後の操作はエラー
            let mut o = fresh();
            for a in &ops {
                let _ = match a {
                    Action::Start => o.start(),
                    Action::Complete => o.complete(),
                    Action::Cancel => o.cancel(),
                };
            }
            if o.state().is_terminal() {
                prop_assert!(o.start().is_err());
                prop_assert!(o.complete().is_err());
                prop_assert!(o.cancel().is_err());
            }
        }
    }
}
