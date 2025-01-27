//! The floating panel that displays the sidebearings, advance, and other
//! glyph metrics

use druid::widget::{prelude::*, Controller, Flex};
use druid::{FontDescriptor, FontFamily, LensExt, WidgetExt};

use crate::data::{EditorState, GlyphDetail, Sidebearings};
use crate::widgets::{EditableLabel, GlyphPainter};
use crate::{consts, theme};

/// A panel for editing the selected coordinate
pub struct GlyphPane;

impl GlyphPane {
    // this is not a blessed pattern
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> impl Widget<EditorState> {
        build_widget()
    }
}

impl<W: Widget<Sidebearings>> Controller<Sidebearings, W> for GlyphPane {
    #[allow(clippy::float_cmp)]
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Sidebearings,
        env: &Env,
    ) {
        let mut child_data = data.clone();
        child.event(ctx, event, &mut child_data, env);

        // if an edit has occured in the panel, we turn it into
        // a command so that the Editor can update undo state:
        let (d_x, is_left) = if child_data.left != data.left {
            (child_data.left - data.left, true)
        } else if child_data.right != data.right {
            (child_data.right - data.right, false)
        } else {
            (0.0, false)
        };

        if d_x != 0.0 {
            let args = consts::cmd::AdjustSidebearing {
                delta: d_x,
                is_left,
            };
            ctx.submit_command(consts::cmd::ADJUST_SIDEBEARING.with(args));
        }
        // suppress clicks so that the editor doesn't handle them.
        if matches!(event, Event::MouseUp(_) | Event::MouseDown(_)) {
            ctx.set_handled();
        }
    }
}

fn build_widget() -> impl Widget<EditorState> {
    let glyph_font: FontDescriptor = FontDescriptor::new(FontFamily::MONOSPACE);
    Flex::column()
        .with_child(
            GlyphPainter::new()
                .color(theme::FIGURE_3)
                .frame_color(theme::FIGURE_4)
                .draw_layout_frame(true)
                .fix_height(256.0)
                .padding((4.0, 4.0))
                .lens(EditorState::detail_glyph),
        )
        .with_child(
            Flex::row()
                .with_child(
                    EditableLabel::parse()
                        .with_font(glyph_font.clone())
                        .with_text_size(24.0)
                        .with_text_color(theme::SECONDARY_TEXT_COLOR)
                        .with_text_alignment(druid::TextAlignment::Center)
                        .lens(Sidebearings::left)
                        .controller(GlyphPane)
                        .lens(EditorState::sidebearings)
                        .padding((4.0, 4.0))
                        .fix_width(72.0),
                )
                .with_child(
                    EditableLabel::parse()
                        .with_font(glyph_font.clone())
                        .with_text_size(24.0)
                        .with_text_color(theme::SECONDARY_TEXT_COLOR)
                        .with_text_alignment(druid::TextAlignment::Center)
                        .lens(EditorState::detail_glyph.then(GlyphDetail::advance))
                        .padding((4.0, 4.0))
                        .fix_width(72.0),
                )
                .with_child(
                    EditableLabel::parse()
                        .with_font(glyph_font.clone())
                        .with_text_size(24.0)
                        .with_text_color(theme::SECONDARY_TEXT_COLOR)
                        .with_text_alignment(druid::TextAlignment::Center)
                        .lens(Sidebearings::right)
                        .controller(GlyphPane)
                        .lens(EditorState::sidebearings)
                        .padding((4.0, 4.0))
                        .fix_width(72.0),
                )
        )
        .padding(16.0)
        .background(theme::GROUND_2)
}