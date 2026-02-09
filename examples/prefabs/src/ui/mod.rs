mod panels;

use insim::insim::BtnStyle;
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
    ShowToolboxTab,
    ShowOptionsTab,
    TogglePrefabsSection,
    ToggleNudgeSection,
    ReloadYaml,
    SavePrefab(String),
    SpawnPrefab(usize),
    PaintedTextInput(String),
    RotateInput(String),
    SplineDistribInput(String),
    NudgeDistanceInput(String),
    NudgeNorth,
    NudgeSouth,
    NudgeEast,
    NudgeWest,
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

impl PrefabView {
    pub fn display_selection_info(&self) -> bool {
        self.display_selection_info
    }
}

impl kitcar::ui::Component for PrefabView {
    type Props = PrefabViewProps;
    type Message = PrefabViewMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            PrefabViewMessage::ShowToolboxTab => {
                self.top_tab = TopTab::Toolbox;
            },
            PrefabViewMessage::ShowOptionsTab => {
                self.top_tab = TopTab::Options;
            },
            PrefabViewMessage::TogglePrefabsSection => {
                self.expanded_section = if matches!(self.expanded_section, ExpandedSection::Prefabs)
                {
                    ExpandedSection::None
                } else {
                    ExpandedSection::Prefabs
                };
            },
            PrefabViewMessage::ToggleNudgeSection => {
                self.expanded_section = if matches!(self.expanded_section, ExpandedSection::Nudge) {
                    ExpandedSection::None
                } else {
                    ExpandedSection::Nudge
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

        let prefabs_section_style = if matches!(self.expanded_section, ExpandedSection::Prefabs) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };
        let nudge_section_style = if matches!(self.expanded_section, ExpandedSection::Nudge) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };
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

        let prefabs_panel = panels::prefabs_panel(&props.prefabs);
        let nudge_panel = panels::nudge_panel(props.nudge_distance_metres);
        let options_panel =
            panels::options_panel(props.compass_visible, self.display_selection_info);

        let compass_text = if let Some(compass_text) = props.compass_text.as_ref() {
            ui::text(compass_text, BtnStyle::default().dark().white())
                .w(48.)
                .h(5.)
                .mb(1.)
        } else {
            ui::empty()
        };

        let toolbox_panel = ui::container()
            .flex()
            .flex_col()
            .w(48.)
            .with_child(
                ui::clickable(
                    "Prefabs",
                    prefabs_section_style,
                    PrefabViewMessage::TogglePrefabsSection,
                )
                .mt(1.)
                .h(5.),
            )
            .with_child_if(
                prefabs_panel,
                matches!(self.expanded_section, ExpandedSection::Prefabs),
            )
            .with_child(
                ui::typein(
                    "Spline Distribution (m)",
                    BtnStyle::default().black().light(),
                    32,
                    PrefabViewMessage::SplineDistribInput,
                )
                .mt(1.)
                .block()
                .h(5.),
            )
            .with_child(
                ui::typein(
                    "Paint Text",
                    BtnStyle::default().black().light(),
                    64,
                    PrefabViewMessage::PaintedTextInput,
                )
                .mt(1.)
                .block()
                .h(5.),
            )
            .with_child(
                ui::typein(
                    "Rotate Selection (deg)",
                    BtnStyle::default().black().light(),
                    16,
                    PrefabViewMessage::RotateInput,
                )
                .mt(1.)
                .block()
                .h(5.),
            )
            .with_child(
                ui::clickable(
                    "Jiggle Selection",
                    BtnStyle::default().pale_blue().light().clickable(),
                    PrefabViewMessage::JiggleSelection,
                )
                .mt(1.)
                .h(5.),
            )
            .with_child(
                ui::clickable(
                    "Nudge Selection",
                    nudge_section_style,
                    PrefabViewMessage::ToggleNudgeSection,
                )
                .mt(1.)
                .h(5.),
            )
            .with_child_if(
                nudge_panel,
                matches!(self.expanded_section, ExpandedSection::Nudge),
            );

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
            .with_child(compass_text)
            .with_child(
                ui::container()
                    .flex()
                    .flex_row()
                    .w(48.)
                    .with_child(
                        ui::clickable(
                            "Toolbox",
                            toolbox_tab_style,
                            PrefabViewMessage::ShowToolboxTab,
                        )
                        .h(5.)
                        .flex_grow(1.0),
                    )
                    .with_child(
                        ui::clickable(
                            "Options",
                            options_tab_style,
                            PrefabViewMessage::ShowOptionsTab,
                        )
                        .h(5.)
                        .flex_grow(1.0),
                    ),
            )
            .with_child(if matches!(self.top_tab, TopTab::Toolbox) {
                toolbox_panel
            } else {
                options_panel
            })
    }
}
