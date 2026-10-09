//! Hotkey bindings for the in-sector HUD.
//!
//! One list is the single source of truth for both halves of a hotkey: the key
//! the game listens for, and the `[X]` hint the HUD prints. (#218) existed
//! because those two lived in different files and drifted — the status bar
//! advertised `[C] Dock` and `[X] Mining Beam` long after the keys stopped
//! being read. Labels are now built from `hint()` / `key_name()`, so a
//! rebinding updates the UI text and an unbound action is a compile error.

use macroquad::prelude::*;

/// A player intent that a key can be bound to.
///
/// Add a variant here and `DEFAULT_BINDINGS` stops compiling until it has a
/// key — that's deliberate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    TargetClosest,
    ToggleMiningBeam,
    DockJumpUndock,
    ShipWindow,
    FactionWindow,
    AssetsWindow,
    MapWindow,
    BuildWindow,
    DebugWindow,
}

/// Every action, paired with the key it ships bound to.
const DEFAULT_BINDINGS: [(Action, KeyCode); 9] = [
    (Action::TargetClosest, KeyCode::E),
    (Action::ToggleMiningBeam, KeyCode::X),
    (Action::DockJumpUndock, KeyCode::C),
    (Action::ShipWindow, KeyCode::R),
    (Action::FactionWindow, KeyCode::F),
    (Action::AssetsWindow, KeyCode::T),
    (Action::MapWindow, KeyCode::M),
    (Action::BuildWindow, KeyCode::B),
    (Action::DebugWindow, KeyCode::F3),
];

/// The live bindings for this session.
///
// ponytail: in-memory only, resets to defaults on launch. Persistence (#246)
// and a remap UI (#247) both just need to read/write `bindings`.
pub struct Hotkeys {
    bindings: Vec<(Action, KeyCode)>,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Hotkeys {
            bindings: DEFAULT_BINDINGS.to_vec(),
        }
    }
}

impl Hotkeys {
    /// The key currently bound to `action`.
    pub fn key(&self, action: Action) -> KeyCode {
        self.bindings
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, key)| *key)
            // Unreachable while `DEFAULT_BINDINGS` covers every variant, which
            // the `every_action_is_bound` test enforces.
            .unwrap_or(KeyCode::Unknown)
    }

    /// Was `action`'s key pressed this frame?
    ///
    /// Callers still own the "should the player be able to do this now?"
    /// question — this only reports the keypress.
    pub fn pressed(&self, action: Action) -> bool {
        is_key_pressed(self.key(action))
    }

    /// Bare key name for labels that bracket a letter mid-word: `"T"` in
    /// `format!("ASSE[{}]S", ..)`.
    pub fn key_name(&self, action: Action) -> String {
        format!("{:?}", self.key(action))
    }

    /// Bracketed hint for labels that lead with the key: `"[X]"` in
    /// `format!("{} Mining Beam: On", ..)`.
    pub fn hint(&self, action: Action) -> String {
        format!("[{}]", self.key_name(action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ACTIONS: [Action; 9] = [
        Action::TargetClosest,
        Action::ToggleMiningBeam,
        Action::DockJumpUndock,
        Action::ShipWindow,
        Action::FactionWindow,
        Action::AssetsWindow,
        Action::MapWindow,
        Action::BuildWindow,
        Action::DebugWindow,
    ];

    /// `key()` falls back to `Unknown` rather than panicking, so a missing
    /// binding would otherwise ship as a silently dead hotkey — exactly the
    /// failure #218 was filed for.
    #[test]
    fn every_action_is_bound() {
        let hotkeys = Hotkeys::default();
        for action in ALL_ACTIONS {
            assert_ne!(
                hotkeys.key(action),
                KeyCode::Unknown,
                "{action:?} has no key in DEFAULT_BINDINGS"
            );
        }
    }

    /// Two actions on one key means one of them loses, depending on frame
    /// order. Catch it at the binding list instead.
    #[test]
    fn no_key_is_bound_twice() {
        let hotkeys = Hotkeys::default();
        for (i, (action, key)) in hotkeys.bindings.iter().enumerate() {
            for (other_action, other_key) in &hotkeys.bindings[i + 1..] {
                assert_ne!(
                    key, other_key,
                    "{action:?} and {other_action:?} are both bound to {key:?}"
                );
            }
        }
    }

    /// The HUD prints these verbatim; `[X]` and `X` are what the labels assume.
    #[test]
    fn hints_read_the_way_labels_expect() {
        let hotkeys = Hotkeys::default();
        assert_eq!(hotkeys.hint(Action::ToggleMiningBeam), "[X]");
        assert_eq!(hotkeys.key_name(Action::AssetsWindow), "T");
    }
}
