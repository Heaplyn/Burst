//! Ring 2 · Elaboration · **Layer 5 (Optimization) — dead-code elimination**
//!
//! Deletes layers that produce no observable effect. Right now the only
//! signal we trust is `ObservabilityFlags` — a layer with
//! `AffectsOutput=false && AffectsHardware=false && ObservableToTrace=false`
//! and no children with observable effects can be dropped.
//!
//! Since Ring 2 doesn't yet populate `ObservabilityFlags` from data flow, this
//! pass will be a no-op on most programs today. Wire it up once
//! observability propagation lands.

use ast::{Layer, LayerKind};

/// Rewrite `Root` — return a new tree with unobservable layers removed.
pub fn Eliminate(Root: &Layer) -> Layer {
    let mut node = Root.clone();
    Prune(&mut node);
    node
}

fn Prune(L: &mut Layer) {
    L.Children.retain(|c| IsObservable(c));
    for c in &mut L.Children {
        Prune(c);
    }
}

fn IsObservable(L: &Layer) -> bool {
    // Terminals we never drop.
    if matches!(
        L.Kind,
        LayerKind::Return { .. } | LayerKind::Panic | LayerKind::Unreachable | LayerKind::Havoc { .. } | LayerKind::Interrupt { .. }
    ) {
        return true;
    }
    // If the observability flags claim any external effect, keep it.
    let f = &L.Observability;
    if f.AffectsOutput || f.AffectsHardware {
        return true;
    }
    // Anything with observable children is kept.
    L.Children.iter().any(IsObservable) || !matches!(L.Kind, LayerKind::Expression(_))
}
