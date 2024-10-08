//! The toolbar widget

//use druid::kurbo::{Affine, BezPath, Line, Shape};
use druid::kurbo::{Affine, BezPath, Line, Shape, Vec2};
use druid::widget::prelude::*;
use druid::widget::{Painter, WidgetExt};
use druid::{Color, Data, HotKey, KeyEvent, Rect, SysMods, WidgetPod};

//use crate::{consts, bez_cache};
use crate::{consts, theme};
use crate::tools::ToolId;

// TODO: move these to theme
const TOOLBAR_ITEM_SIZE: Size = Size::new(64.0, 64.0);
const TOOLBAR_ITEM_PADDING: f64 = 0.0; // Not sure why anyone would use this, but it's here if they want it
const TOOLBAR_ICON_PADDING: f64 = 10.0;
const TOOLBAR_BORDER_STROKE_WIDTH: f64 = 2.0;
const TOOLBAR_ITEM_STROKE_WIDTH: f64 = 1.25;
//const TOOLBAR_BG_DEFAULT: Color = Color::grey8(0x00);
//const TOOLBAR_BG_SELECTED: Color = Color::rgb8(0xff, 0xaa, 0x11);

struct ToolbarItem {
    icon: BezPath,
    name: ToolId,
    hotkey: HotKey,
}

/// The floating toolbar.
///
/// This is a very hacky implementation; it is not very
/// reusable, but can be refactored at a future date.
pub struct Toolbar {
    items: Vec<ToolbarItem>,
    selected: usize,
    widgets: Vec<WidgetPod<bool, Box<dyn Widget<bool>>>>,
}

/// A wrapper around control UI elements, drawing a drop shadow & rounded rect
pub struct FloatingPanel<W> {
    hide_panel: bool,
    inner: W,
}

impl Toolbar {
    fn new(items: Vec<ToolbarItem>) -> Self {
        let mut widgets = Vec::with_capacity(items.capacity());
        for icon in items.iter().map(|item| item.icon.clone()) {
            let widg = Painter::new(move |ctx, is_selected: &bool, env: &Env| {
                let color = if *is_selected {
                    // Toolbar BG selected
                    env.get(theme::FOCUS_3)
                } else {
                    // Toolbar BG default
                    env.get(theme::TOOLBAR_3)
                };
                let frame = ctx.size().to_rect();
                ctx.fill(frame, &color);
                if *is_selected {
                    //ctx.fill(frame, &env.get(theme::TOOLBAR_1));
                    ctx.fill(&icon, &env.get(theme::FOCUS_2));
                    ctx.stroke(&icon, &env.get(theme::FOCUS_1), TOOLBAR_ITEM_STROKE_WIDTH);
                } else {
                    //ctx.fill(frame, &env.get(theme::TOOLBAR_3));
                    ctx.fill(&icon, &env.get(theme::TOOLBAR_5));
                    ctx.stroke(&icon, &env.get(theme::TOOLBAR_1), TOOLBAR_ITEM_STROKE_WIDTH);
                };
            });

            let widg = widg.on_click(|ctx, selected, _| {
                *selected = true;
                ctx.request_paint();
            });
            widgets.push(WidgetPod::new(widg.boxed()));
        }

        Toolbar {
            items,
            widgets,
            selected: 0,
        }
    }

    pub fn tool_for_keypress(&self, key: &KeyEvent) -> Option<ToolId> {
        self.items
            .iter()
            .find(|tool| tool.hotkey.matches(key))
            .map(|tool| tool.name)
    }
}

