use crossterm::event::{Event, KeyEventKind, KeyCode};
use ratatui::Frame;
use ratatui::style::{Modifier, Style, Color};
use ratatui::widgets::{List, Block, Borders, ListDirection, ListState};

use crate::errors::Errcode;

use super::{enter_tui_screen, quit_tui_screen, Tuiterm, get_keyboard_events};

pub struct SelectFromList<'a> {
    list: List<'a>,
    state: ListState,
}

impl<'a> SelectFromList<'a> {
    fn init<L: Iterator<Item = String>>(list: L) -> SelectFromList<'a> {
        let list = List::new(list)
            .block(Block::default().title("List").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let mut state = ListState::default();
        state.select(Some(0));
        SelectFromList { list, state, }
    }

    fn ui(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(self.list.clone(), frame.size(), &mut self.state);
    }

    fn handle_events(&mut self) -> Result<bool, Errcode> {
        if let Some(Event::Key(key)) = get_keyboard_events()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Up => self.state.select(Some(self.state.selected().unwrap().saturating_sub(1))),
                    KeyCode::Down => self.state.select(Some((self.state.selected().unwrap() + 1).min(self.list.len() - 1))),
                    _ => {},
                }
            }
        }
        Ok(false)
    }

    pub fn exec(&mut self, terminal: &mut Tuiterm) -> Result<(), Errcode> {
        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|frame| self.ui(frame))?;
            should_quit = self.handle_events()?;
        }
        Ok(())
    }
}

pub fn select_from_list<T, F>(list: &[T], disp_f: F) -> usize
where
    F: Fn(&T) -> String,
{
    let mut terminal = enter_tui_screen().expect("Unable to init TUI");
    let mut widget = SelectFromList::init(list.iter().map(|n| disp_f(n)));
    widget.exec(&mut terminal).expect("Error while exec widget");
    quit_tui_screen().expect("Unable to exit TUI");
    widget.state.selected().unwrap()
}
