pub mod component;
pub mod id_pool;
pub mod ui;
pub mod vdom;

pub use component::Component;
pub use id_pool::ClickIdPool;
pub use ui::{Ui, UiDiff};
pub use vdom::{Element, ElementDiff};

// Each plugin will be responsible for it's own set of Ui's. Nothing shared except for the id_pool.
// Each Ui is per-connection.
// Ui will be declarative and we'll just prop drill, because thats probably good enough.
// Dont waste more time on signals, its not worth the effort.
// The average UI is probably going to be simple enough to do this way, and we'll just rerender at
// each plugin's pace.
//
// Workflow:
//  Create Element(s) -> call should_render on root node -> if should_render compute LayoutTree ->
//  Diff vs last LayoutTree -> render changes to Insim packets
//
// #[derive(Clone, PartialEq, Default)]
// pub struct AppProps {
//     pub bar: bool,
// }
//
// TODO: replace with a macro
//
// fn app(props: AppProps) -> VirtualDOM {
//     let mut children = Vec::new();
//
//     children.push(VirtualDOM::Button {
//         props: ButtonProps { text: "foo".to_string(), ..Default::default() },
//     });
//
//     if props.bar {
//         children.push(VirtualDOM::Button {
//             props: ButtonProps { text: "bar".to_string(), ..Default::default() },
//         });
//     }
//
//     VirtualDOM::Container {
//         props: ContainerProps::default(),
//         children,
//     }
// }
// let id_pool = ClickIdPool::new();
// let ui = Ui::new(id_pool, app_component);
// loop {
//  ...
//  let props = ...
//
//  if let Some(diff) = ui.render(ucid, props) {
//   .. output buttons
//  }
// }

#[cfg(test)]
mod test {
    use insim::{identifiers::ConnectionId, insim::BtnStyle};

    use super::*;

    struct App {}

    impl Component for App {
        type Props = AppProps;

        fn render(&self, props: &Self::Props) -> Element {
            if props.empty {
                return Element::Empty;
            }

            let mut children = Vec::new();

            children.push(Element::Button {
                text: "foo".to_string(),
                key: "1".into(),
                height: 5,
                width: 5,
                style: taffy::Style::DEFAULT,
                btnstyle: BtnStyle::default(),
            });

            if props.bar {
                children.push(Element::Button {
                    text: "bar".to_string(),
                    key: "2".into(),
                    height: 5,
                    width: 5,
                    style: taffy::Style::DEFAULT,
                    btnstyle: BtnStyle::default(),
                });
            }

            Element::Container {
                children,
                style: taffy::Style::DEFAULT,
            }
        }
    }

    #[derive(Clone, PartialEq, Default)]
    pub struct AppProps {
        pub empty: bool,
        pub bar: bool,
    }

    #[test]
    fn test_ui() {
        let mut ui = Ui::new(ClickIdPool::new(), App {});

        let diff = ui
            .render(
                ConnectionId::ALL,
                &AppProps {
                    empty: false,
                    bar: false,
                },
            )
            .expect("Initial render should render *something*");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        let expected_click_id = diff.to_update[0].clickid;

        assert_eq!(diff.to_update[0].text, "foo");

        let diff = ui.render(
            ConnectionId::ALL,
            &AppProps {
                empty: false,
                bar: false,
            },
        );

        // nothing changed
        assert!(diff.is_none());

        let diff = ui
            .render(
                ConnectionId::ALL,
                &AppProps {
                    empty: false,
                    bar: true,
                },
            )
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        assert_eq!(diff.to_update[0].text, "bar");
        assert_ne!(diff.to_update[0].clickid, expected_click_id); // we dont reuse an id

        let diff = ui
            .render(
                ConnectionId::ALL,
                &AppProps {
                    empty: true,
                    bar: true,
                },
            )
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_remove.len(), 2, "received diff: {:?}", diff);
    }
}
