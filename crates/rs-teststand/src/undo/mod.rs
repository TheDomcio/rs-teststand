//! Undo and redo support.
//!
//! The engine records each edit as an undo item. [`UndoItemCreator`] brackets
//! the edits and produces an [`UndoItem`]; [`UndoStack`] holds the undo and redo
//! collections and drives them.
//!
//! ```no_run
//! use rs_teststand::{EditKind, Engine};
//!
//! let engine = Engine::new()?;
//! let file = engine.new_sequence_file()?;
//! let creator = engine.new_undo_item_creator(
//!     EditKind::ChangeObject,
//!     &file.as_property_object_file()?,
//!     "edit",
//! )?;
//! // Make an edit through the file, then:
//! let stack = engine.new_undo_stack()?;
//! let item = creator.create_and_post_undo_item(Default::default())?;
//! stack.push(&item)?;
//! stack.undo()?;
//! # Ok::<(), rs_teststand::Error>(())
//! ```

mod create_undo_item_option;
mod edit_kind;
mod locations;
mod undo_item;
mod undo_item_creator;
mod undo_items;
mod undo_stack;

pub use create_undo_item_option::CreateUndoItemOption;
pub use edit_kind::EditKind;
pub use locations::Locations;
pub use undo_item::UndoItem;
pub use undo_item_creator::UndoItemCreator;
pub use undo_items::UndoItems;
pub use undo_stack::UndoStack;
