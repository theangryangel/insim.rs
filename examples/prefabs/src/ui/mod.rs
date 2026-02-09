mod panels;
mod tabs;

use insim::{core::heading::Heading, insim::BtnStyle};
use kitcar::ui;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpandedSection {
    #[default]
    None,
    Prefabs,
    Nudge,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TopTab {
    #[default]
    Toolbox,
    Options,
}

#[derive(Debug, Clone, Default)]
pub struct PrefabListItem {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PrefabViewProps {
    pub ui_visible: bool,
    pub selection_count: usize,
    pub prefabs: Vec<PrefabListItem>,
    pub nudge_distance_metres: f64,
    pub compass_visible: bool,
    pub compass_text: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PrefabViewMessage {
    TopTab(TopTab),
    ExpandToolboxSection(ExpandedSection),
    ReloadYaml,
    SavePrefab(String),
    SpawnPrefab(usize),
    PaintedTextInput(String),
    RotateInput(String),
    SplineDistribInput(String),
    NudgeDistanceInput(String),
    Nudge(Heading),
    JiggleSelection,
    ToggleCompass,
    ToggleSelectionInfo,
}

pub struct PrefabView {
    top_tab: TopTab,
    expanded_section: ExpandedSection,
    display_selection_info: bool,
}

impl Default for PrefabView {
    fn default() -> Self {
        Self {
            top_tab: TopTab::Toolbox,
            expanded_section: ExpandedSection::None,
            display_selection_info: true,
        }
    }
}

impl kitcar::ui::Component for PrefabView {
    type Props = PrefabViewProps;
    type Message = PrefabViewMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            PrefabViewMessage::TopTab(i) => {
                self.top_tab = i;
            },
            PrefabViewMessage::ExpandToolboxSection(e) => {
                self.expanded_section = if self.expanded_section == e {
                    ExpandedSection::None
                } else {
                    e
                };
            },
            PrefabViewMessage::ToggleSelectionInfo => {
                self.display_selection_info = !self.display_selection_info;
            },
            _ => {},
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if !props.ui_visible {
            return ui::empty();
        }

        let toolbox_tab_style = if matches!(self.top_tab, TopTab::Toolbox) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };
        let options_tab_style = if matches!(self.top_tab, TopTab::Options) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };

        ui::container()
            .flex()
            .flex_col()
            .w(170.)
            .pt(7.)
            .items_end()
            .with_child(if self.display_selection_info {
                ui::text(
                    format!("Selection: {} object(s)", props.selection_count),
                    BtnStyle::default().dark().white(),
                )
                .w(48.)
                .h(5.)
                .mb(1.)
            } else {
                ui::empty()
            })
            .with_child(if let Some(compass_text) = props.compass_text.as_ref() {
                ui::text(compass_text, BtnStyle::default().dark().white())
                    .w(48.)
                    .h(5.)
                    .mb(1.)
            } else {
                ui::empty()
            })
            .with_child(
                ui::container()
                    .flex()
                    .flex_row()
                    .w(48.)
                    .with_child(
                        ui::clickable(
                            "Toolbox",
                            toolbox_tab_style,
                            PrefabViewMessage::TopTab(TopTab::Toolbox),
                        )
                        .h(5.)
                        .flex_grow(1.0),
                    )
                    .with_child(
                        ui::clickable(
                            "Options",
                            options_tab_style,
                            PrefabViewMessage::TopTab(TopTab::Options),
                        )
                        .h(5.)
                        .flex_grow(1.0),
                    ),
            )
            .with_child(match self.top_tab {
                TopTab::Toolbox => tabs::toolbox_tab(&props, self.expanded_section),
                TopTab::Options => {
                    tabs::options_tab(props.compass_visible, self.display_selection_info)
                },
            })
    }
}
