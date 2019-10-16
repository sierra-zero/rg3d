use std::{
    cell::Cell,
    any::{Any, TypeId},
};
use crate::gui::{button::Button, Canvas, text::Text, VerticalAlignment, HorizontalAlignment, Thickness, Visibility, border::Border, scroll_bar::ScrollBar, scroll_viewer::ScrollViewer, image::Image, grid::Grid, scroll_content_presenter::ScrollContentPresenter, window::Window, event::UIEventHandler, EventSource, event::UIEvent, Drawable, Layout, UserInterface};
use rg3d_core::{
    color::Color, pool::Handle,
    math::{vec2::Vec2, Rect},
};
use crate::gui::draw::DrawingContext;

pub trait CustomUINodeKind: Any + EventSource {
    fn set_owner_handle(&mut self, handle: Handle<UINode>);
}

pub enum UINodeKind {
    Text(Text),
    Border(Border),
    Button(Button),
    ScrollBar(ScrollBar),
    ScrollViewer(ScrollViewer),
    Image(Image),
    /// Automatically arranges children by rows and columns
    Grid(Grid),
    /// Allows user to directly set position and size of a node
    Canvas(Canvas),
    /// Allows user to scroll content
    ScrollContentPresenter(ScrollContentPresenter),
    Window(Window),
    /// Custom user-defined node kind, allows to build your own UI nodes.
    User(Box<dyn CustomUINodeKind>),
}

impl Drawable for UINodeKind {
    fn draw(&mut self, drawing_context: &mut DrawingContext, bounds: &Rect<f32>, color: Color) {
        match self {
            UINodeKind::Text(text) => text.draw(drawing_context, bounds, color),
            UINodeKind::Border(border) => border.draw(drawing_context, bounds, color),
            UINodeKind::Image(image) => image.draw(drawing_context, bounds, color),
            _ => ()
        }
    }
}

impl Layout for UINodeKind {
    fn measure_override(&self, self_handle: Handle<UINode>, ui: &UserInterface, available_size: Vec2) -> Vec2 {
        match self {
            UINodeKind::Border(border) => border.measure_override(self_handle, ui, available_size),
            UINodeKind::Canvas(canvas) => canvas.measure_override(self_handle, ui, available_size),
            UINodeKind::Grid(grid) => grid.measure_override(self_handle, ui, available_size),
            UINodeKind::ScrollContentPresenter(scp) => scp.measure_override(self_handle, ui, available_size),
            UINodeKind::ScrollBar(scroll_bar) => scroll_bar.measure_override(self_handle, ui, available_size),
            _ => ui.default_measure_override(self_handle, available_size)
        }
    }

    fn arrange_override(&self, self_handle: Handle<UINode>, ui: &UserInterface, final_size: Vec2) -> Vec2 {
        match self {
            UINodeKind::Border(border) => border.arrange_override(self_handle, ui, final_size),
            UINodeKind::Canvas(canvas) => canvas.arrange_override(self_handle, ui, final_size),
            UINodeKind::Grid(grid) => grid.arrange_override(self_handle, ui, final_size),
            UINodeKind::ScrollContentPresenter(scp) => scp.arrange_override(self_handle, ui, final_size),
            UINodeKind::ScrollBar(scroll_bar) => scroll_bar.arrange_override(self_handle, ui, final_size),
            _ => ui.default_arrange_override(self_handle, final_size)
        }
    }
}

