use crate::app::state::State;
use crate::app::terminal::Terminal;
use crate::args::UserType;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::iter;
use tui::{layout, style, text, widgets};

const COMMIT_INFO_INNER_HEIGHT: u16 = 2;
const COMMIT_INFO_OUTER_HEIGHT: u16 = COMMIT_INFO_INNER_HEIGHT + 2;
const COMMIT_INFO_HORIZONTAL_PADDING: u16 = 1;
const NAVI_WIDTH: u16 = 3;

static BINARY_ALERT_TEXT: Lazy<Vec<text::Spans>> = Lazy::new(|| {
    vec![
        text::Spans::from(vec![text::Span::styled(
            "╭──────────────────────────────────────────────╮",
            style::Style::default().add_modifier(style::Modifier::DIM),
        )]),
        text::Spans::from(vec![
            text::Span::styled(
                "│",
                style::Style::default().add_modifier(style::Modifier::DIM),
            ),
            text::Span::raw(" Note: binary data is not shown in a terminal "),
            text::Span::styled(
                "│",
                style::Style::default().add_modifier(style::Modifier::DIM),
            ),
        ]),
        text::Spans::from(vec![text::Span::styled(
            "╰──────────────────────────────────────────────╯",
            style::Style::default().add_modifier(style::Modifier::DIM),
        )]),
    ]
});

#[derive(Debug)]
pub struct Dashboard<'a> {
    commit_info_title: text::Spans<'a>,
    commit_info_paragraph: widgets::Paragraph<'a>,
    left_navi_paragraph: widgets::Paragraph<'a>,
    right_navi_paragraph: widgets::Paragraph<'a>,
    diff_paragraph: widgets::Paragraph<'a>,
}

impl<'a> Dashboard<'a> {
    pub fn new(state: &'a State) -> Self {
        Self {
            commit_info_title: Self::get_commit_info_title(&state),
            commit_info_paragraph: Self::get_commit_info_paragraph(&state),
            left_navi_paragraph: Self::get_left_navi_paragraph(&state),
            right_navi_paragraph: Self::get_right_navi_paragraph(&state),
            diff_paragraph: Self::get_diff_paragraph(&state),
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

            frame.render_widget(self.left_navi_paragraph, left_navi_chunk);
            frame.render_widget(self.right_navi_paragraph, right_navi_chunk);

            let commit_info_block = widgets::Block::default()
                .title(self.commit_info_title)
                .borders(widgets::Borders::ALL)
                .border_type(widgets::BorderType::Rounded);

            let commit_info_inner_chunk = layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints(
                    [
                        layout::Constraint::Length(COMMIT_INFO_HORIZONTAL_PADDING),
                        layout::Constraint::Min(0),
                        layout::Constraint::Length(COMMIT_INFO_HORIZONTAL_PADDING),
                    ]
                    .as_ref(),
                )
                .split(commit_info_block.inner(commit_info_chunk))[1];

            frame.render_widget(commit_info_block, commit_info_chunk);
            frame.render_widget(self.commit_info_paragraph, commit_info_inner_chunk);

            // diff
            frame.render_widget(self.diff_paragraph, diff_chunk);
        })?;

        Ok(())
    }

    pub fn diff_height(terminal_height: usize) -> usize {
        terminal_height.saturating_sub(usize::from(COMMIT_INFO_OUTER_HEIGHT))
    }

    fn get_left_navi_paragraph(state: &'a State) -> widgets::Paragraph<'a> {
        let backward_symbol = if state.point().is_earliest() {
            ""
        } else {
            "<<"
        };
        let up_symbol = if state.can_move_up() { "^" } else { "" };
        let down_symbol = if state.can_move_down() { "v" } else { "" };

        widgets::Paragraph::new(vec![
            text::Spans::from(format!("{:^1$}", up_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:<1$}", backward_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:<1$}", backward_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:^1$}", down_symbol, usize::from(NAVI_WIDTH))),
        ])
    }

    fn get_right_navi_paragraph(state: &'a State) -> widgets::Paragraph<'a> {
        let forward_symbol = if state.point().is_latest() { "" } else { ">>" };
        let up_symbol = if state.can_move_up() { "^" } else { "" };
        let down_symbol = if state.can_move_down() { "v" } else { "" };

        widgets::Paragraph::new(vec![
            text::Spans::from(format!("{:^1$}", up_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:>1$}", forward_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:>1$}", forward_symbol, usize::from(NAVI_WIDTH))),
            text::Spans::from(format!("{:^1$}", down_symbol, usize::from(NAVI_WIDTH))),
        ])
    }

    fn get_commit_info_title(state: &'a State) -> text::Spans<'a> {
        let hash = if state.args().should_use_full_commit_hash {
            state.point().commit().long_id()
        } else {
            state.point().commit().short_id()
        };
        let references = state.point().commit().references();

        let name = format!(
            "@{}",
            match state.args().user_for_name {
                UserType::Author => state.point().commit().author_name(),
                UserType::Committer => state.point().commit().committer_name(),
            }
        );

        let date = (match state.args().user_for_name {
            UserType::Author => state.point().commit().author_date(),
            UserType::Committer => state.point().commit().committer_date(),
        })
        .format(&state.args().date_format)
        .to_string();

        let mut commit_info_title = vec![];
        {
            commit_info_title.push(text::Span::raw("[ "));
            commit_info_title.push(text::Span::styled(
                hash,
                style::Style::default().fg(style::Color::Yellow),
            ));
            commit_info_title.push(text::Span::raw(" "));
            commit_info_title.push(text::Span::styled(
                date,
                style::Style::default().fg(style::Color::LightMagenta),
            ));
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
            commit_info_title.push(text::Span::styled(
                name,
                style::Style::default().fg(style::Color::Cyan),
            ));
            commit_info_title.push(text::Span::raw(" ]"));
        }

        text::Spans::from(commit_info_title)
    }

    fn get_commit_info_paragraph(state: &'a State) -> widgets::Paragraph<'a> {
        let commit_summary =
            text::Spans::from(vec![text::Span::raw(state.point().commit().summary())]);
        let change_status = text::Spans(vec![text::Span::raw(state.point().diff().status())]);

        widgets::Paragraph::new(vec![commit_summary, change_status])
    }

    fn get_diff_paragraph(state: &'a State) -> widgets::Paragraph<'a> {
        if let Some(lines) = state.point().diff().lines() {
            let mut diff_text = vec![];
            let max_line_number_len = state.max_line_number_len();
            for line in lines.iter().skip(state.line_index()) {
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
                    text::Span::raw(" │"),
                    text::Span::styled(sign, style),
                    text::Span::styled(" ", style),
                ];
                for part in line.parts().iter() {
                    let style = if state.args().should_emphasize_diff {
                        part.emphasize(style)
                    } else {
                        style
                    };
                    let text = part.text().replace("\t", &state.args().tab_spaces);
                    spans.push(text::Span::styled(text, style));
                }

                let spans = text::Spans::from(spans);

                diff_text.push(spans);
            }
            widgets::Paragraph::new(diff_text)
        } else {
            // for a binary file
            let mut alert_text = vec![];

            let diff_height = Self::diff_height(state.terminal_height());
            let offset = diff_height.saturating_sub(BINARY_ALERT_TEXT.len()) / 2;
            alert_text.append(
                &mut iter::repeat(text::Spans::from(vec![]))
                    .take(offset)
                    .collect(),
            );
            alert_text.append(&mut BINARY_ALERT_TEXT.clone());

            widgets::Paragraph::new(alert_text).alignment(layout::Alignment::Center)
        }
    }
}