impl<T: Data> Widget<T> for Toolbar {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(tool_id) = cmd.get(consts::cmd::SET_TOOL) {
                let sel = self.items.iter().position(|item| item.name == *tool_id);
                self.selected = sel.unwrap_or(self.selected);
                ctx.request_paint();
            }
        }

        for (i, child) in self.widgets.iter_mut().enumerate() {
            let mut is_selected = i == self.selected;
            child.event(ctx, event, &mut is_selected, env);

            if is_selected && i != self.selected {
                let tool = self.items[i].name;
                ctx.submit_command(consts::cmd::SET_TOOL.with(tool));
            }
        }

        // if there's a click here we don't want to pass it down to the child
        if matches!(event, Event::MouseDown(_) | Event::MouseUp(_)) {
            ctx.set_handled();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &T, env: &Env) {
        for (i, child) in self.widgets.iter_mut().enumerate() {
            let is_selected = i == self.selected;
            child.lifecycle(ctx, event, &is_selected, env);
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {
        //todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let constraints = BoxConstraints::tight(TOOLBAR_ITEM_SIZE);
        let mut x_pos = 0.0;

        for child in self.widgets.iter_mut() {
            // data doesn't matter here
            let size = child.layout(ctx, &constraints, &false, env);
            child.set_layout_rect(ctx, &false, env, Rect::from_origin_size((x_pos, 0.0), size));
            x_pos += TOOLBAR_ITEM_SIZE.width + TOOLBAR_ITEM_PADDING;
        }

        // Size doesn't account for stroke etc
        bc.constrain(Size::new(
            x_pos - TOOLBAR_ITEM_PADDING,
            TOOLBAR_ITEM_SIZE.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        for (i, child) in self.widgets.iter_mut().enumerate() {
            let is_selected = i == self.selected;
            child.paint(ctx, &is_selected, env);
        }

        let stroke_inset = TOOLBAR_BORDER_STROKE_WIDTH / 2.0;
        for child in self.widgets.iter().skip(1) {
            let child_frame = child.layout_rect();
            let line = Line::new(
                (child_frame.min_x() - stroke_inset, child_frame.min_y()),
                (child_frame.min_x() - stroke_inset, child_frame.max_y()),
            );
            //ctx.stroke(line, &Color::WHITE, TOOLBAR_BORDER_STROKE_WIDTH);
            ctx.stroke(line, &env.get(theme::TOOLBAR_1), TOOLBAR_BORDER_STROKE_WIDTH);
        }
    }
}

impl<W> FloatingPanel<W> {
    pub fn new(inner: W) -> Self {
        FloatingPanel {
            hide_panel: false,
            inner,
        }
    }

    /// return a reference to the inner widget.
    pub fn inner(&self) -> &W {
        &self.inner
    }
}

impl<T: Data, W: Widget<T>> Widget<T> for FloatingPanel<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.inner.event(ctx, event, data, env);
        if let Event::Command(cmd) = event {
            if let Some(in_temporary_preview) = cmd.get(consts::cmd::TOGGLE_PREVIEW_TOOL) {
                self.hide_panel = *in_temporary_preview;
                ctx.request_paint();
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.inner.layout(ctx, bc, data, env);
        ctx.set_paint_insets((0., 6.0, 6.0, 0.));
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if self.hide_panel {
            return;
        }
        let frame = ctx.size().to_rect();
        ctx.blurred_rect(frame + Vec2::new(2.0, 2.0), 4.0, &Color::grey(0.5));
        let rounded = frame.to_rounded_rect(5.0);
        //ctx.fill(rounded, &TOOLBAR_BG_DEFAULT);
        ctx.fill(rounded, &env.get(theme::TOOLBAR_4));
        ctx.with_save(|ctx| {
            ctx.clip(rounded);
            self.inner.paint(ctx, data, env);
        });
        //ctx.stroke(rounded, &Color::WHITE, TOOLBAR_BORDER_STROKE_WIDTH);
        ctx.stroke(rounded, &env.get(theme::TOOLBAR_1), TOOLBAR_BORDER_STROKE_WIDTH);
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        let select = ToolbarItem {
            name: "Select",
            icon: constrain_path(select_path()),
            hotkey: HotKey::new(None, "v"),
        };

        let pen = ToolbarItem {
            name: "Pen",
            icon: constrain_path(pen_path()),
            hotkey: HotKey::new(None, "p"),
        };

        let hyperpen = ToolbarItem {
            name: "HyperPen",
            icon: constrain_path(hyperpen_path()),
            hotkey: HotKey::new(SysMods::Shift, "P"),
        };

        let metaball = ToolbarItem {
            name: "Metaball",
            icon: constrain_path(metaball_path()),
            hotkey: HotKey::new(SysMods::Shift, "M"),
        };

        let preview = ToolbarItem {
            name: "Preview",
            icon: constrain_path(preview_path()),
            hotkey: HotKey::new(None, "h"),
        };

        let rectangle = ToolbarItem {
            name: "Rectangle",
            icon: constrain_path(rect_path()),
            hotkey: HotKey::new(None, "u"),
        };

        let ellipse = ToolbarItem {
            name: "Ellipse",
            icon: constrain_path(ellipse_path()),
            hotkey: HotKey::new(SysMods::Shift, "U"),
        };

        let knife = ToolbarItem {
            name: "Knife",
            icon: constrain_path(knife_path()),
            hotkey: HotKey::new(None, "e"),
        };

        let measure = ToolbarItem {
            name: "Measure",
            icon: constrain_path(measure_path()),
            hotkey: HotKey::new(None, "m"),
        };

        let star = ToolbarItem {
            name: "Star",
            icon: constrain_path(star_path()),
            hotkey: HotKey::new(None, "s"),
        };

        Toolbar::new(vec![
            select, pen, hyperpen, metaball, knife, preview, measure, rectangle, ellipse, star,
        ])
    }
}

fn constrain_path(mut path: BezPath) -> BezPath {
    let path_size = path.bounding_box().size();
    let icon_size = TOOLBAR_ITEM_SIZE.max_side() - TOOLBAR_ICON_PADDING * 2.0;
    let scale = icon_size / path_size.max_side();
    path.apply_affine(Affine::scale(scale));
    let center_offset = (TOOLBAR_ITEM_SIZE - (path_size * scale)).to_vec2() / 2.0;
    path.apply_affine(Affine::translate(center_offset));
    path
}

fn select_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((144.0, 500.0));
    bez.line_to((96.0, 349.0));
    bez.line_to((87.0, 345.0));
    bez.line_to((10.0, 408.0));
    bez.line_to((0.0, 406.0));
    bez.line_to((5.0, 5.0));
    bez.line_to((17.0, 0.0));
    bez.line_to((266.0, 286.0));
    bez.line_to((263.0, 296.0));
    bez.line_to((165.0, 313.0));
    bez.line_to((162.0, 321.0));
    bez.line_to((227.0, 456.0));
    bez.line_to((221.0, 465.0));
    bez.line_to((157.0, 500.0));
    bez.line_to((144.0, 500.0));
    bez.close_path();
    bez    

//    bez.move_to((110.0, 500.0));
//    bez.line_to((110.0, 380.0));
//    bez.line_to((2.0, 410.0));
//    bez.line_to((0.0, 410.0));
//    bez.line_to((159.0, 0.0));
//    bez.line_to((161.0, 0.0));
//    bez.line_to((320.0, 410.0));
//    bez.line_to((318.0, 410.0));
//    bez.line_to((210.0, 380.0));
//    bez.line_to((210.0, 500.0));
//    bez.line_to((110.0, 500.0));
//    bez.close_path();
//    bez
}

fn pen_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((40.0, 500.0));
    bez.line_to((240.0, 500.0));
    bez.line_to((240.0, 410.0));
    bez.line_to((40.0, 410.0));
    bez.line_to((40.0, 500.0));
    bez.close_path();

    bez.move_to((40.0, 410.0));
    bez.line_to((240.0, 410.0));
    bez.line_to((239.0, 370.0));
    bez.line_to((280.0, 290.0));
    bez.curve_to((240.0, 220.0), (205.0, 130.0), (195.0, 0.0));
    bez.line_to((85.0, 0.0));
    bez.curve_to((75.0, 130.0), (40.0, 220.0), (0.0, 290.0));
    bez.line_to((40.0, 370.0));
    bez.line_to((40.0, 410.0));
    bez.close_path();

    bez.move_to((140.0, 0.0));
    bez.line_to((140.0, 266.0));

    bez.move_to((173.0, 300.0));
    bez.curve_to((173.0, 283.0), (159.0, 267.0), (140.0, 267.0));
    bez.curve_to((121.0, 267.0), (107.0, 283.0), (107.0, 300.0));
    bez.curve_to((107.0, 317.0), (121.0, 333.0), (140.0, 333.0));
    bez.curve_to((159.0, 333.0), (173.0, 317.0), (173.0, 300.0));
    bez.close_path();
    bez
}

fn hyperpen_path() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((194.0, 237.0));
    bez.curve_to((194.0, 220.0), (162.0, 217.0), (162.0, 199.0));
    bez.curve_to((162.0, 184.0), (169.0, 169.0), (195.0, 169.0));
    bez.curve_to((218.0, 169.0), (242.0, 192.0), (242.0, 234.0));
    bez.curve_to((241.0, 285.0), (229.0, 307.0), (190.0, 310.0));
    bez.curve_to((144.0, 308.0), (69.0, 293.0), (69.0, 190.0));
    bez.curve_to((69.0, 126.0), (122.0, 66.0), (185.0, 66.0));
    bez.curve_to((290.0, 66.0), (335.0, 131.0), (336.0, 246.0));
    bez.curve_to((336.0, 323.0), (284.0, 356.0), (284.0, 392.0));
    bez.curve_to((284.0, 408.0), (300.0, 422.0), (322.0, 422.0));
    bez.curve_to((385.0, 422.0), (412.0, 364.0), (414.0, 254.0));
    bez.curve_to((414.0, 106.0), (345.0, 0.0), (187.0, 0.0));
    bez.curve_to((82.0, 0.0), (0.0, 78.0), (0.0, 175.0));
    bez.curve_to((0.0, 305.0), (51.0, 367.0), (177.0, 369.0));
    bez.curve_to((245.0, 367.0), (293.0, 304.0), (293.0, 251.0));
    bez.curve_to((294.0, 164.0), (247.0, 127.0), (189.0, 125.0));
    bez.curve_to((153.0, 125.0), (117.0, 149.0), (117.0, 207.0));
    bez.curve_to((117.0, 246.0), (133.0, 266.0), (160.0, 266.0));
    bez.curve_to((188.0, 266.0), (194.0, 252.0), (194.0, 237.0));
    bez.close_path();
    bez
}

