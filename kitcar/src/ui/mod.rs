//! An implementation of a retained immediate mode UI.
//! A hybrid approach that combines the programming model of immediate mode with the
//! performance optimizations of retained mode.
//! Functions are called every frame, but we diff the output to minimise the rendering
//! requirements.
//! Each plugin will be responsible for it's own set of Ui's. Nothing shared except for the id_pool.
//! `Ui` represents the ui for a single connection.
pub mod id_pool;
pub mod instance_id;
pub mod renderer;
pub mod styled;
pub mod vdom;

use std::collections::HashMap;

pub use id_pool::ClickIdPool;
use insim::{
    identifiers::ConnectionId,
    insim::{BfnType, Mso, TinyType},
    Packet, WithRequestId,
};
pub use instance_id::InstanceIdPool;
pub use renderer::{UiRenderer, UiRendererDiff};
pub use styled::Styled;
use tokio::{sync::watch, task::JoinHandle};
pub use vdom::{Element, ElementKey};

#[derive(Default, Debug)]
pub struct ComponentResult {
    pub(crate) should_render: bool,
    // TODO: implement
    // pub(crate) commands: Vec<Box<dyn Any + Send>>,
}

impl ComponentResult {
    pub fn render(mut self) -> Self {
        self.should_render = true;
        self
    }

    // pub fn with_command(mut self, _cmd: impl Any + Send + 'static) -> Self {
    //     todo!()
    //     // self.commands.push(Box::new(cmd));
    //     // self
    // }
}

pub trait ComponentBehaviour {
    /// Get this component's instance id
    fn instance_id(&self) -> u32;

    /// Click occured for this connection
    fn on_click(&mut self, _click_id: &ElementKey) -> ComponentResult {
        ComponentResult::default()
    }

    /// Chat occurred for this connection
    fn on_mso(&mut self, _mso: &Mso) -> ComponentResult {
        ComponentResult::default()
    }

    /// Child components
    fn children_mut(&mut self) -> Option<HashMap<u32, &mut dyn ComponentBehaviour>>;
}

/// Trait for users to implement a Ui for a single connection
pub trait Component: ComponentBehaviour + Send + 'static {
    type Props: Clone + Sync + Send + 'static;

    /// Create a new instance of this Component
    fn mount(instance_ids: &mut InstanceIdPool, props: Self::Props) -> Self;

    /// Update props
    fn update(&mut self, props: Self::Props) -> ComponentResult;

    /// Render
    fn render(&self) -> Option<Element>;
}

/// Manager to implement Ui
#[derive(Debug)]
pub struct UiManager;

impl UiManager {
    pub fn spawn<U: Component>(
        signals: watch::Receiver<U::Props>,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut packet_rx = insim.subscribe();
            let mut active = HashMap::new();

            let _ = insim.send(TinyType::Ncn.with_request_id(1)).await?;

            while let Ok(packet) = packet_rx.recv().await {
                match packet {
                    Packet::Ncn(ncn) => {
                        let _ = active.entry(ncn.ucid).or_insert_with(|| {
                            Self::spawn_player_ui::<U>(ncn.ucid, signals.clone(), insim.clone())
                        });
                    },
                    Packet::Cnl(cnl) => {
                        if let Some(handle) = active.remove(&cnl.ucid) {
                            handle.abort();
                        }
                    },
                    _ => {},
                }
            }

            // FIXME: masking the error if one occurs
            Ok(())
        })
    }

    fn spawn_player_ui<U: Component>(
        ucid: ConnectionId,
        mut signals: watch::Receiver<U::Props>,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut instance_ids = InstanceIdPool::new();
            let props = signals.borrow().clone();
            let mut root = U::mount(&mut instance_ids, props);
            let mut renderer = UiRenderer::new(ClickIdPool::new());
            // Honor when a user blocks/requests buttons
            let mut blocked = false;

            // Initial render
            if !blocked {
                let element = root.render();
                if let Some(diff) = renderer.render(element, &ucid) {
                    insim.send_all(diff.into_merged()).await?;
                }
            }

            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    // Handle button clicks
                    Ok(packet) = packet_rx.recv() => {
                        let render_result = match packet {
                            Packet::Mso(mso) => {
                                if mso.ucid != ucid {
                                    None
                                } else {
                                    // FIXME: this should also traverse the component tree
                                    Some(root.on_mso(&mso))
                                }
                            },

                            Packet::Btc(btc) => {
                                if_chain::if_chain! {
                                    if btc.ucid == ucid;
                                    if !blocked;
                                    if let Some(key) = renderer.click_id_to_key(&btc.clickid);
                                    then {
                                        let mut res: Option<ComponentResult> = None;
                                        let mut stack = vec![&mut root as &mut dyn ComponentBehaviour];

                                        // Iterative search
                                        while let Some(component) = stack.pop() {
                                            if component.instance_id() == key.instance_id {
                                                res = Some(component.on_click(&key));
                                                break;
                                            }
                                            if let Some(next) = component.children_mut() {
                                                stack.extend(next.into_values());
                                            }
                                        }

                                        res
                                    } else {
                                        None
                                    }
                                }
                            },
                            Packet::Bfn(bfn) => {
                                if bfn.ucid != ucid {
                                    None
                                }
                                else if matches!(bfn.subt, BfnType::Clear | BfnType::UserClear) {
                                    blocked = true;
                                    renderer.clear();
                                    None
                                }
                                else if matches!(bfn.subt, BfnType::BtnRequest) {
                                    blocked = false;
                                    Some(ComponentResult::default().render())
                                } else {
                                    None
                                }
                            },

                            _ => {
                                None
                            }
                        };

                        if_chain::if_chain! {
                            if !blocked;
                            if let Some(result) = render_result;
                            if result.should_render;
                            then {
                                let element = root.render();
                                if let Some(diff) = renderer.render(element, &ucid) {
                                    insim.send_all(diff.into_merged()).await?;
                                }
                            }
                        }
                    },

                    // Handle signal changes
                    _ = signals.changed() => {
                        let props = signals.borrow_and_update().clone();
                        let res = root.update(props);
                        if blocked || !res.should_render {
                            continue;
                        }
                        let element = root.render();
                        if let Some(diff) = renderer.render(element, &ucid) {
                            insim.send_all(diff.into_merged()).await?;
                        }
                    }
                }
            }
        })
    }
}