/// UI node is a building block for all UI widgets. For example button could be a node with
/// this structure
///
/// Border
///    Text
///
/// or
///
/// Border
///    SomeOtherNode
///      Child1
///      Child2
///      ...
///      ChildN
///
///
/// Notes. Some fields wrapped into Cell's to be able to modify them while in measure/arrange
/// stage. This is required evil, I can't just unwrap all the recursive calls in measure/arrange.
pub struct UINode {
    pub(in crate::gui) name: String,
    pub(in crate::gui) kind: UINodeKind,
    /// Desired position relative to parent node
    pub(in crate::gui) desired_local_position: Cell<Vec2>,
    /// Explicit width for node or automatic if NaN (means value is undefined). Default is NaN
    pub(in crate::gui) width: Cell<f32>,
    /// Explicit height for node or automatic if NaN (means value is undefined). Default is NaN
    pub(in crate::gui) height: Cell<f32>,
    /// Screen position of the node
    pub(in crate::gui) screen_position: Vec2,
    /// Desired size of the node after Measure pass.
    pub(in crate::gui) desired_size: Cell<Vec2>,
    /// Actual node local position after Arrange pass.
    pub(in crate::gui) actual_local_position: Cell<Vec2>,
    /// Actual size of the node after Arrange pass.
    pub(in crate::gui) actual_size: Cell<Vec2>,
    /// Minimum width and height
    pub(in crate::gui) min_size: Vec2,
    /// Maximum width and height
    pub(in crate::gui) max_size: Vec2,
    /// Overlay color of the node
    pub(in crate::gui) color: Color,
    /// Index of row to which this node belongs
    pub(in crate::gui) row: usize,
    /// Index of column to which this node belongs
    pub(in crate::gui) column: usize,
    /// Vertical alignment
    pub(in crate::gui) vertical_alignment: VerticalAlignment,
    /// Horizontal alignment
    pub(in crate::gui) horizontal_alignment: HorizontalAlignment,
    /// Margin (four sides)
    pub(in crate::gui) margin: Thickness,
    /// Current visibility state
    pub(in crate::gui) visibility: Visibility,
    pub(in crate::gui) children: Vec<Handle<UINode>>,
    pub(in crate::gui) parent: Handle<UINode>,
    /// Indices of commands in command buffer emitted by the node.
    pub(in crate::gui) command_indices: Vec<usize>,
    pub(in crate::gui) is_mouse_over: bool,
    pub(in crate::gui) measure_valid: Cell<bool>,
    pub(in crate::gui) arrange_valid: Cell<bool>,
    pub(in crate::gui) event_handler: Option<Box<UIEventHandler>>,
}

macro_rules! define_is_as {
    ($is:ident, $as_ref:ident, $as_mut:ident, $kind:ident, $result:ty) => {
        #[inline]
        pub fn $is(&self) -> bool {
            match self.kind {
                UINodeKind::$kind(_) => true,
                _ => false
            }
        }

        #[inline]
        pub fn $as_ref(&self) -> &$result {
            match self.kind {
                UINodeKind::$kind(ref val) => val,
                _ => panic!("Cast to {} failed!", stringify!($kind))
            }
        }

        #[inline]
        pub fn $as_mut(&mut self) -> &mut $result {
            match self.kind {
                UINodeKind::$kind(ref mut val) => val,
                _ => panic!("Cast to {} failed!", stringify!($kind))
            }
        }
    }
}