fn metaball_path() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((333.0, 520.0));
    bez.curve_to((409.0, 520.0), (468.0, 462.0), (468.0, 385.0));
    bez.curve_to((468.0, 310.0), (416.0, 258.0), (344.0, 247.0));
    bez.curve_to((296.0, 240.0), (270.0, 202.0), (270.0, 128.0));
    bez.curve_to((270.0, 64.0), (217.0, 0.0), (137.0, 0.0));
    bez.curve_to((59.0, 0.0), (0.0, 58.0), (0.0, 128.0));
    bez.curve_to((0.0, 206.0), (48.0, 252.0), (98.0, 266.0));
    bez.curve_to((166.0, 285.0), (187.0, 299.0), (198.0, 401.0));
    bez.curve_to((203.0, 447.0), (240.0, 520.0), (333.0, 520.0));
    bez.close_path();
    bez.move_to((333.0, 453.0));
    bez.curve_to((296.0, 453.0), (265.0, 422.0), (265.0, 385.0));
    bez.curve_to((265.0, 347.0), (296.0, 317.0), (333.0, 317.0));
    bez.curve_to((370.0, 317.0), (401.0, 347.0), (401.0, 385.0));
    bez.curve_to((401.0, 422.0), (370.0, 453.0), (333.0, 453.0));
    bez.close_path();
    bez.move_to((333.0, 401.0));
    bez.curve_to((342.0, 401.0), (349.0, 393.0), (349.0, 385.0));
    bez.curve_to((349.0, 376.0), (342.0, 369.0), (333.0, 369.0));
    bez.curve_to((324.0, 369.0), (317.0, 376.0), (317.0, 385.0));
    bez.curve_to((317.0, 393.0), (324.0, 401.0), (333.0, 401.0));
    bez.close_path();
    bez.move_to((137.0, 144.0));
    bez.curve_to((146.0, 144.0), (153.0, 136.0), (153.0, 128.0));
    bez.curve_to((153.0, 119.0), (146.0, 112.0), (137.0, 112.0));
    bez.curve_to((128.0, 112.0), (121.0, 119.0), (121.0, 128.0));
    bez.curve_to((121.0, 136.0), (128.0, 144.0), (137.0, 144.0));
    bez.close_path();
    bez.move_to((137.0, 196.0));
    bez.curve_to((100.0, 196.0), (69.0, 165.0), (69.0, 128.0));
    bez.curve_to((69.0, 90.0), (100.0, 60.0), (137.0, 60.0));
    bez.curve_to((174.0, 60.0), (205.0, 90.0), (205.0, 128.0));
    bez.curve_to((205.0, 165.0), (174.0, 196.0), (137.0, 196.0));
    bez.close_path();
    bez
}