const MAGIC_TEXT_RATIO: f32 = 0.2;

#[derive(Debug, Clone)]
pub struct WrapTextIter<'a> {
    remaining_text: &'a str,
    line_width: usize,
    lines_yielded: usize,
    max_lines: usize,
}

impl<'a> Iterator for WrapTextIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_text.is_empty() || self.lines_yielded >= self.max_lines {
            return None;
        }

        // Check for explicit newlines
        let mut scan_end_byte = self.remaining_text.len();
        if self.remaining_text.chars().count() > self.line_width {
            // This unwrap is safe because we've confirmed the character count is > line_width
            scan_end_byte = self
                .remaining_text
                .char_indices()
                .nth(self.line_width)
                .unwrap()
                .0;
        }

        if let Some(newline_byte_index) = &self.remaining_text[..scan_end_byte].find('\n') {
            self.lines_yielded += 1;
            let (line, rest) = self.remaining_text.split_at(*newline_byte_index);
            // Update remaining text to start *after* the newline character.
            self.remaining_text = &rest[1..];
            return Some(line);
        }

        self.lines_yielded += 1;

        // If the rest of the text fits on one line (and we know it has no early newline), this is the last piece.
        if self.remaining_text.chars().count() <= self.line_width {
            let line = self.remaining_text;
            self.remaining_text = ""; // Exhaust the iterator for the next call.
            return Some(line);
        }

        // Find whitespace to wrap at
        // Find the potential split point. We look for a space *before* the line_width limit.
        // We check a slice that is up to `line_width` characters long.
        let mut potential_end_byte = self.remaining_text.len();
        if let Some((byte_index, _)) = self.remaining_text.char_indices().nth(self.line_width) {
            // This is the byte index right at the character limit.
            potential_end_byte = byte_index;
        }

        let candidate_slice = &self.remaining_text[..potential_end_byte];

        // Find the last space within our candidate slice.
        let split_byte_index = if let Some(space_byte_index) = candidate_slice.rfind(' ') {
            // Found a space! This is our preferred split point.
            space_byte_index
        } else {
            // No space found, so we are forced to split the word at the character limit.
            potential_end_byte
        };

        // Create the line slice and update the remaining text state.
        let (line, rest) = self.remaining_text.split_at(split_byte_index);

        // The rest of the string needs to be trimmed of leading whitespace for the next iteration.
        self.remaining_text = rest.trim_start();

        // Return the current line, trimmed of any trailing space from the split.
        Some(line.trim_end())
    }
}

impl<'a> WrapTextIter<'a> {
    pub fn has_remaining_text(&self) -> bool {
        !self.remaining_text.is_empty()
    }
}

pub fn wrap_text<'a>(input: &'a str, height: u8, width: u8, max_lines: usize) -> WrapTextIter<'a> {
    let max_per_button = (width as f32 / (height as f32 * MAGIC_TEXT_RATIO)).floor();

    WrapTextIter {
        remaining_text: input,
        line_width: max_per_button as usize,
        lines_yielded: 0,
        max_lines,
    }
}
