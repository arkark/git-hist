use crate::app::state::State;
use crate::app::terminal::Terminal;
use anyhow::Result;
use std::cmp;
use std::convert::TryFrom;
use tui::{layout, text, widgets};

pub const COMMIT_INFO_INNER_HEIGHT: u16 = 2;
pub const COMMIT_INFO_OUTER_HEIGHT: u16 = COMMIT_INFO_INNER_HEIGHT + 2;
pub const NAVI_WIDTH: u16 = 3;

#[derive(Debug)]
pub struct Dashboard<'a> {
    commit_info_title: text::Spans<'a>,
    commit_info_text: Vec<text::Spans<'a>>,
    left_navi_text: Vec<text::Spans<'a>>,
    right_navi_text: Vec<text::Spans<'a>>,
    diff_text: Vec<text::Spans<'a>>,
}

impl<'a> Dashboard<'a> {
    pub fn new(state: &'a State) -> Self {
        let left_navi_text = get_left_navi_text(&state);
        let right_navi_text = get_right_navi_text(&state);
        let commit_info_title = get_commit_info_title(&state);
        let commit_info_text = get_commit_info_text(&state);
        let diff_text = get_diff_text(&state);

        Self {
            commit_info_title,
            commit_info_text,
            left_navi_text,
            right_navi_text,
            diff_text,
        }
    }

    pub fn draw(self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|frame| {
            let vertical_chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        layout::Constraint::Length(COMMIT_INFO_OUTER_HEIGHT),
                        layout::Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            let commit_chunk = vertical_chunks[0];
            let diff_chunk = vertical_chunks[1];

            let horizontal_chunks = layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints(
                    [
                        layout::Constraint::Length(NAVI_WIDTH),
                        layout::Constraint::Min(0),
                        layout::Constraint::Length(NAVI_WIDTH),
                    ]
                    .as_ref(),
                )
                .split(commit_chunk);
            let left_navi_chunk = horizontal_chunks[0];
            let right_navi_chunk = horizontal_chunks[2];
            let commit_info_chunk = layout::Layout::default()
                .constraints([layout::Constraint::Min(0)].as_ref())
                .split(horizontal_chunks[1])[0];

            let left_navi_paragraph = widgets::Paragraph::new(self.left_navi_text);
            frame.render_widget(left_navi_paragraph, left_navi_chunk);

            let right_navi_paragraph = widgets::Paragraph::new(self.right_navi_text);
            frame.render_widget(right_navi_paragraph, right_navi_chunk);

            let block = widgets::Block::default()
                .title(self.commit_info_title)
                .borders(widgets::Borders::ALL)
                .border_type(widgets::BorderType::Rounded);
            let commit_info_paragraph = widgets::Paragraph::new(self.commit_info_text).block(block);
            frame.render_widget(commit_info_paragraph, commit_info_chunk);

            // diff
            let diff_paragraph = widgets::Paragraph::new(self.diff_text);
            frame.render_widget(diff_paragraph, diff_chunk);
        })?;

        Ok(())
    }

    pub fn diff_height(terminal_height: usize) -> usize {
        let terminal_height: isize = isize::try_from(terminal_height).unwrap();
        let commit_info_outer_height: isize = isize::try_from(COMMIT_INFO_OUTER_HEIGHT).unwrap();
        usize::try_from(cmp::max(0, terminal_height - commit_info_outer_height)).unwrap()
    }
}

fn get_left_navi_text<'a>(state: &'a State) -> Vec<text::Spans<'a>> {
    let backward_symbol = if state.is_earliest_commit() { "" } else { "<<" };
    let up_symbol = if state.can_move_up() { "^" } else { "" };
    let down_symbol = if state.can_move_down() { "v" } else { "" };

    vec![
        text::Spans::from(format!("{:^1$}", up_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:<1$}", backward_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:<1$}", backward_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:^1$}", down_symbol, usize::from(NAVI_WIDTH))),
    ]
}

fn get_right_navi_text<'a>(state: &'a State) -> Vec<text::Spans<'a>> {
    let forward_symbol = if state.is_latest_commit() { "" } else { ">>" };
    let up_symbol = if state.can_move_up() { "^" } else { "" };
    let down_symbol = if state.can_move_down() { "v" } else { "" };

    vec![
        text::Spans::from(format!("{:^1$}", up_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:>1$}", forward_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:>1$}", forward_symbol, usize::from(NAVI_WIDTH))),
        text::Spans::from(format!("{:^1$}", down_symbol, usize::from(NAVI_WIDTH))),
    ]
}

fn get_commit_info_title<'a>(state: &'a State) -> text::Spans<'a> {
    let short_id = state.point().commit().short_id();
    let references = state.point().commit().references();

    // TODO:
    //   - option: date format (default: "[%Y-%m-%d]")
    //     - ref. https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html
    //   - option: author (default) or committer
    let author = format!("@{}", state.point().commit().author());
    let author_date = state
        .point()
        .commit()
        .author_date()
        .format("[%Y-%m-%d]")
        .to_string();
    let _committer = format!("@{}", state.point().commit().committer());
    let _committer_date = state
        .point()
        .commit()
        .committer_date()
        .format("[%Y-%m-%d]")
        .to_string();

    let mut commit_info_title = vec![];
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw("Commit:"));
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw(short_id));
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw(author_date));
    commit_info_title.push(text::Span::raw(" "));
    if !references.is_empty() {
        commit_info_title.push(text::Span::raw("("));
        for name in references.head_names().into_iter() {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in references.local_branch_names().into_iter() {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in references.remote_branch_names().into_iter() {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in references.tag_names().into_iter() {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        commit_info_title.pop();
        commit_info_title.push(text::Span::raw(")"));
        commit_info_title.push(text::Span::raw(" "));
    }
    commit_info_title.push(text::Span::raw(author));
    commit_info_title.push(text::Span::raw(" "));

    text::Spans::from(commit_info_title)
}

fn get_commit_info_text<'a>(state: &'a State) -> Vec<text::Spans<'a>> {
    let commit_summary = text::Spans::from(vec![text::Span::raw(state.point().commit().summary())]);
    let change_status = text::Spans(vec![text::Span::raw(state.point().diff().status())]);

    vec![commit_summary, change_status]
}

fn get_diff_text<'a>(state: &'a State) -> Vec<text::Spans<'a>> {
    let mut diff_text = vec![];
    let max_line_number_len = state.max_line_number_len();
    for line in state.point().diff().lines().iter().skip(state.line_index()) {
        let old_line_number = format!(
            "{:>1$}",
            if let Some(number) = line.old_line_number() {
                number.to_string()
            } else {
                String::new()
            },
            max_line_number_len,
        );
        let new_line_number = format!(
            "{:>1$}",
            if let Some(number) = line.new_line_number() {
                number.to_string()
            } else {
                String::new()
            },
            max_line_number_len,
        );
        let sign = line.sign();
        let style = line.style();

        let mut spans = vec![
            text::Span::raw(old_line_number),
            text::Span::raw(" "),
            text::Span::raw(new_line_number),
            text::Span::raw(" |"),
            text::Span::styled(sign, style),
            text::Span::styled(" ", style),
        ];
        for part in line.parts().iter() {
            // TODO:
            //   - option: --emphasize-diff (default: false)
            let _style = part.emphasize(style); // if true
            let style = style; // if false
            spans.push(text::Span::styled(part.text(), style));
        }

        let spans = text::Spans::from(spans);

        diff_text.push(spans);
    }

    diff_text
}