impl UINode {
    pub fn new(kind: UINodeKind) -> UINode {
        UINode {
            kind,
            name: String::new(),
            desired_local_position: Cell::new(Vec2::zero()),
            width: Cell::new(std::f32::NAN),
            height: Cell::new(std::f32::NAN),
            screen_position: Vec2::zero(),
            desired_size: Cell::new(Vec2::zero()),
            actual_local_position: Cell::new(Vec2::zero()),
            actual_size: Cell::new(Vec2::zero()),
            min_size: Vec2::make(0.0, 0.0),
            max_size: Vec2::make(std::f32::INFINITY, std::f32::INFINITY),
            color: Color::white(),
            row: 0,
            column: 0,
            vertical_alignment: VerticalAlignment::Stretch,
            horizontal_alignment: HorizontalAlignment::Stretch,
            margin: Thickness::zero(),
            visibility: Visibility::Visible,
            children: Vec::new(),
            parent: Handle::NONE,
            command_indices: Vec::new(),
            is_mouse_over: false,
            measure_valid: Cell::new(false),
            arrange_valid: Cell::new(false),
            event_handler: None,
        }
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    #[inline]
    pub fn set_width(&mut self, width: f32) {
        self.width.set(width);
    }

    #[inline]
    pub fn set_height(&mut self, height: f32) {
        self.height.set(height);
    }

    #[inline]
    pub fn set_desired_local_position(&self, pos: Vec2) {
        self.desired_local_position.set(pos);
    }

    #[inline]
    pub fn get_kind(&self) -> &UINodeKind {
        &self.kind
    }

    #[inline]
    pub fn set_vertical_alignment(&mut self, valign: VerticalAlignment) {
        self.vertical_alignment = valign;
    }

    #[inline]
    pub fn set_horizontal_alignment(&mut self, halign: HorizontalAlignment) {
        self.horizontal_alignment = halign;
    }

    #[inline]
    pub fn get_kind_mut(&mut self) -> &mut UINodeKind {
        &mut self.kind
    }

    #[inline]
    pub fn get_screen_bounds(&self) -> Rect<f32> {
        Rect::new(self.screen_position.x, self.screen_position.y, self.actual_size.get().x, self.actual_size.get().y)
    }

    #[inline]
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    #[inline]
    pub fn get_visibility(&self) -> Visibility {
        self.visibility
    }

    define_is_as!(is_scroll_bar, as_scroll_bar, as_scroll_bar_mut, ScrollBar, ScrollBar);
    define_is_as!(is_text, as_text, as_text_mut, Text, Text);
    define_is_as!(is_border, as_border, as_border_mut, Border, Border);
    define_is_as!(is_button, as_button, as_button_mut, Button, Button);
    define_is_as!(is_scroll_viewer, as_scroll_viewer, as_scroll_viewer_mut, ScrollViewer, ScrollViewer);
    define_is_as!(is_image, as_image, as_image_mut, Image, Image);
    define_is_as!(is_canvas, as_canvas, as_canvas_mut, Canvas, Canvas);
    define_is_as!(is_scroll_content_presenter, as_scroll_content_presenter,
     as_scroll_content_presenter_mut, ScrollContentPresenter, ScrollContentPresenter);
    define_is_as!(is_window, as_window, as_window_mut, Window, Window);

    #[inline]
    pub fn get_kind_id(&self) -> TypeId {
        match &self.kind {
            UINodeKind::ScrollBar(scroll_bar) => scroll_bar.type_id(),
            UINodeKind::Text(text) => text.type_id(),
            UINodeKind::Border(border) => border.type_id(),
            UINodeKind::Button(button) => button.type_id(),
            UINodeKind::ScrollViewer(scroll_viewer) => scroll_viewer.type_id(),
            UINodeKind::Image(image) => image.type_id(),
            UINodeKind::Grid(grid) => grid.type_id(),
            UINodeKind::Canvas(canvas) => canvas.type_id(),
            UINodeKind::ScrollContentPresenter(scp) => scp.type_id(),
            UINodeKind::Window(window) => window.type_id(),
            UINodeKind::User(user) => user.as_ref().type_id(),
        }
    }
}

impl EventSource for UINode {
    fn emit_event(&mut self) -> Option<UIEvent> {
        match self.kind {
            UINodeKind::Text(ref mut text) => text.emit_event(),
            UINodeKind::Border(ref mut border) => border.emit_event(),
            UINodeKind::Button(ref mut button) => button.emit_event(),
            UINodeKind::ScrollBar(ref mut scroll_bar) => scroll_bar.emit_event(),
            UINodeKind::ScrollViewer(ref mut scroll_viewer) => scroll_viewer.emit_event(),
            UINodeKind::Image(ref mut image) => image.emit_event(),
            UINodeKind::Grid(ref mut grid) => grid.emit_event(),
            UINodeKind::Canvas(ref mut canvas) => canvas.emit_event(),
            UINodeKind::ScrollContentPresenter(ref mut scp) => scp.emit_event(),
            UINodeKind::Window(ref mut window) => window.emit_event(),
            UINodeKind::User(ref mut user) => user.emit_event(),
        }
    }
}

impl Layout for UINode {
    fn measure_override(&self, self_handle: Handle<UINode>, ui: &UserInterface, available_size: Vec2) -> Vec2 {
        self.kind.measure_override(self_handle, ui, available_size)
    }

    fn arrange_override(&self, self_handle: Handle<UINode>, ui: &UserInterface, final_size: Vec2) -> Vec2 {
        self.kind.arrange_override(self_handle, ui, final_size)
    }
}