fn knife_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((30.0, 500.0));
    bez.line_to((190.0, 500.0));
    bez.line_to((190.0, 410.0));
    bez.line_to((30.0, 410.0));
    bez.line_to((30.0, 500.0));
    bez.close_path();

    bez.move_to((40.0, 360.0));
    bez.line_to((180.0, 360.0));
    bez.line_to((180.0, 330.0));
    bez.line_to((220.0, 290.0));
    bez.line_to((42.0, 0.0));
    bez.line_to((40.0, 0.0));
    bez.line_to((40.0, 360.0));
    bez.close_path();

    bez.move_to((30.0, 410.0));
    bez.line_to((190.0, 410.0));
    bez.curve_to((205.0, 410.0), (220.0, 405.0), (220.0, 385.0));
    bez.curve_to((220.0, 365.0), (205.0, 360.0), (190.0, 360.0));
    bez.line_to((30.0, 360.0));
    bez.curve_to((15.0, 360.0), (0.0, 365.0), (0.0, 385.0));
    bez.curve_to((0.0, 405.0), (15.0, 410.0), (30.0, 410.0));
    bez.close_path();
    bez
}

fn preview_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((130.0, 500.0));
    bez.line_to((310.0, 500.0));
    bez.line_to((310.0, 410.0));
    bez.curve_to((336.0, 375.0), (360.0, 351.0), (360.0, 310.0));
    bez.line_to((360.0, 131.0));
    bez.curve_to((360.0, 89.0), (352.0, 70.0), (336.0, 70.0));
    bez.curve_to((316.0, 70.0), (310.0, 85.0), (310.0, 101.0));
    bez.curve_to((310.0, 60.0), (309.0, 20.0), (280.0, 20.0));
    bez.curve_to((260.0, 20.0), (250.0, 36.0), (250.0, 60.0));
    bez.curve_to((250.0, 26.0), (242.0, 0.0), (216.0, 0.0));
    bez.curve_to((192.0, 0.0), (180.0, 16.0), (180.0, 75.0));
    bez.curve_to((180.0, 48.0), (169.0, 30.0), (150.0, 30.0));
    bez.curve_to((130.0, 30.0), (120.0, 53.0), (120.0, 75.0));
    bez.line_to((120.0, 250.0));
    bez.curve_to((120.0, 270.0), (110.0, 270.0), (100.0, 270.0));
    bez.curve_to((85.0, 270.0), (77.0, 264.0), (70.0, 250.0));
    bez.curve_to((45.0, 199.0), (32.0, 190.0), (20.0, 190.0));
    bez.curve_to((8.0, 190.0), (0.0, 197.0), (0.0, 210.0));
    bez.curve_to((0.0, 234.0), (19.0, 313.0), (30.0, 330.0));
    bez.curve_to((41.0, 347.0), (87.0, 383.0), (130.0, 410.0));
    bez.line_to((130.0, 500.0));
    bez.close_path();

    bez.move_to((130.0, 410.0));
    bez.line_to((310.0, 410.0));

    bez.move_to((180.0, 75.0));
    bez.line_to((180.0, 210.0));

    bez.move_to((250.0, 60.0));
    bez.line_to((250.0, 210.0));

    bez.move_to((310.0, 101.0));
    bez.line_to((310.0, 220.0));
    bez
}

