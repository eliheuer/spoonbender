//! The top-level widget for the main glyph list window.

use std::sync::Arc;

use druid::kurbo::{Affine, Rect, Shape, Size};
//use druid::piet::{
//FontBuilder, PietText, PietTextLayout, RenderContext, Text, TextLayout, TextLayoutBuilder,
//};
use druid::widget::prelude::*;
use druid::{Data, Insets, TextLayout, WidgetExt, WidgetPod};
use druid::widget::Padding;

use crate::app_delegate::EDIT_GLYPH;
use crate::data::{GridGlyph, Workspace};
use crate::theme;
use crate::widgets::Maybe;

const GLYPH_SIZE: f64 = 157.;

#[derive(Default)]
pub struct GlyphGrid {
    children: Vec<WidgetPod<Workspace, Box<dyn Widget<Workspace>>>>,
}

impl GlyphGrid {
    fn update_children(&mut self, data: &Workspace) {
        self.children.clear();
        for key in data.font.ufo.iter_names() {
            let widget = Maybe::or_empty(GridInner::new);
            self.children.push(WidgetPod::new(
                widget.lens(Workspace::glyph_grid(key)).boxed(),
            ));
        }
    }
}

impl Widget<Workspace> for GlyphGrid {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &Workspace, env: &Env) {
        ctx.render_ctx.clear(env.get(theme::GROUND_1));
        let row_len = 1.0_f64.max(ctx.size().width / GLYPH_SIZE).floor() as usize;
        let _row_count = if self.children.is_empty() {
            0
        } else {
            self.children.len() / row_len + 1
        };

        for child in &mut self.children {
            child.paint(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Workspace,
        env: &Env,
    ) -> Size {
        let available_width = bc.max().width;
        let cell_margin = 8.0; // Add this line to define the margin
        let cell_size = GLYPH_SIZE + cell_margin; // Modify this line
        let width = (available_width / cell_size).floor() * cell_size;
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;

        let child_bc = BoxConstraints::tight(Size::new(GLYPH_SIZE, GLYPH_SIZE));

        for child in &mut self.children {
            if x > 0. && x + cell_size > width {
                y += cell_size;
                x = 0.;
            }
            child.layout(ctx, &child_bc, data, env);
            let rect = Rect::from_origin_size((x + cell_margin / 2.0, y + cell_margin / 2.0), (GLYPH_SIZE, GLYPH_SIZE));
            child.set_layout_rect(ctx, data, env, rect);
            x += cell_size;
        }
        Size::new(width, y + cell_size)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Workspace, env: &Env) {
        for child in &mut self.children {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Workspace,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if self.children.is_empty() {
                self.update_children(data);
            }
        }

        for child in &mut self.children {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &Workspace, new: &Workspace, env: &Env) {
        //eprintln!("grid update generation {}, {}", old.cache.generation.get(), new.cache.generation.get());
        if !old.font.same(&new.font) {
            //eprintln!("old font changed");
            self.update_children(new);
            ctx.children_changed();
            ctx.request_paint();
        } else {
            //eprintln!("cache same {}", old.cache.same(&new.cache));
            for child in &mut self.children {
                child.update(ctx, new, env);
            }
        }
    }
}

impl GlyphGrid {
    pub fn new() -> impl Widget<Workspace> {
        Padding::new(
            8.0, // Adjust this value to increase or decrease padding
            GlyphGrid {
                children: Vec::new(),
            }
        )
    }
}

#[derive(Debug, Clone)]
struct GridInner {
    text: TextLayout<Arc<str>>,
}

impl GridInner {
    fn new() -> Self {
        GridInner {
            text: TextLayout::new(),
        }
    }
}

impl Widget<GridGlyph> for GridInner {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &GridGlyph, env: &Env) {
        let path = data.outline.clone();
        let bb = path.bounding_box();
        let geom = ctx.size().to_rect();
        // Increase padding by adjusting this value (e.g., from 0.55 to 0.5)
        let scale = geom.height() as f64 / data.upm;
        let scale = scale * 0.6; // Reduced from 0.55 to add more padding
        let scaled_width = bb.width() * scale as f64;
        let l_pad = ((geom.width() as f64 - scaled_width) / 2.25).round();
        let baseline = (geom.height() * 0.3) as f64;
        let affine = Affine::new([
            scale as f64,
            0.0,
            0.0,
            -scale as f64,
            l_pad,
            geom.height() - baseline,
        ]);

        let glyph_rect: Rect = geom - Insets::uniform(2.0);
        let rounded = glyph_rect.to_rounded_rect(16.0);
        ctx.fill(rounded, &env.get(theme::GROUND_2));
        ctx.stroke(rounded, &env.get(theme::GROUND_0), 2.0);
        if ctx.is_active() || data.is_selected {
            ctx.fill(rounded, &env.get(theme::FOCUS_3));
            ctx.stroke(rounded, &env.get(theme::FOCUS_2), 3.0);
        }
        let glyph_color = if data.is_placeholder {
            env.get(theme::FOCUS_1)
        } else {
            env.get(theme::FIGURE_4)
        };

        ctx.render_ctx.fill(affine * &*path, &glyph_color);

        let text_size = self.text.size();

        let xpos = geom.x0 + (geom.width() - text_size.width) / 2.0;
        let ypos = geom.max_y() - text_size.height;

        self.text.draw(ctx, (xpos, ypos - 8.0));
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _: &GridGlyph,
        env: &Env,
    ) -> Size {
        self.text.rebuild_if_needed(ctx.text(), env);
        bc.max()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GridGlyph, _env: &Env) {
        match event {
            Event::MouseDown(m) => {
                ctx.set_active(true);
                ctx.request_paint();
                if m.count == 1 {
                    data.is_selected = true;
                } else if m.count == 2 {
                    ctx.submit_command(EDIT_GLYPH.with(data.name.clone()));
                }
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &GridGlyph,
        env: &Env,
    ) {
        match event {
            LifeCycle::HotChanged(_) => ctx.request_paint(),
            LifeCycle::WidgetAdded => {
                self.text.set_text(data.name.clone());
                self.text.set_font(theme::UI_DETAIL_FONT);
                self.text.set_text_color(theme::FIGURE_4);
                self.text.rebuild_if_needed(ctx.text(), env);
            }
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &GridGlyph, new: &GridGlyph, _env: &Env) {
        if !old.same(new) {
            ctx.request_paint();
            // I don't know if our name can change? but if it can we need to rebuild our text
            // object.
            if !old.name.same(&new.name) {
                self.text.set_text(new.name.clone());
                ctx.request_layout();
            }
        }
        if self.text.needs_rebuild_after_update(ctx) {
            ctx.request_layout();
        }
    }
}