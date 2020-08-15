use crate::traits::UIElement;

use crate::traits::UIEvent;

use super::util::cursor_in_rect;

use piston::input::{
  UpdateArgs,
  RenderArgs,
  Key
};

// Import drawing helper functions
use graphics::{Context, rectangle, text, Transformed};

use graphics::character::CharacterCache;
use opengl_graphics::GlGraphics;
use opengl_graphics::GlyphCache;
use opengl_graphics::TextureSettings;

pub struct UIDropdown {
  id: usize,
  items: Vec<(usize, String)>,
  base_font_size: f64,
  position: [f64; 2],
  rect: [f64; 4],
  item_height: f64,
  padding: f64,
  draw_from_bottom: bool,
  font: graphics::glyph_cache::rusttype::GlyphCache<'static, (), opengl_graphics::Texture>
}

impl UIDropdown {
  pub fn create (id: usize, items: Vec<(usize, String)>, draw_up: bool, position: [f64; 2], min_width: f64, font_size: f64, font_path: String) -> Self {
    // Copy over the elements
    let mut i = Vec::new();
    for (idx, elem) in items.iter() {
      i.push((*idx, elem.clone()));
    }

    let padding = 5.0;

    let mut font = GlyphCache::new(font_path.as_str(), (), TextureSettings::new()).unwrap();

    let item_height = font_size + 2.0 * padding;
    let item_count = items.len() as f64;

    let top_x = position[0];
    let mut top_y = position[1];

    if draw_up {
      // Calculate top_y depending on the size
      top_y -= item_height * item_count;
    }

    // Now we need to get the width of that piece of sh**
    let mut item_width = 0.0;
    for (_idx, item) in items.iter() {
      let width = font.width(font_size as u32, item.as_str()).unwrap() + 2.0 * padding;
      if width > item_width {
        item_width = width;
      }
    }

    if item_width < min_width {
      item_width = min_width;
    }

    Self {
      id,
      items: i,
      base_font_size: font_size,
      rect: [top_x, top_y, item_width, item_height * item_count],
      position,
      padding,
      item_height,
      draw_from_bottom: draw_up,
      font
    }
  }
}

impl UIElement for UIDropdown {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, _args: &RenderArgs) {
    // Draws a dropdown
    let bg_color = [0.1, 0.2, 0.4, 1.0];
    let hover_color = [0.2, 0.6, 0.8, 1.0];
    let fg_color = [1.0, 1.0, 1.0, 1.0];

    // Now we have the correct x/y coords, the width and the height. Now: DRAW!
    rectangle(bg_color, self.rect, context.transform, gl);

    let mut i = -1.0;
    for (_idx, item) in self.items.iter() {
      // Draw em'
      i += 1.0;
      let item_y = self.rect[1] + i * self.item_height;
      if cursor_in_rect(self.position, [self.rect[0], item_y, self.rect[2], self.item_height]) {
        // Hover effect for the item
        rectangle(hover_color, [self.rect[0], item_y, self.rect[2], self.item_height], context.transform, gl);
      }

      text::Text::new_color(fg_color, self.base_font_size as u32).draw(
        item.as_str(),
        &mut self.font,
        &context.draw_state,
        context.transform.trans(self.rect[0] + self.padding, item_y + self.padding + self.base_font_size),
        gl
      ).unwrap();
    }
  }

  fn update (&mut self, _args: &UpdateArgs) {}
  fn on_cursor_state (&mut self, _is_over_window: bool) {}

  // /// Called whenever the cursor position changed
  fn on_cursor_movement (&mut self, x: f64, y: f64) {
    self.position = [x, y];
  }

  // /// Called when the left button has been pressed
  fn on_click (&mut self) -> Option<UIEvent> {
    // If the cursor is not in the rect there's nothing to do anyway
    if !cursor_in_rect(self.position, self.rect) {
      return None;
    }

    let mut i = -1.0;
    for (_idx, _item) in self.items.iter() {
      // Draw em'
      i += 1.0;
      let item_y = self.rect[1] + i * self.item_height;
      if cursor_in_rect(self.position, [self.rect[0], item_y, self.rect[2], self.item_height]) {
        // Cursor is within this element
        // Following event needs to be emitted:
        return Some(UIEvent::Selection(i as usize, self.id));
      }
    }

    None // Fallback
  }

  // /// Called when a key on the keyboard has been pressed
  fn on_keypress (&mut self, _key: Key) {
    // TODO: Handle up/down arrows when this thing has focus
  }
}