fn measure_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((0.0, 500.0));
    bez.line_to((140.0, 500.0));
    bez.line_to((140.0, 0.0));
    bez.line_to((0.0, 0.0));
    bez.line_to((0.0, 500.0));
    bez.close_path();

    bez.move_to((190.0, 0.0));
    bez.line_to((330.0, 0.0));

    bez.move_to((190.0, 500.0));
    bez.line_to((330.0, 500.0));

    bez.move_to((210.0, 100.0));
    bez.line_to((310.0, 100.0));
    bez.line_to((260.0, 10.0));
    bez.line_to((210.0, 100.0));
    bez.close_path();

    bez.move_to((210.0, 400.0));
    bez.line_to((310.0, 400.0));
    bez.line_to((260.0, 490.0));
    bez.line_to((210.0, 400.0));
    bez.close_path();

    bez.move_to((260.0, 100.0));
    bez.line_to((260.0, 400.0));

    bez.move_to((70.0, 350.0));
    bez.line_to((140.0, 350.0));

    bez.move_to((100.0, 400.0));
    bez.line_to((140.0, 400.0));

    bez.move_to((50.0, 450.0));
    bez.line_to((140.0, 450.0));

    bez.move_to((100.0, 300.0));
    bez.line_to((140.0, 300.0));

    bez.move_to((50.0, 250.0));
    bez.line_to((140.0, 250.0));

    bez.move_to((70.0, 150.0));
    bez.line_to((140.0, 150.0));

    bez.move_to((100.0, 200.0));
    bez.line_to((140.0, 200.0));

    bez.move_to((100.0, 100.0));
    bez.line_to((140.0, 100.0));

    bez.move_to((50.0, 50.0));
    bez.line_to((140.0, 50.0));
    bez
}

