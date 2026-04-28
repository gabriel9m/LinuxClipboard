use crate::domain::HistoryItem;
use crate::paste::SelectionSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupCommand {
    MoveUp,
    MoveDown,
    Enter,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupAction {
    None,
    Activate {
        index: usize,
        source: SelectionSource,
    },
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupState {
    item_count: usize,
    selected_index: Option<usize>,
}

impl PopupState {
    pub fn from_items(items: &[HistoryItem]) -> Self {
        Self {
            item_count: items.len(),
            selected_index: (!items.is_empty()).then_some(0),
        }
    }

    pub fn empty() -> Self {
        Self {
            item_count: 0,
            selected_index: None,
        }
    }

    pub fn item_count(&self) -> usize {
        self.item_count
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    pub fn empty_message(&self) -> Option<&'static str> {
        self.is_empty().then_some("Nenhum item no histórico")
    }

    pub fn handle_command(&mut self, command: PopupCommand) -> PopupAction {
        match command {
            PopupCommand::MoveUp => {
                self.move_up();
                PopupAction::None
            }
            PopupCommand::MoveDown => {
                self.move_down();
                PopupAction::None
            }
            PopupCommand::Enter => self.activate_selected(SelectionSource::Enter),
            PopupCommand::Escape => PopupAction::Cancel,
        }
    }

    pub fn click_item(&mut self, index: usize) -> PopupAction {
        if index >= self.item_count {
            return PopupAction::None;
        }

        self.selected_index = Some(index);
        self.activate_selected(SelectionSource::Click)
    }

    fn move_up(&mut self) {
        let Some(index) = self.selected_index else {
            return;
        };

        self.selected_index = Some(index.saturating_sub(1));
    }

    fn move_down(&mut self) {
        let Some(index) = self.selected_index else {
            return;
        };

        let last_index = self.item_count.saturating_sub(1);
        self.selected_index = Some((index + 1).min(last_index));
    }

    fn activate_selected(&self, source: SelectionSource) -> PopupAction {
        match self.selected_index {
            Some(index) => PopupAction::Activate { index, source },
            None => PopupAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_item(value: &str) -> HistoryItem {
        HistoryItem::text(value).expect("valid text item")
    }

    fn items() -> Vec<HistoryItem> {
        vec![
            text_item("newest"),
            text_item("middle"),
            text_item("oldest"),
        ]
    }

    #[test]
    fn selects_newest_item_by_default_when_popup_opens_with_items() {
        let items = items();

        let state = PopupState::from_items(&items);

        assert_eq!(state.item_count(), 3);
        assert_eq!(state.selected_index(), Some(0));
        assert!(!state.is_empty());
        assert_eq!(items[state.selected_index().unwrap()].preview, "newest");
    }

    #[test]
    fn represents_empty_history_explicitly() {
        let state = PopupState::empty();

        assert_eq!(state.item_count(), 0);
        assert_eq!(state.selected_index(), None);
        assert!(state.is_empty());
        assert_eq!(state.empty_message(), Some("Nenhum item no histórico"));
    }

    #[test]
    fn from_empty_items_has_no_selection_and_empty_message() {
        let items = Vec::new();

        let state = PopupState::from_items(&items);

        assert_eq!(state.selected_index(), None);
        assert_eq!(state.empty_message(), Some("Nenhum item no histórico"));
    }

    #[test]
    fn arrow_down_moves_selection_without_passing_last_item() {
        let items = items();
        let mut state = PopupState::from_items(&items);

        assert_eq!(
            state.handle_command(PopupCommand::MoveDown),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(
            state.handle_command(PopupCommand::MoveDown),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(
            state.handle_command(PopupCommand::MoveDown),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn arrow_up_moves_selection_without_passing_first_item() {
        let items = items();
        let mut state = PopupState::from_items(&items);

        state.handle_command(PopupCommand::MoveDown);
        state.handle_command(PopupCommand::MoveDown);
        assert_eq!(state.selected_index(), Some(2));

        assert_eq!(
            state.handle_command(PopupCommand::MoveUp),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(
            state.handle_command(PopupCommand::MoveUp),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(
            state.handle_command(PopupCommand::MoveUp),
            PopupAction::None
        );
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn enter_activates_selected_item() {
        let items = items();
        let mut state = PopupState::from_items(&items);
        state.handle_command(PopupCommand::MoveDown);

        let action = state.handle_command(PopupCommand::Enter);

        assert_eq!(
            action,
            PopupAction::Activate {
                index: 1,
                source: SelectionSource::Enter
            }
        );
    }

    #[test]
    fn enter_on_empty_popup_does_not_activate_anything() {
        let mut state = PopupState::empty();

        let action = state.handle_command(PopupCommand::Enter);

        assert_eq!(action, PopupAction::None);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn escape_cancels_without_changing_selection() {
        let items = items();
        let mut state = PopupState::from_items(&items);
        state.handle_command(PopupCommand::MoveDown);

        let action = state.handle_command(PopupCommand::Escape);

        assert_eq!(action, PopupAction::Cancel);
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn clicking_item_selects_and_activates_it() {
        let items = items();
        let mut state = PopupState::from_items(&items);

        let action = state.click_item(2);

        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(
            action,
            PopupAction::Activate {
                index: 2,
                source: SelectionSource::Click
            }
        );
    }

    #[test]
    fn clicking_out_of_range_item_does_nothing() {
        let items = items();
        let mut state = PopupState::from_items(&items);

        let action = state.click_item(99);

        assert_eq!(action, PopupAction::None);
        assert_eq!(state.selected_index(), Some(0));
    }
}