fn rect_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((0.0, 500.0));
    bez.line_to((220.0, 500.0));
    bez.line_to((220.0, 0.0));
    bez.line_to((0.0, 0.0));
    bez.line_to((0.0, 500.0));
    bez.close_path();
    bez
}

fn ellipse_path() -> BezPath {
    let mut bez = BezPath::new();

    bez.move_to((110.0, 0.0));
    bez.curve_to((50.0, 0.0), (0.0, 100.0), (0.0, 240.0));
    bez.curve_to((0.0, 380.0), (50.0, 480.0), (110.0, 480.0));
    bez.curve_to((170.0, 480.0), (220.0, 380.0), (220.0, 240.0));
    bez.curve_to((220.0, 100.0), (170.0, 0.0), (110.0, 0.0));
    bez.close_path();
    bez
}

fn star_path() -> BezPath {
    let mut bez = BezPath::new();
    
    // Define the star path here
    // This is a simple 5-point star. Adjust as needed.
    bez.move_to((150.0, 0.0));
    bez.line_to((185.0, 115.0));
    bez.line_to((300.0, 115.0));
    bez.line_to((205.0, 185.0));
    bez.line_to((240.0, 300.0));
    bez.line_to((150.0, 230.0));
    bez.line_to((60.0, 300.0));
    bez.line_to((95.0, 185.0));
    bez.line_to((0.0, 115.0));
    bez.line_to((115.0, 115.0));
    bez.close_path();
    
    bez
}
